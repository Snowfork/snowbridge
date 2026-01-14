// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import {Test} from "forge-std/Test.sol";
import {console} from "forge-std/console.sol";

import {BeefyClientWrapper} from "../src/BeefyClientWrapper.sol";
import {IBeefyClient} from "../src/interfaces/IBeefyClient.sol";
import {ScaleCodec} from "../src/utils/ScaleCodec.sol";

/**
 * @title MockBeefyClient
 * @dev A simplified mock of BeefyClient for testing the wrapper
 */
contract MockBeefyClient {
    uint64 public latestBeefyBlock;
    bytes32 public latestMMRRoot;

    // Track submissions for verification
    uint256 public submitInitialCount;
    uint256 public commitPrevRandaoCount;
    uint256 public submitFinalCount;

    mapping(bytes32 => bool) public ticketExists;

    constructor(uint64 _initialBeefyBlock) {
        latestBeefyBlock = _initialBeefyBlock;
    }

    function submitInitial(
        IBeefyClient.Commitment calldata commitment,
        uint256[] calldata,
        IBeefyClient.ValidatorProof calldata
    ) external {
        // Just track that it was called and create a ticket
        submitInitialCount++;
        bytes32 commitmentHash = keccak256(_encodeCommitment(commitment));
        ticketExists[commitmentHash] = true;
    }

    function commitPrevRandao(bytes32 commitmentHash) external {
        require(ticketExists[commitmentHash], "No ticket");
        commitPrevRandaoCount++;
    }

    function submitFinal(
        IBeefyClient.Commitment calldata commitment,
        uint256[] calldata,
        IBeefyClient.ValidatorProof[] calldata,
        IBeefyClient.MMRLeaf calldata,
        bytes32[] calldata,
        uint256
    ) external {
        bytes32 commitmentHash = keccak256(_encodeCommitment(commitment));
        require(ticketExists[commitmentHash], "No ticket");
        submitFinalCount++;
        delete ticketExists[commitmentHash];

        // Update latest beefy block
        latestBeefyBlock = commitment.blockNumber;
    }

    function createFinalBitfield(bytes32, uint256[] calldata bitfield)
        external
        pure
        returns (uint256[] memory)
    {
        return bitfield;
    }

    function createInitialBitfield(uint256[] calldata, uint256) external pure returns (uint256[] memory) {
        return new uint256[](1);
    }

    function randaoCommitDelay() external pure returns (uint256) {
        return 4;
    }

    function currentValidatorSet() external pure returns (uint128 id, uint128 length, bytes32 root) {
        return (1, 100, bytes32(0));
    }

    function nextValidatorSet() external pure returns (uint128 id, uint128 length, bytes32 root) {
        return (2, 100, bytes32(0));
    }

    function setLatestBeefyBlock(uint64 _block) external {
        latestBeefyBlock = _block;
    }

    function _encodeCommitment(IBeefyClient.Commitment calldata commitment)
        internal
        pure
        returns (bytes memory)
    {
        return bytes.concat(
            _encodeCommitmentPayload(commitment.payload),
            ScaleCodec.encodeU32(commitment.blockNumber),
            ScaleCodec.encodeU64(commitment.validatorSetID)
        );
    }

    function _encodeCommitmentPayload(IBeefyClient.PayloadItem[] calldata items)
        internal
        pure
        returns (bytes memory)
    {
        bytes memory payload = ScaleCodec.checkedEncodeCompactU32(items.length);
        for (uint256 i = 0; i < items.length; i++) {
            payload = bytes.concat(
                payload,
                items[i].payloadID,
                ScaleCodec.checkedEncodeCompactU32(items[i].data.length),
                items[i].data
            );
        }
        return payload;
    }
}

