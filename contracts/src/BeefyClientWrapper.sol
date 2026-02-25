// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.33;

import {BeefyClient} from "./BeefyClient.sol";

interface IGateway {
    function BEEFY_CLIENT() external view returns (address);
}

/**
 * @title BeefyClientWrapper
 * @dev Forwards BeefyClient submissions and refunds gas costs to relayers.
 * Anyone can relay. Refunds are only paid when the relayer advances the light
 * client by at least `refundTarget` blocks, ensuring meaningful progress.
 *
 * The BeefyClient address is resolved dynamically from the Gateway (via GatewayProxy),
 * so after a Gateway upgrade this wrapper automatically points to the new BeefyClient.
 *
 * This contract is permissionless and stateless (aside from in-flight ticket tracking).
 * Configuration is immutable. To change parameters, deploy a new instance.
 */
contract BeefyClientWrapper {
    event CostCredited(address indexed relayer, bytes32 indexed commitmentHash, uint256 cost);
    event SubmissionRefunded(address indexed relayer, uint256 progress, uint256 refundAmount);

    error InvalidAddress();
    error NotTicketOwner();
    error TicketAlreadyOwned();
    error InsufficientProgress();

    struct PendingTicket {
        address owner;
        uint256 creditedCost;
        uint64 createdAt;
    }

    // Base transaction gas cost (intrinsic gas for any Ethereum transaction)
    uint256 private constant BASE_TX_GAS = 21000;

    address public immutable gateway;

    // Ticket tracking (for multi-step submission)
    mapping(bytes32 => PendingTicket) public pendingTickets;

    // Refund configuration (immutable)
    uint256 public immutable maxGasPrice;
    uint256 public immutable maxRefundAmount;
    uint256 public immutable refundTarget; // Blocks of progress for 100% gas refund (e.g., 350 = ~35 min)
    uint256 public immutable ticketTimeout; // Seconds before a pending ticket expires

    // Highest commitment block number currently in progress (helps relayers avoid duplicate work)
    uint256 public highestPendingBlock;
    uint256 public highestPendingBlockTimestamp;

    constructor(
        address _gateway,
        uint256 _maxGasPrice,
        uint256 _maxRefundAmount,
        uint256 _refundTarget,
        uint256 _ticketTimeout
    ) {
        if (_gateway == address(0)) {
            revert InvalidAddress();
        }

        gateway = _gateway;
        maxGasPrice = _maxGasPrice;
        maxRefundAmount = _maxRefundAmount;
        refundTarget = _refundTarget;
        ticketTimeout = _ticketTimeout;
    }

    /* Beefy Client Proxy Functions */

    function submitInitial(
        BeefyClient.Commitment calldata commitment,
        uint256[] calldata bitfield,
        BeefyClient.ValidatorProof calldata proof
    ) external {
        uint256 startGas = gasleft();

        // Revert early if commitment won't make enough progress for a refund
        uint64 latestBeefy = _beefyClient().latestBeefyBlock();
        if (commitment.blockNumber <= latestBeefy || commitment.blockNumber - latestBeefy < refundTarget) {
            revert InsufficientProgress();
        }

        // Check if ticket is already owned (prevent race condition between relayers)
        bytes32 commitmentHash = _beefyClient().computeCommitmentHash(commitment);
        PendingTicket storage ticket = pendingTickets[commitmentHash];
        if (ticket.owner != address(0)) {
            // Allow overwriting only if the ticket has expired
            if (block.timestamp < ticket.createdAt + ticketTimeout) {
                revert TicketAlreadyOwned();
            }
            // Expired ticket — clear it so a new relayer can take over
            delete pendingTickets[commitmentHash];
        }

        _beefyClient().submitInitial(commitment, bitfield, proof);

        pendingTickets[commitmentHash] =
            PendingTicket({owner: msg.sender, creditedCost: 0, createdAt: uint64(block.timestamp)});

        // Track highest pending block so other relayers can check before starting
        if (commitment.blockNumber > highestPendingBlock) {
            highestPendingBlock = commitment.blockNumber;
            highestPendingBlockTimestamp = block.timestamp;
        }

        _creditCost(startGas, commitmentHash);
    }

    function commitPrevRandao(bytes32 commitmentHash) external {
        uint256 startGas = gasleft();

        if (pendingTickets[commitmentHash].owner != msg.sender) {
            revert NotTicketOwner();
        }

        _beefyClient().commitPrevRandao(commitmentHash);

        _creditCost(startGas, commitmentHash);
    }

    function submitFinal(
        BeefyClient.Commitment calldata commitment,
        uint256[] calldata bitfield,
        BeefyClient.ValidatorProof[] calldata proofs,
        BeefyClient.MMRLeaf calldata leaf,
        bytes32[] calldata leafProof,
        uint256 leafProofOrder
    ) external {
        uint256 startGas = gasleft();

        // Capture previous state for progress calculation
        uint64 previousBeefyBlock = _beefyClient().latestBeefyBlock();

        bytes32 commitmentHash = _beefyClient().computeCommitmentHash(commitment);
        if (pendingTickets[commitmentHash].owner != msg.sender) {
            revert NotTicketOwner();
        }

        _beefyClient().submitFinal(commitment, bitfield, proofs, leaf, leafProof, leafProofOrder);

        // Calculate progress
        uint256 progress = commitment.blockNumber - previousBeefyBlock;

        // Clear highest pending block if light client has caught up
        if (_beefyClient().latestBeefyBlock() >= highestPendingBlock) {
            highestPendingBlock = 0;
            highestPendingBlockTimestamp = 0;
        }

        uint256 previousCost = pendingTickets[commitmentHash].creditedCost;
        delete pendingTickets[commitmentHash];

        _refundWithProgress(startGas, previousCost, progress);
    }

    function submitFiatShamir(
        BeefyClient.Commitment calldata commitment,
        uint256[] calldata bitfield,
        BeefyClient.ValidatorProof[] calldata proofs,
        BeefyClient.MMRLeaf calldata leaf,
        bytes32[] calldata leafProof,
        uint256 leafProofOrder
    ) external {
        uint256 startGas = gasleft();

        // Capture previous state for progress calculation
        uint64 previousBeefyBlock = _beefyClient().latestBeefyBlock();

        _beefyClient().submitFiatShamir(commitment, bitfield, proofs, leaf, leafProof, leafProofOrder);

        // Calculate progress
        uint256 progress = commitment.blockNumber - previousBeefyBlock;

        // Clear highest pending block if light client has caught up
        if (_beefyClient().latestBeefyBlock() >= highestPendingBlock) {
            highestPendingBlock = 0;
            highestPendingBlockTimestamp = 0;
        }

        _refundWithProgress(startGas, 0, progress);
    }

    /* Internal Functions */

    function _beefyClient() internal view returns (BeefyClient) {
        return BeefyClient(IGateway(gateway).BEEFY_CLIENT());
    }

    function _effectiveGasPrice() internal view returns (uint256) {
        return tx.gasprice < maxGasPrice ? tx.gasprice : maxGasPrice;
    }

    function _creditCost(uint256 startGas, bytes32 commitmentHash) internal {
        uint256 gasUsed = startGas - gasleft() + BASE_TX_GAS;
        uint256 cost = gasUsed * _effectiveGasPrice();
        pendingTickets[commitmentHash].creditedCost += cost;
        emit CostCredited(msg.sender, commitmentHash, cost);
    }

    /**
     * @dev Calculate and send refund if progress meets threshold.
     *
     * Refund if progress >= refundTarget.
     */
    function _refundWithProgress(uint256 startGas, uint256 previousCost, uint256 progress) internal {
        if (progress < refundTarget) {
            return;
        }

        uint256 currentGas = startGas - gasleft() + BASE_TX_GAS;
        uint256 currentCost = currentGas * _effectiveGasPrice();
        uint256 refundAmount = previousCost + currentCost;

        if (refundAmount > maxRefundAmount) {
            refundAmount = maxRefundAmount;
        }

        if (refundAmount > 0 && address(this).balance >= refundAmount) {
            (bool success,) = payable(msg.sender).call{value: refundAmount}("");
            if (success) {
                emit SubmissionRefunded(msg.sender, progress, refundAmount);
            }
        }
    }

    receive() external payable {}
}
