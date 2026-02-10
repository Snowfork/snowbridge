// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.33;

import {IBeefyClient} from "./interfaces/IBeefyClient.sol";

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
 */
contract BeefyClientWrapper {
    event CostCredited(address indexed relayer, bytes32 indexed commitmentHash, uint256 cost);
    event SubmissionRefunded(address indexed relayer, uint256 progress, uint256 refundAmount);
    event FundsDeposited(address indexed depositor, uint256 amount);
    event FundsWithdrawn(address indexed recipient, uint256 amount);

    error Unauthorized();
    error InvalidAddress();
    error NotTicketOwner();
    error TicketAlreadyOwned();
    error TransferFailed();

    // Base transaction gas cost (intrinsic gas for any Ethereum transaction)
    uint256 private constant BASE_TX_GAS = 21000;

    address public owner;
    address public immutable gateway;

    // Ticket tracking (for multi-step submission)
    mapping(bytes32 => address) public ticketOwner;
    mapping(bytes32 => uint256) public creditedCost; // Accumulated ETH cost from previous steps

    // Refund configuration
    uint256 public maxGasPrice;
    uint256 public maxRefundAmount;

    // Progress-based refund target
    uint256 public refundTarget; // Blocks of progress for 100% gas refund (e.g., 350 = ~35 min)

    // Highest commitment block number currently in progress (helps relayers avoid duplicate work)
    uint256 public highestPendingBlock;
    uint256 public highestPendingBlockTimestamp;

    constructor(
        address _gateway,
        address _owner,
        uint256 _maxGasPrice,
        uint256 _maxRefundAmount,
        uint256 _refundTarget
    ) {
        if (_gateway == address(0) || _owner == address(0)) {
            revert InvalidAddress();
        }

        gateway = _gateway;
        owner = _owner;
        maxGasPrice = _maxGasPrice;
        maxRefundAmount = _maxRefundAmount;
        refundTarget = _refundTarget;
    }

    /* Beefy Client Proxy Functions */

    function submitInitial(
        IBeefyClient.Commitment calldata commitment,
        uint256[] calldata bitfield,
        IBeefyClient.ValidatorProof calldata proof
    ) external {
        uint256 startGas = gasleft();

        // Check if ticket is already owned (prevent race condition between relayers)
        bytes32 commitmentHash = _beefyClient().computeCommitmentHash(commitment);
        if (ticketOwner[commitmentHash] != address(0)) {
            revert TicketAlreadyOwned();
        }

        _beefyClient().submitInitial(commitment, bitfield, proof);

        ticketOwner[commitmentHash] = msg.sender;

        // Track highest pending block so other relayers can check before starting
        if (commitment.blockNumber > highestPendingBlock) {
            highestPendingBlock = commitment.blockNumber;
            highestPendingBlockTimestamp = block.timestamp;
        }

        _creditCost(startGas, commitmentHash);
    }

    function commitPrevRandao(bytes32 commitmentHash) external {
        uint256 startGas = gasleft();

        if (ticketOwner[commitmentHash] != msg.sender) {
            revert NotTicketOwner();
        }

        _beefyClient().commitPrevRandao(commitmentHash);

        _creditCost(startGas, commitmentHash);
    }

    function submitFinal(
        IBeefyClient.Commitment calldata commitment,
        uint256[] calldata bitfield,
        IBeefyClient.ValidatorProof[] calldata proofs,
        IBeefyClient.MMRLeaf calldata leaf,
        bytes32[] calldata leafProof,
        uint256 leafProofOrder
    ) external {
        uint256 startGas = gasleft();

        // Capture previous state for progress calculation
        uint64 previousBeefyBlock = _beefyClient().latestBeefyBlock();

        bytes32 commitmentHash = _beefyClient().computeCommitmentHash(commitment);
        if (ticketOwner[commitmentHash] != msg.sender) {
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

        uint256 previousCost = creditedCost[commitmentHash];
        delete creditedCost[commitmentHash];
        delete ticketOwner[commitmentHash];

        _refundWithProgress(startGas, previousCost, progress);
    }

    /**
     * @dev Submit a Fiat-Shamir proof. This is a single-step submission that doesn't require
     * ticket tracking or gas refunds since there's no multi-step flow to protect.
     * Gas refunds are only provided for the multi-step submitInitial -> commitPrevRandao -> submitFinal flow.
     */
    function submitFiatShamir(
        IBeefyClient.Commitment calldata commitment,
        uint256[] calldata bitfield,
        IBeefyClient.ValidatorProof[] calldata proofs,
        IBeefyClient.MMRLeaf calldata leaf,
        bytes32[] calldata leafProof,
        uint256 leafProofOrder
    ) external {
        _beefyClient().submitFiatShamir(commitment, bitfield, proofs, leaf, leafProof, leafProofOrder);

        // Clear highest pending block if light client has caught up
        if (_beefyClient().latestBeefyBlock() >= highestPendingBlock) {
            highestPendingBlock = 0;
            highestPendingBlockTimestamp = 0;
        }
    }

    /**
     * @dev Abandon a ticket. Useful if another relayer is competing for the same commitment.
     * Credited cost is forfeited when clearing a ticket.
     */
    function clearTicket(bytes32 commitmentHash) external {
        if (ticketOwner[commitmentHash] != msg.sender) {
            revert NotTicketOwner();
        }

        delete creditedCost[commitmentHash];
        delete ticketOwner[commitmentHash];
    }

    /* Internal Functions */

    function _beefyClient() internal view returns (IBeefyClient) {
        return IBeefyClient(IGateway(gateway).BEEFY_CLIENT());
    }

    function _checkOwner() internal view {
        if (msg.sender != owner) {
            revert Unauthorized();
        }
    }

    function _effectiveGasPrice() internal view returns (uint256) {
        return tx.gasprice < maxGasPrice ? tx.gasprice : maxGasPrice;
    }

    function _creditCost(uint256 startGas, bytes32 commitmentHash) internal {
        uint256 gasUsed = startGas - gasleft() + BASE_TX_GAS;
        uint256 cost = gasUsed * _effectiveGasPrice();
        creditedCost[commitmentHash] += cost;
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

    /* Admin Functions */

    function setMaxGasPrice(uint256 _maxGasPrice) external {
        _checkOwner();
        maxGasPrice = _maxGasPrice;
    }

    function setMaxRefundAmount(uint256 _maxRefundAmount) external {
        _checkOwner();
        maxRefundAmount = _maxRefundAmount;
    }

    function setRefundTarget(uint256 _refundTarget) external {
        _checkOwner();
        refundTarget = _refundTarget;
    }

    function withdrawFunds(address payable recipient, uint256 amount) external {
        _checkOwner();
        if (recipient == address(0)) {
            revert InvalidAddress();
        }

        (bool success,) = recipient.call{value: amount}("");
        if (!success) {
            revert TransferFailed();
        }

        emit FundsWithdrawn(recipient, amount);
    }

    function transferOwnership(address newOwner) external {
        _checkOwner();
        if (newOwner == address(0)) {
            revert InvalidAddress();
        }
        owner = newOwner;
    }

    receive() external payable {
        emit FundsDeposited(msg.sender, msg.value);
    }
}