contract BeefyClientWrapperTest is Test {
    BeefyClientWrapper wrapper;
    MockBeefyClient mockBeefyClient;

    address owner = address(0x1);
    address relayer1 = address(0x2);
    address relayer2 = address(0x3);
    address anyone = address(0x5);

    uint256 constant MAX_GAS_PRICE = 100 gwei;
    uint256 constant MAX_REFUND_AMOUNT = 1 ether;
    uint256 constant REFUND_TARGET = 300; // 300 blocks for 100% refund
    uint256 constant REWARD_TARGET = 2400; // 2400 blocks for 100% reward
    uint256 constant INITIAL_BEEFY_BLOCK = 1000;

    function setUp() public {
        // Deploy mock BeefyClient
        mockBeefyClient = new MockBeefyClient(uint64(INITIAL_BEEFY_BLOCK));

        // Deploy wrapper directly (no proxy)
        wrapper = new BeefyClientWrapper(
            address(mockBeefyClient),
            owner,
            MAX_GAS_PRICE,
            MAX_REFUND_AMOUNT,
            REFUND_TARGET,
            REWARD_TARGET
        );

        // Fund the wrapper with ETH for refunds
        vm.deal(address(wrapper), 100 ether);
    }

    /* Helper Functions */

    function createCommitment(uint32 blockNumber) internal pure returns (IBeefyClient.Commitment memory) {
        IBeefyClient.PayloadItem[] memory payload = new IBeefyClient.PayloadItem[](1);
        payload[0] = IBeefyClient.PayloadItem(bytes2("mh"), abi.encodePacked(bytes32(0)));
        return IBeefyClient.Commitment(blockNumber, 1, payload);
    }

    function createValidatorProof() internal pure returns (IBeefyClient.ValidatorProof memory) {
        bytes32[] memory proof = new bytes32[](0);
        return IBeefyClient.ValidatorProof(27, bytes32(0), bytes32(0), 0, address(0), proof);
    }

    function createValidatorProofs(uint256 count) internal pure returns (IBeefyClient.ValidatorProof[] memory) {
        IBeefyClient.ValidatorProof[] memory proofs = new IBeefyClient.ValidatorProof[](count);
        for (uint256 i = 0; i < count; i++) {
            bytes32[] memory proof = new bytes32[](0);
            proofs[i] = IBeefyClient.ValidatorProof(27, bytes32(0), bytes32(0), i, address(0), proof);
        }
        return proofs;
    }

    function createMMRLeaf() internal pure returns (IBeefyClient.MMRLeaf memory) {
        return IBeefyClient.MMRLeaf(1, 0, bytes32(0), 1, 100, bytes32(0), bytes32(0));
    }

    function computeCommitmentHash(IBeefyClient.Commitment memory commitment) internal pure returns (bytes32) {
        bytes memory payload = ScaleCodec.checkedEncodeCompactU32(commitment.payload.length);
        for (uint256 i = 0; i < commitment.payload.length; i++) {
            payload = bytes.concat(
                payload,
                commitment.payload[i].payloadID,
                ScaleCodec.checkedEncodeCompactU32(commitment.payload[i].data.length),
                commitment.payload[i].data
            );
        }
        return keccak256(
            bytes.concat(
                payload,
                ScaleCodec.encodeU32(commitment.blockNumber),
                ScaleCodec.encodeU64(commitment.validatorSetID)
            )
        );
    }

    /* Initialization Tests */

    function test_initialization() public {
        assertEq(wrapper.owner(), owner);
        assertEq(address(wrapper.beefyClient()), address(mockBeefyClient));
        assertEq(wrapper.maxGasPrice(), MAX_GAS_PRICE);
        assertEq(wrapper.maxRefundAmount(), MAX_REFUND_AMOUNT);
        assertEq(wrapper.refundTarget(), REFUND_TARGET);
        assertEq(wrapper.rewardTarget(), REWARD_TARGET);
    }

    function test_invalidBeefyClientAddress() public {
        vm.expectRevert(BeefyClientWrapper.InvalidAddress.selector);
        new BeefyClientWrapper(
            address(0),
            owner,
            MAX_GAS_PRICE,
            MAX_REFUND_AMOUNT,
            REFUND_TARGET,
            REWARD_TARGET
        );
    }

    function test_invalidOwnerAddress() public {
        vm.expectRevert(BeefyClientWrapper.InvalidAddress.selector);
        new BeefyClientWrapper(
            address(mockBeefyClient),
            address(0),
            MAX_GAS_PRICE,
            MAX_REFUND_AMOUNT,
            REFUND_TARGET,
            REWARD_TARGET
        );
    }

    /* Submission Flow Tests */

    function test_fullSubmissionFlow() public {
        uint32 newBlockNumber = uint32(INITIAL_BEEFY_BLOCK + 100);
        IBeefyClient.Commitment memory commitment = createCommitment(newBlockNumber);
        uint256[] memory bitfield = new uint256[](1);
        IBeefyClient.ValidatorProof memory proof = createValidatorProof();
        IBeefyClient.ValidatorProof[] memory proofs = createValidatorProofs(1);
        IBeefyClient.MMRLeaf memory leaf = createMMRLeaf();
        bytes32[] memory leafProof = new bytes32[](0);

        // Step 1: submitInitial
        vm.prank(relayer1);
        wrapper.submitInitial(commitment, bitfield, proof);
        assertEq(mockBeefyClient.submitInitialCount(), 1);

        // Step 2: commitPrevRandao
        bytes32 commitmentHash = computeCommitmentHash(commitment);
        vm.prank(relayer1);
        wrapper.commitPrevRandao(commitmentHash);
        assertEq(mockBeefyClient.commitPrevRandaoCount(), 1);

        // Step 3: submitFinal
        vm.prank(relayer1);
        wrapper.submitFinal(commitment, bitfield, proofs, leaf, leafProof, 0);
        assertEq(mockBeefyClient.submitFinalCount(), 1);
    }

    function test_anyoneCanSubmit() public {
        // Anyone can submit - no whitelist
        uint32 newBlockNumber = uint32(INITIAL_BEEFY_BLOCK + 100);
        IBeefyClient.Commitment memory commitment = createCommitment(newBlockNumber);
        uint256[] memory bitfield = new uint256[](1);
        IBeefyClient.ValidatorProof memory proof = createValidatorProof();

        // Random address can submit
        vm.prank(anyone);
        wrapper.submitInitial(commitment, bitfield, proof);

        assertEq(mockBeefyClient.submitInitialCount(), 1);
    }

    function test_onlyTicketOwnerCanCommitPrevRandao() public {
        uint32 newBlockNumber = uint32(INITIAL_BEEFY_BLOCK + 100);
        IBeefyClient.Commitment memory commitment = createCommitment(newBlockNumber);
        uint256[] memory bitfield = new uint256[](1);
        IBeefyClient.ValidatorProof memory proof = createValidatorProof();

        // relayer1 submits initial
        vm.prank(relayer1);
        wrapper.submitInitial(commitment, bitfield, proof);

        // relayer2 tries to commit (should fail)
        bytes32 commitmentHash = computeCommitmentHash(commitment);
        vm.prank(relayer2);
        vm.expectRevert(BeefyClientWrapper.NotTicketOwner.selector);
        wrapper.commitPrevRandao(commitmentHash);
    }

    function test_onlyTicketOwnerCanSubmitFinal() public {
        uint32 newBlockNumber = uint32(INITIAL_BEEFY_BLOCK + 100);
        IBeefyClient.Commitment memory commitment = createCommitment(newBlockNumber);
        uint256[] memory bitfield = new uint256[](1);
        IBeefyClient.ValidatorProof memory proof = createValidatorProof();
        IBeefyClient.ValidatorProof[] memory proofs = createValidatorProofs(1);
        IBeefyClient.MMRLeaf memory leaf = createMMRLeaf();
        bytes32[] memory leafProof = new bytes32[](0);

        // relayer1 submits initial and commits
        vm.startPrank(relayer1);
        wrapper.submitInitial(commitment, bitfield, proof);
        bytes32 commitmentHash = computeCommitmentHash(commitment);
        wrapper.commitPrevRandao(commitmentHash);
        vm.stopPrank();

        // relayer2 tries to submit final (should fail)
        vm.prank(relayer2);
        vm.expectRevert(BeefyClientWrapper.NotTicketOwner.selector);
        wrapper.submitFinal(commitment, bitfield, proofs, leaf, leafProof, 0);
    }

    function test_clearTicket() public {
        uint32 newBlockNumber = uint32(INITIAL_BEEFY_BLOCK + 100);
        IBeefyClient.Commitment memory commitment = createCommitment(newBlockNumber);
        uint256[] memory bitfield = new uint256[](1);
        IBeefyClient.ValidatorProof memory proof = createValidatorProof();

        vm.startPrank(relayer1);
        wrapper.submitInitial(commitment, bitfield, proof);

        bytes32 commitmentHash = computeCommitmentHash(commitment);
        assertEq(wrapper.ticketOwner(commitmentHash), relayer1);
        assertGt(wrapper.getCreditedGas(commitmentHash), 0);

        // Clear ticket - gas should be forfeited
        wrapper.clearTicket(commitmentHash);
        assertEq(wrapper.ticketOwner(commitmentHash), address(0));
        assertEq(wrapper.getCreditedGas(commitmentHash), 0);
        vm.stopPrank();
    }

    function test_clearTicket_notOwner() public {
        uint32 newBlockNumber = uint32(INITIAL_BEEFY_BLOCK + 100);
        IBeefyClient.Commitment memory commitment = createCommitment(newBlockNumber);
        uint256[] memory bitfield = new uint256[](1);
        IBeefyClient.ValidatorProof memory proof = createValidatorProof();

        vm.prank(relayer1);
        wrapper.submitInitial(commitment, bitfield, proof);

        bytes32 commitmentHash = computeCommitmentHash(commitment);
        vm.prank(relayer2);
        vm.expectRevert(BeefyClientWrapper.NotTicketOwner.selector);
        wrapper.clearTicket(commitmentHash);
    }

    /* Progress-Based Refund Tests */

    function test_gasCreditedOnSubmitInitial() public {
        uint32 newBlockNumber = uint32(INITIAL_BEEFY_BLOCK + 100);
        IBeefyClient.Commitment memory commitment = createCommitment(newBlockNumber);
        uint256[] memory bitfield = new uint256[](1);
        IBeefyClient.ValidatorProof memory proof = createValidatorProof();

        uint256 relayerBalanceBefore = relayer1.balance;

        vm.prank(relayer1);
        vm.txGasPrice(50 gwei);
        wrapper.submitInitial(commitment, bitfield, proof);

        // No immediate refund, gas is credited instead
        assertEq(relayer1.balance, relayerBalanceBefore);

        // Gas should be credited
        bytes32 commitmentHash = computeCommitmentHash(commitment);
        assertGt(wrapper.getCreditedGas(commitmentHash), 0);
    }

    function test_refundSentOnlyAfterSubmitFinal() public {
        uint32 newBlockNumber = uint32(INITIAL_BEEFY_BLOCK + REFUND_TARGET); // 100% refund
        IBeefyClient.Commitment memory commitment = createCommitment(newBlockNumber);
        uint256[] memory bitfield = new uint256[](1);
        IBeefyClient.ValidatorProof memory proof = createValidatorProof();
        IBeefyClient.ValidatorProof[] memory proofs = createValidatorProofs(1);
        IBeefyClient.MMRLeaf memory leaf = createMMRLeaf();
        bytes32[] memory leafProof = new bytes32[](0);

        uint256 relayerBalanceBefore = relayer1.balance;

        vm.startPrank(relayer1);
        vm.txGasPrice(50 gwei);

        wrapper.submitInitial(commitment, bitfield, proof);
        assertEq(relayer1.balance, relayerBalanceBefore);

        bytes32 commitmentHash = computeCommitmentHash(commitment);
        wrapper.commitPrevRandao(commitmentHash);
        assertEq(relayer1.balance, relayerBalanceBefore);

        wrapper.submitFinal(commitment, bitfield, proofs, leaf, leafProof, 0);
        assertGt(relayer1.balance, relayerBalanceBefore);

        vm.stopPrank();

        assertEq(wrapper.getCreditedGas(commitmentHash), 0);
    }

    function test_partialRefundForLowProgress() public {
        // 50% of refund target = 50% refund
        uint32 progress = uint32(REFUND_TARGET / 2); // 150 blocks
        uint32 newBlockNumber = uint32(INITIAL_BEEFY_BLOCK + progress);
        IBeefyClient.Commitment memory commitment = createCommitment(newBlockNumber);
        uint256[] memory bitfield = new uint256[](1);
        IBeefyClient.ValidatorProof memory proof = createValidatorProof();
        IBeefyClient.ValidatorProof[] memory proofs = createValidatorProofs(1);
        IBeefyClient.MMRLeaf memory leaf = createMMRLeaf();
        bytes32[] memory leafProof = new bytes32[](0);

        uint256 relayerBalanceBefore = relayer1.balance;
        uint256 wrapperBalanceBefore = address(wrapper).balance;

        vm.startPrank(relayer1);
        vm.txGasPrice(50 gwei);
        wrapper.submitInitial(commitment, bitfield, proof);
        bytes32 commitmentHash = computeCommitmentHash(commitment);
        wrapper.commitPrevRandao(commitmentHash);
        wrapper.submitFinal(commitment, bitfield, proofs, leaf, leafProof, 0);
        vm.stopPrank();

        uint256 refundAmount = relayer1.balance - relayerBalanceBefore;
        uint256 wrapperSpent = wrapperBalanceBefore - address(wrapper).balance;

        // Verify refund was paid
        assertGt(refundAmount, 0);
        assertEq(refundAmount, wrapperSpent);
    }

    function test_fullRefundAt100PercentProgress() public {
        // 100% of refund target = 100% refund
        uint32 newBlockNumber = uint32(INITIAL_BEEFY_BLOCK + REFUND_TARGET);
        IBeefyClient.Commitment memory commitment = createCommitment(newBlockNumber);
        uint256[] memory bitfield = new uint256[](1);
        IBeefyClient.ValidatorProof memory proof = createValidatorProof();
        IBeefyClient.ValidatorProof[] memory proofs = createValidatorProofs(1);
        IBeefyClient.MMRLeaf memory leaf = createMMRLeaf();
        bytes32[] memory leafProof = new bytes32[](0);

        uint256 relayerBalanceBefore = relayer1.balance;

        vm.startPrank(relayer1);
        vm.txGasPrice(50 gwei);
        wrapper.submitInitial(commitment, bitfield, proof);
        bytes32 commitmentHash = computeCommitmentHash(commitment);
        wrapper.commitPrevRandao(commitmentHash);
        wrapper.submitFinal(commitment, bitfield, proofs, leaf, leafProof, 0);
        vm.stopPrank();

        uint256 refundAmount = relayer1.balance - relayerBalanceBefore;
        assertGt(refundAmount, 0);
    }

    function test_refundCappedAtMaxGasPrice() public {
        uint32 newBlockNumber = uint32(INITIAL_BEEFY_BLOCK + REFUND_TARGET);
        IBeefyClient.Commitment memory commitment = createCommitment(newBlockNumber);
        uint256[] memory bitfield = new uint256[](1);
        IBeefyClient.ValidatorProof memory proof = createValidatorProof();
        IBeefyClient.ValidatorProof[] memory proofs = createValidatorProofs(1);
        IBeefyClient.MMRLeaf memory leaf = createMMRLeaf();
        bytes32[] memory leafProof = new bytes32[](0);

        uint256 relayerBalanceBefore = relayer1.balance;
        uint256 wrapperBalanceBefore = address(wrapper).balance;

        // Use gas price higher than max
        vm.startPrank(relayer1);
        vm.txGasPrice(200 gwei); // Higher than MAX_GAS_PRICE (100 gwei)
        wrapper.submitInitial(commitment, bitfield, proof);
        bytes32 commitmentHash = computeCommitmentHash(commitment);
        wrapper.commitPrevRandao(commitmentHash);
        wrapper.submitFinal(commitment, bitfield, proofs, leaf, leafProof, 0);
        vm.stopPrank();

        uint256 refundAmount = relayer1.balance - relayerBalanceBefore;
        uint256 wrapperSpent = wrapperBalanceBefore - address(wrapper).balance;

        // Refund should be based on maxGasPrice, not actual tx.gasprice
        assertEq(refundAmount, wrapperSpent);
    }

    function test_refundCappedAtMaxRefundAmount() public {
        // Set a very low maxRefundAmount to test capping
        vm.prank(owner);
        wrapper.setMaxRefundAmount(0.0001 ether);

        uint32 newBlockNumber = uint32(INITIAL_BEEFY_BLOCK + REFUND_TARGET);
        IBeefyClient.Commitment memory commitment = createCommitment(newBlockNumber);
        uint256[] memory bitfield = new uint256[](1);
        IBeefyClient.ValidatorProof memory proof = createValidatorProof();
        IBeefyClient.ValidatorProof[] memory proofs = createValidatorProofs(1);
        IBeefyClient.MMRLeaf memory leaf = createMMRLeaf();
        bytes32[] memory leafProof = new bytes32[](0);

        uint256 relayerBalanceBefore = relayer1.balance;

        vm.startPrank(relayer1);
        vm.txGasPrice(100 gwei);
        wrapper.submitInitial(commitment, bitfield, proof);
        bytes32 commitmentHash = computeCommitmentHash(commitment);
        wrapper.commitPrevRandao(commitmentHash);
        wrapper.submitFinal(commitment, bitfield, proofs, leaf, leafProof, 0);
        vm.stopPrank();

        uint256 refundAmount = relayer1.balance - relayerBalanceBefore;

        // Refund should be capped at maxRefundAmount
        assertEq(refundAmount, 0.0001 ether);
    }

    function test_noRefundWhenInsufficientBalance() public {
        // Drain wrapper balance
        vm.prank(owner);
        wrapper.withdrawFunds(payable(owner), address(wrapper).balance);

        uint32 newBlockNumber = uint32(INITIAL_BEEFY_BLOCK + REFUND_TARGET);
        IBeefyClient.Commitment memory commitment = createCommitment(newBlockNumber);
        uint256[] memory bitfield = new uint256[](1);
        IBeefyClient.ValidatorProof memory proof = createValidatorProof();
        IBeefyClient.ValidatorProof[] memory proofs = createValidatorProofs(1);
        IBeefyClient.MMRLeaf memory leaf = createMMRLeaf();
        bytes32[] memory leafProof = new bytes32[](0);

        uint256 relayerBalanceBefore = relayer1.balance;

        vm.startPrank(relayer1);
        wrapper.submitInitial(commitment, bitfield, proof);
        bytes32 commitmentHash = computeCommitmentHash(commitment);
        wrapper.commitPrevRandao(commitmentHash);
        wrapper.submitFinal(commitment, bitfield, proofs, leaf, leafProof, 0);
        vm.stopPrank();

        assertEq(relayer1.balance, relayerBalanceBefore); // No refund
        assertEq(mockBeefyClient.submitFinalCount(), 1); // Submission succeeded
    }

    /* Reward Pool Tests */

    function test_fundRewardPool() public {
        vm.deal(address(this), 10 ether);

        wrapper.fundRewardPool{value: 5 ether}();

        assertEq(wrapper.getRewardPool(), 5 ether);
    }

    function test_rewardPaidForHighProgress() public {
        // Fund the reward pool
        vm.deal(address(this), 10 ether);
        wrapper.fundRewardPool{value: 5 ether}();

        // Progress beyond refundTarget triggers rewards
        uint32 progress = uint32(REWARD_TARGET); // Max progress = max reward
        uint32 newBlockNumber = uint32(INITIAL_BEEFY_BLOCK + progress);
        IBeefyClient.Commitment memory commitment = createCommitment(newBlockNumber);
        uint256[] memory bitfield = new uint256[](1);
        IBeefyClient.ValidatorProof memory proof = createValidatorProof();
        IBeefyClient.ValidatorProof[] memory proofs = createValidatorProofs(1);
        IBeefyClient.MMRLeaf memory leaf = createMMRLeaf();
        bytes32[] memory leafProof = new bytes32[](0);

        uint256 relayerBalanceBefore = relayer1.balance;
        uint256 rewardPoolBefore = wrapper.getRewardPool();

        vm.startPrank(relayer1);
        vm.txGasPrice(50 gwei);
        wrapper.submitInitial(commitment, bitfield, proof);
        bytes32 commitmentHash = computeCommitmentHash(commitment);
        wrapper.commitPrevRandao(commitmentHash);
        wrapper.submitFinal(commitment, bitfield, proofs, leaf, leafProof, 0);
        vm.stopPrank();

        uint256 relayerPayout = relayer1.balance - relayerBalanceBefore;
        uint256 rewardPoolAfter = wrapper.getRewardPool();

        // Relayer should receive refund + reward
        assertGt(relayerPayout, 0);
        // Reward pool should be depleted
        assertLt(rewardPoolAfter, rewardPoolBefore);
    }

    function test_noRewardForLowProgress() public {
        // Fund the reward pool
        vm.deal(address(this), 10 ether);
        wrapper.fundRewardPool{value: 5 ether}();

        // Progress below refundTarget = no reward (only partial refund)
        uint32 progress = uint32(REFUND_TARGET / 2);
        uint32 newBlockNumber = uint32(INITIAL_BEEFY_BLOCK + progress);
        IBeefyClient.Commitment memory commitment = createCommitment(newBlockNumber);
        uint256[] memory bitfield = new uint256[](1);
        IBeefyClient.ValidatorProof memory proof = createValidatorProof();
        IBeefyClient.ValidatorProof[] memory proofs = createValidatorProofs(1);
        IBeefyClient.MMRLeaf memory leaf = createMMRLeaf();
        bytes32[] memory leafProof = new bytes32[](0);

        uint256 rewardPoolBefore = wrapper.getRewardPool();

        vm.startPrank(relayer1);
        vm.txGasPrice(50 gwei);
        wrapper.submitInitial(commitment, bitfield, proof);
        bytes32 commitmentHash = computeCommitmentHash(commitment);
        wrapper.commitPrevRandao(commitmentHash);
        wrapper.submitFinal(commitment, bitfield, proofs, leaf, leafProof, 0);
        vm.stopPrank();

        // Reward pool should be unchanged (no reward paid)
        assertEq(wrapper.getRewardPool(), rewardPoolBefore);
    }

    function test_estimatePayout() public {
        uint256 gasUsed = 500000;
        uint256 gasPrice = 50 gwei;

        // Test 50% progress (50% refund, 0% reward)
        (uint256 refund50, uint256 reward50) = wrapper.estimatePayout(gasUsed, gasPrice, REFUND_TARGET / 2);
        assertEq(refund50, (gasUsed * gasPrice * 50) / 100);
        assertEq(reward50, 0);

        // Test 100% refund progress (100% refund, 0% reward)
        (uint256 refund100, uint256 reward100) = wrapper.estimatePayout(gasUsed, gasPrice, REFUND_TARGET);
        assertEq(refund100, gasUsed * gasPrice);
        assertEq(reward100, 0);
    }

    function test_estimatePayout_withRewardPool() public {
        // Fund the reward pool
        vm.deal(address(this), 10 ether);
        wrapper.fundRewardPool{value: 5 ether}();

        uint256 gasUsed = 500000;
        uint256 gasPrice = 50 gwei;

        // Test max progress (100% refund, 100% reward)
        (uint256 refundMax, uint256 rewardMax) = wrapper.estimatePayout(gasUsed, gasPrice, REWARD_TARGET);
        assertEq(refundMax, gasUsed * gasPrice);
        assertEq(rewardMax, 5 ether); // Full reward pool
    }

    /* Admin Function Tests */

    function test_setMaxGasPrice() public {
        vm.prank(owner);
        wrapper.setMaxGasPrice(200 gwei);

        assertEq(wrapper.maxGasPrice(), 200 gwei);
    }

    function test_setMaxRefundAmount() public {
        vm.prank(owner);
        wrapper.setMaxRefundAmount(2 ether);

        assertEq(wrapper.maxRefundAmount(), 2 ether);
    }

    function test_setRefundTarget() public {
        vm.prank(owner);
        wrapper.setRefundTarget(600);

        assertEq(wrapper.refundTarget(), 600);
    }

    function test_setRewardTarget() public {
        vm.prank(owner);
        wrapper.setRewardTarget(4800);

        assertEq(wrapper.rewardTarget(), 4800);
    }

    function test_withdrawFunds() public {
        uint256 ownerBalanceBefore = owner.balance;
        uint256 withdrawAmount = 10 ether;

        vm.prank(owner);
        wrapper.withdrawFunds(payable(owner), withdrawAmount);

        assertEq(owner.balance, ownerBalanceBefore + withdrawAmount);
    }

    function test_transferOwnership() public {
        address newOwner = address(0x999);

        vm.prank(owner);
        wrapper.transferOwnership(newOwner);

        assertEq(wrapper.owner(), newOwner);
    }

    function test_adminFunctions_onlyOwner() public {
        vm.startPrank(anyone);

        vm.expectRevert(BeefyClientWrapper.Unauthorized.selector);
        wrapper.setMaxGasPrice(1);

        vm.expectRevert(BeefyClientWrapper.Unauthorized.selector);
        wrapper.setMaxRefundAmount(1);

        vm.expectRevert(BeefyClientWrapper.Unauthorized.selector);
        wrapper.setRefundTarget(1);

        vm.expectRevert(BeefyClientWrapper.Unauthorized.selector);
        wrapper.setRewardTarget(1);

        vm.expectRevert(BeefyClientWrapper.Unauthorized.selector);
        wrapper.withdrawFunds(payable(anyone), 1);

        vm.expectRevert(BeefyClientWrapper.Unauthorized.selector);
        wrapper.transferOwnership(anyone);

        vm.stopPrank();
    }

    /* Deposit Tests */

    function test_acceptsDeposits() public {
        uint256 balanceBefore = address(wrapper).balance;

        vm.deal(address(this), 1 ether);
        (bool success,) = address(wrapper).call{value: 1 ether}("");

        assertTrue(success);
        assertEq(address(wrapper).balance, balanceBefore + 1 ether);
    }

    /* View Function Tests */

    function test_getBalance() public {
        assertEq(wrapper.getBalance(), 100 ether);
    }

    function test_latestBeefyBlock() public {
        assertEq(wrapper.latestBeefyBlock(), INITIAL_BEEFY_BLOCK);
    }

    /* Proxy View Function Tests */

    function test_createFinalBitfield() public {
        bytes32 commitmentHash = bytes32(uint256(1));
        uint256[] memory bitfield = new uint256[](1);
        bitfield[0] = 123;

        uint256[] memory result = wrapper.createFinalBitfield(commitmentHash, bitfield);
        assertEq(result[0], 123);
    }

    function test_createInitialBitfield() public {
        uint256[] memory bitsToSet = new uint256[](1);
        bitsToSet[0] = 5;

        uint256[] memory result = wrapper.createInitialBitfield(bitsToSet, 100);
        assertEq(result.length, 1);
    }

    function test_randaoCommitDelay() public {
        assertEq(wrapper.randaoCommitDelay(), 4);
    }

    function test_currentValidatorSet() public {
        (uint128 id, uint128 length, bytes32 root) = wrapper.currentValidatorSet();
        assertEq(id, 1);
        assertEq(length, 100);
        assertEq(root, bytes32(0));
    }

    function test_nextValidatorSet() public {
        (uint128 id, uint128 length, bytes32 root) = wrapper.nextValidatorSet();
        assertEq(id, 2);
        assertEq(length, 100);
        assertEq(root, bytes32(0));
    }
}
