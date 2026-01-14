// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.28;

import {IBeefyClient} from "./interfaces/IBeefyClient.sol";
import {ScaleCodec} from "./utils/ScaleCodec.sol";

/**
 * @title BeefyClientWrapper
 * @dev Forwards BeefyClient submissions and refunds gas costs to relayers.
 * Anyone can relay. Uses progress-based refunds: the more blocks a relayer
 * advances the light client, the higher percentage of gas refund and rewards they receive.
 */
contract BeefyClientWrapper {
    event GasCredited(address indexed relayer, bytes32 indexed commitmentHash, uint256 gasUsed);
    event SubmissionRefunded(
        address indexed relayer, uint256 progress, uint256 refundAmount, uint256 rewardAmount, uint256 totalGasUsed
    );
    event FundsDeposited(address indexed depositor, uint256 amount);
    event FundsWithdrawn(address indexed recipient, uint256 amount);
    event RewardPoolFunded(address indexed funder, uint256 amount);

    error Unauthorized();
    error InvalidAddress();
    error NotTicketOwner();
    error TransferFailed();

    address public owner;
    IBeefyClient public beefyClient;

    // Ticket tracking (for multi-step submission)
    mapping(bytes32 => address) public ticketOwner;
    mapping(bytes32 => uint256) public creditedGas;

    // Refund configuration
    uint256 public maxGasPrice;
    uint256 public maxRefundAmount;

    // Progress-based refund/reward targets
    uint256 public refundTarget; // Blocks of progress for 100% gas refund (e.g., 300 = ~30 min)
    uint256 public rewardTarget; // Blocks of progress for 100% reward (e.g., 2400 = ~4 hours)
    uint256 public rewardPool; // Available reward pool

    constructor(
        address _beefyClient,
        address _owner,
        uint256 _maxGasPrice,
        uint256 _maxRefundAmount,
        uint256 _refundTarget,
        uint256 _rewardTarget
    ) {
        if (_beefyClient == address(0) || _owner == address(0)) {
            revert InvalidAddress();
        }

        beefyClient = IBeefyClient(_beefyClient);
        owner = _owner;
        maxGasPrice = _maxGasPrice;
        maxRefundAmount = _maxRefundAmount;
        refundTarget = _refundTarget;
        rewardTarget = _rewardTarget;
    }

    /* Beefy Client Proxy Functions */

    function submitInitial(
        IBeefyClient.Commitment calldata commitment,
        uint256[] calldata bitfield,
        IBeefyClient.ValidatorProof calldata proof
    ) external {
        uint256 startGas = gasleft();

        beefyClient.submitInitial(commitment, bitfield, proof);

        bytes32 commitmentHash = keccak256(_encodeCommitment(commitment));
        ticketOwner[commitmentHash] = msg.sender;

        _creditGas(startGas, commitmentHash);
    }

    function commitPrevRandao(bytes32 commitmentHash) external {
        uint256 startGas = gasleft();

        if (ticketOwner[commitmentHash] != msg.sender) {
            revert NotTicketOwner();
        }

        beefyClient.commitPrevRandao(commitmentHash);

        _creditGas(startGas, commitmentHash);
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
        uint64 previousBeefyBlock = beefyClient.latestBeefyBlock();

        bytes32 commitmentHash = keccak256(_encodeCommitment(commitment));
        if (ticketOwner[commitmentHash] != msg.sender) {
            revert NotTicketOwner();
        }

        beefyClient.submitFinal(commitment, bitfield, proofs, leaf, leafProof, leafProofOrder);

        // Calculate progress
        uint256 progress = commitment.blockNumber - previousBeefyBlock;

        uint256 previousGas = creditedGas[commitmentHash];
        delete creditedGas[commitmentHash];
        delete ticketOwner[commitmentHash];

        _refundWithProgress(startGas, previousGas, progress);
    }

    function createFinalBitfield(bytes32 commitmentHash, uint256[] calldata bitfield)
        external
        view
        returns (uint256[] memory)
    {
        return beefyClient.createFinalBitfield(commitmentHash, bitfield);
    }

    function latestBeefyBlock() external view returns (uint64) {
        return beefyClient.latestBeefyBlock();
    }

    function createInitialBitfield(uint256[] calldata bitsToSet, uint256 length)
        external
        view
        returns (uint256[] memory)
    {
        return beefyClient.createInitialBitfield(bitsToSet, length);
    }

    function randaoCommitDelay() external view returns (uint256) {
        return beefyClient.randaoCommitDelay();
    }

    function currentValidatorSet() external view returns (uint128 id, uint128 length, bytes32 root) {
        return beefyClient.currentValidatorSet();
    }

    function nextValidatorSet() external view returns (uint128 id, uint128 length, bytes32 root) {
        return beefyClient.nextValidatorSet();
    }

    /**
     * @dev Abandon a ticket. Useful if another relayer is competing for the same commitment.
     * Credited gas is forfeited when clearing a ticket.
     */
    function clearTicket(bytes32 commitmentHash) external {
        if (ticketOwner[commitmentHash] != msg.sender) {
            revert NotTicketOwner();
        }

        delete creditedGas[commitmentHash];
        delete ticketOwner[commitmentHash];
    }

    /**
     * @dev Fund the reward pool. Anyone can contribute.
     */
    function fundRewardPool() external payable {
        rewardPool += msg.value;
        emit RewardPoolFunded(msg.sender, msg.value);
    }

    /* Internal Functions */

    function _checkOwner() internal view {
        if (msg.sender != owner) {
            revert Unauthorized();
        }
    }

    function _creditGas(uint256 startGas, bytes32 commitmentHash) internal {
        uint256 gasUsed = startGas - gasleft() + 21000;
        creditedGas[commitmentHash] += gasUsed;
        emit GasCredited(msg.sender, commitmentHash, gasUsed);
    }

    /**
     * @dev Calculate and send refund + reward based on progress made.
     *
     * Refund: Scales from 0% to 100% as progress goes from 0 to refundTarget.
     * Reward: Kicks in after refundTarget, scales to 100% at rewardTarget.
     *
     * Example with refundTarget=300, rewardTarget=2400:
     * - 150 blocks progress: 50% gas refund, 0% reward
     * - 300 blocks progress: 100% gas refund, 0% reward
     * - 600 blocks progress: 100% gas refund, 14.3% reward (300/2100)
     * - 2400+ blocks progress: 100% gas refund, 100% reward
     */
    function _refundWithProgress(uint256 startGas, uint256 previousGas, uint256 progress) internal {
        uint256 currentGas = startGas - gasleft() + 21000;
        uint256 totalGasUsed = currentGas + previousGas;
        uint256 effectiveGasPrice = tx.gasprice < maxGasPrice ? tx.gasprice : maxGasPrice;
        uint256 baseRefund = totalGasUsed * effectiveGasPrice;

        // Cap base refund
        if (baseRefund > maxRefundAmount) {
            baseRefund = maxRefundAmount;
        }

        // Calculate refund ratio (0-100%)
        uint256 refundRatio = progress >= refundTarget ? 100 : (progress * 100) / refundTarget;
        uint256 refundAmount = (baseRefund * refundRatio) / 100;

        // Calculate reward ratio (0-100%, only kicks in after refundTarget)
        uint256 rewardAmount = 0;
        if (progress > refundTarget && rewardPool > 0 && rewardTarget > refundTarget) {
            uint256 extraProgress = progress - refundTarget;
            uint256 rewardWindow = rewardTarget - refundTarget;
            uint256 rewardRatio = extraProgress >= rewardWindow ? 100 : (extraProgress * 100) / rewardWindow;
            rewardAmount = (rewardPool * rewardRatio) / 100;

            // Deduct from reward pool
            if (rewardAmount > rewardPool) {
                rewardAmount = rewardPool;
            }
            rewardPool -= rewardAmount;
        }

        uint256 totalPayout = refundAmount + rewardAmount;

        if (totalPayout > 0 && address(this).balance >= totalPayout) {
            (bool success,) = payable(msg.sender).call{value: totalPayout}("");
            if (success) {
                emit SubmissionRefunded(msg.sender, progress, refundAmount, rewardAmount, totalGasUsed);
            }
        }
    }

    function _encodeCommitment(IBeefyClient.Commitment calldata commitment) internal pure returns (bytes memory) {
        return bytes.concat(
            _encodeCommitmentPayload(commitment.payload),
            ScaleCodec.encodeU32(commitment.blockNumber),
            ScaleCodec.encodeU64(commitment.validatorSetID)
        );
    }

    function _encodeCommitmentPayload(IBeefyClient.PayloadItem[] calldata items) internal pure returns (bytes memory) {
        bytes memory payload = ScaleCodec.checkedEncodeCompactU32(items.length);
        for (uint256 i = 0; i < items.length; i++) {
            payload = bytes.concat(
                payload, items[i].payloadID, ScaleCodec.checkedEncodeCompactU32(items[i].data.length), items[i].data
            );
        }
        return payload;
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

    function setRewardTarget(uint256 _rewardTarget) external {
        _checkOwner();
        rewardTarget = _rewardTarget;
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

    /* View Functions */

    function getBalance() external view returns (uint256) {
        return address(this).balance;
    }

    function getCreditedGas(bytes32 commitmentHash) external view returns (uint256) {
        return creditedGas[commitmentHash];
    }

    function getRewardPool() external view returns (uint256) {
        return rewardPool;
    }

    /**
     * @dev Calculate expected refund and reward for a given progress.
     * Useful for relayers to estimate payouts before submitting.
     */
    function estimatePayout(uint256 gasUsed, uint256 gasPrice, uint256 progress)
        external
        view
        returns (uint256 refundAmount, uint256 rewardAmount)
    {
        uint256 effectiveGasPrice = gasPrice < maxGasPrice ? gasPrice : maxGasPrice;
        uint256 baseRefund = gasUsed * effectiveGasPrice;

        if (baseRefund > maxRefundAmount) {
            baseRefund = maxRefundAmount;
        }

        uint256 refundRatio = progress >= refundTarget ? 100 : (progress * 100) / refundTarget;
        refundAmount = (baseRefund * refundRatio) / 100;

        if (progress > refundTarget && rewardPool > 0 && rewardTarget > refundTarget) {
            uint256 extraProgress = progress - refundTarget;
            uint256 rewardWindow = rewardTarget - refundTarget;
            uint256 rewardRatio = extraProgress >= rewardWindow ? 100 : (extraProgress * 100) / rewardWindow;
            rewardAmount = (rewardPool * rewardRatio) / 100;
        }
    }

    receive() external payable {
        emit FundsDeposited(msg.sender, msg.value);
    }
}
