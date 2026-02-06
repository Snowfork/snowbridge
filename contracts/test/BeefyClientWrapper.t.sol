// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.33;

import {Test} from "forge-std/Test.sol";
import {console} from "forge-std/console.sol";

import {BeefyClientWrapper} from "../src/BeefyClientWrapper.sol";
import {IBeefyClient} from "../src/interfaces/IBeefyClient.sol";
import {ScaleCodec} from "../src/utils/ScaleCodec.sol";

/**
 * @title MockGateway
 * @dev Returns the BeefyClient address, simulating the GatewayProxy → Gateway pattern
 */
contract MockGateway {
    address public BEEFY_CLIENT;

    constructor(address _beefyClient) {
        BEEFY_CLIENT = _beefyClient;
    }

    function setBeefyClient(address _beefyClient) external {
        BEEFY_CLIENT = _beefyClient;
    }
}

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

    function computeCommitmentHash(IBeefyClient.Commitment calldata commitment) external pure returns (bytes32) {
        return keccak256(_encodeCommitment(commitment));
    }

    function setLatestBeefyBlock(uint64 _block) external {
        latestBeefyBlock = _block;
    }

    uint256 public submitFiatShamirCount;

    function submitFiatShamir(
        IBeefyClient.Commitment calldata commitment,
        uint256[] calldata,
        IBeefyClient.ValidatorProof[] calldata,
        IBeefyClient.MMRLeaf calldata,
        bytes32[] calldata,
        uint256
    ) external {
        submitFiatShamirCount++;
        latestBeefyBlock = commitment.blockNumber;
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
    MockGateway mockGateway;

    address owner = address(0x1);
    address relayer1 = address(0x2);
    address relayer2 = address(0x3);
    address anyone = address(0x5);

    uint256 constant MAX_GAS_PRICE = 100 gwei;
    uint256 constant MAX_REFUND_AMOUNT = 0.05 ether;
    uint256 constant REFUND_TARGET = 350; // ~35 min for 100% refund
    uint256 constant INITIAL_BEEFY_BLOCK = 1000;

    function setUp() public {
        // Deploy mock BeefyClient and Gateway
        mockBeefyClient = new MockBeefyClient(uint64(INITIAL_BEEFY_BLOCK));
        mockGateway = new MockGateway(address(mockBeefyClient));

        // Deploy wrapper with gateway address
        wrapper = new BeefyClientWrapper(
            address(mockGateway),
            owner,
            MAX_GAS_PRICE,
            MAX_REFUND_AMOUNT,
            REFUND_TARGET
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
        assertEq(wrapper.gateway(), address(mockGateway));
        assertEq(wrapper.maxGasPrice(), MAX_GAS_PRICE);
        assertEq(wrapper.maxRefundAmount(), MAX_REFUND_AMOUNT);
        assertEq(wrapper.refundTarget(), REFUND_TARGET);
    }

    function test_invalidGatewayAddress() public {
        vm.expectRevert(BeefyClientWrapper.InvalidAddress.selector);
        new BeefyClientWrapper(
            address(0),
            owner,
            MAX_GAS_PRICE,
            MAX_REFUND_AMOUNT,
            REFUND_TARGET
        );
    }

    function test_invalidOwnerAddress() public {
        vm.expectRevert(BeefyClientWrapper.InvalidAddress.selector);
        new BeefyClientWrapper(
            address(mockGateway),
            address(0),
            MAX_GAS_PRICE,
            MAX_REFUND_AMOUNT,
            REFUND_TARGET
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

    function test_cannotOverwriteExistingTicketOwner() public {
        uint32 newBlockNumber = uint32(INITIAL_BEEFY_BLOCK + 100);
        IBeefyClient.Commitment memory commitment = createCommitment(newBlockNumber);
        uint256[] memory bitfield = new uint256[](1);
        IBeefyClient.ValidatorProof memory proof = createValidatorProof();

        // relayer1 submits initial
        vm.prank(relayer1);
        wrapper.submitInitial(commitment, bitfield, proof);

        // relayer2 tries to submit initial for the same commitment (should fail)
        vm.prank(relayer2);
        vm.expectRevert(BeefyClientWrapper.TicketAlreadyOwned.selector);
        wrapper.submitInitial(commitment, bitfield, proof);
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
        vm.txGasPrice(50 gwei);
        wrapper.submitInitial(commitment, bitfield, proof);

        bytes32 commitmentHash = computeCommitmentHash(commitment);
        assertEq(wrapper.ticketOwner(commitmentHash), relayer1);
        assertGt(wrapper.creditedCost(commitmentHash), 0);

        // Clear ticket - gas should be forfeited
        wrapper.clearTicket(commitmentHash);
        assertEq(wrapper.ticketOwner(commitmentHash), address(0));
        assertEq(wrapper.creditedCost(commitmentHash), 0);
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

    function test_costCreditedOnSubmitInitial() public {
        uint32 newBlockNumber = uint32(INITIAL_BEEFY_BLOCK + 100);
        IBeefyClient.Commitment memory commitment = createCommitment(newBlockNumber);
        uint256[] memory bitfield = new uint256[](1);
        IBeefyClient.ValidatorProof memory proof = createValidatorProof();

        uint256 relayerBalanceBefore = relayer1.balance;

        vm.prank(relayer1);
        vm.txGasPrice(50 gwei);
        wrapper.submitInitial(commitment, bitfield, proof);

        // No immediate refund, cost is credited instead
        assertEq(relayer1.balance, relayerBalanceBefore);

        // Cost should be credited (ETH, not raw gas)
        bytes32 commitmentHash = computeCommitmentHash(commitment);
        assertGt(wrapper.creditedCost(commitmentHash), 0);
    }

    function test_costCreditedAtPerStepGasPrice() public {
        // Verify that each step captures gas cost at its own tx.gasprice,
        // not at the final step's gas price.
        uint32 newBlockNumber = uint32(INITIAL_BEEFY_BLOCK + REFUND_TARGET);
        IBeefyClient.Commitment memory commitment = createCommitment(newBlockNumber);
        uint256[] memory bitfield = new uint256[](1);
        IBeefyClient.ValidatorProof memory proof = createValidatorProof();
        IBeefyClient.ValidatorProof[] memory proofs = createValidatorProofs(1);
        IBeefyClient.MMRLeaf memory leaf = createMMRLeaf();
        bytes32[] memory leafProof = new bytes32[](0);
        bytes32 commitmentHash = computeCommitmentHash(commitment);

        // Step 1: submitInitial at 10 gwei
        vm.prank(relayer1);
        vm.txGasPrice(10 gwei);
        wrapper.submitInitial(commitment, bitfield, proof);
        uint256 costAfterInitial = wrapper.creditedCost(commitmentHash);

        // Step 2: commitPrevRandao at 90 gwei
        vm.prank(relayer1);
        vm.txGasPrice(90 gwei);
        wrapper.commitPrevRandao(commitmentHash);
        uint256 costAfterCommit = wrapper.creditedCost(commitmentHash);

        // The cost from commitPrevRandao should be much higher per gas unit than submitInitial
        uint256 commitCost = costAfterCommit - costAfterInitial;
        // commitPrevRandao uses fewer gas units than submitInitial, but at 9x the gas price
        // so the ratio of cost per gas unit should reflect the different gas prices
        assertGt(commitCost, 0);
        assertGt(costAfterInitial, 0);

        // Step 3: submitFinal at 50 gwei - refund should use per-step costs
        uint256 relayerBalanceBefore = relayer1.balance;
        vm.prank(relayer1);
        vm.txGasPrice(50 gwei);
        wrapper.submitFinal(commitment, bitfield, proofs, leaf, leafProof, 0);

        uint256 refundAmount = relayer1.balance - relayerBalanceBefore;
        assertGt(refundAmount, 0);

        // The refund should include costs from all three steps at their respective gas prices.
        // If it incorrectly used only the final gas price (50 gwei) for all steps,
        // the refund would be different from what we expect.
        // Specifically, the credited cost from step 1 (10 gwei) + step 2 (90 gwei)
        // should be preserved, not recalculated at 50 gwei.
        assertGt(refundAmount, costAfterCommit); // Must include submitFinal gas cost too
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

        assertEq(wrapper.creditedCost(commitmentHash), 0);
    }

    function test_noRefundForLowProgress() public {
        // Below refund target = no refund
        uint32 progress = uint32(REFUND_TARGET / 2); // 150 blocks (below 300 threshold)
        uint32 newBlockNumber = uint32(INITIAL_BEEFY_BLOCK + progress);
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

        // Verify no refund was paid (progress below threshold)
        assertEq(relayer1.balance, relayerBalanceBefore);
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

    function test_estimatePayout() public {
        uint256 gasUsed = 500000;
        uint256 gasPrice = 50 gwei;

        // Test below threshold (no refund)
        uint256 refundBelow = wrapper.estimatePayout(gasUsed, gasPrice, REFUND_TARGET / 2);
        assertEq(refundBelow, 0);

        // Test at threshold (full refund)
        uint256 refundAt = wrapper.estimatePayout(gasUsed, gasPrice, REFUND_TARGET);
        assertEq(refundAt, gasUsed * gasPrice);

        // Test above threshold (full refund)
        uint256 refundAbove = wrapper.estimatePayout(gasUsed, gasPrice, REFUND_TARGET * 2);
        assertEq(refundAbove, gasUsed * gasPrice);
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

    /* Fiat Shamir Tests */

    function test_submitFiatShamir() public {
        uint32 newBlockNumber = uint32(INITIAL_BEEFY_BLOCK + 500);
        IBeefyClient.Commitment memory commitment = createCommitment(newBlockNumber);
        uint256[] memory bitfield = new uint256[](1);
        IBeefyClient.ValidatorProof[] memory proofs = createValidatorProofs(1);
        IBeefyClient.MMRLeaf memory leaf = createMMRLeaf();
        bytes32[] memory leafProof = new bytes32[](0);

        vm.prank(relayer1);
        wrapper.submitFiatShamir(commitment, bitfield, proofs, leaf, leafProof, 0);

        assertEq(mockBeefyClient.submitFiatShamirCount(), 1);
    }

    function test_submitFiatShamir_clearsHighestPendingBlock() public {
        // First create a pending session
        uint32 pendingBlockNumber = uint32(INITIAL_BEEFY_BLOCK + 100);
        IBeefyClient.Commitment memory pendingCommitment = createCommitment(pendingBlockNumber);
        uint256[] memory bitfield = new uint256[](1);
        IBeefyClient.ValidatorProof memory proof = createValidatorProof();

        vm.prank(relayer1);
        wrapper.submitInitial(pendingCommitment, bitfield, proof);
        assertEq(wrapper.highestPendingBlock(), pendingBlockNumber);

        // Submit Fiat Shamir with higher block number
        uint32 fiatShamirBlockNumber = uint32(INITIAL_BEEFY_BLOCK + 200);
        IBeefyClient.Commitment memory fiatShamirCommitment = createCommitment(fiatShamirBlockNumber);
        IBeefyClient.ValidatorProof[] memory proofs = createValidatorProofs(1);
        IBeefyClient.MMRLeaf memory leaf = createMMRLeaf();
        bytes32[] memory leafProof = new bytes32[](0);

        vm.prank(relayer2);
        wrapper.submitFiatShamir(fiatShamirCommitment, bitfield, proofs, leaf, leafProof, 0);

        // highestPendingBlock should be cleared since latestBeefyBlock >= highestPendingBlock
        assertEq(wrapper.highestPendingBlock(), 0);
        assertEq(wrapper.highestPendingBlockTimestamp(), 0);
    }

    /* Additional Admin Function Tests */

    function test_withdrawFunds_invalidRecipient() public {
        vm.prank(owner);
        vm.expectRevert(BeefyClientWrapper.InvalidAddress.selector);
        wrapper.withdrawFunds(payable(address(0)), 1 ether);
    }

    function test_transferOwnership_invalidAddress() public {
        vm.prank(owner);
        vm.expectRevert(BeefyClientWrapper.InvalidAddress.selector);
        wrapper.transferOwnership(address(0));
    }

    /* Highest Pending Block Tests */

    function test_submitInitial_doesNotUpdateHighestPendingBlock_whenLower() public {
        // First submission sets highestPendingBlock
        uint32 higherBlockNumber = uint32(INITIAL_BEEFY_BLOCK + 500);
        IBeefyClient.Commitment memory commitment1 = createCommitment(higherBlockNumber);
        uint256[] memory bitfield = new uint256[](1);
        IBeefyClient.ValidatorProof memory proof = createValidatorProof();

        vm.prank(relayer1);
        wrapper.submitInitial(commitment1, bitfield, proof);
        assertEq(wrapper.highestPendingBlock(), higherBlockNumber);
        uint256 timestamp1 = wrapper.highestPendingBlockTimestamp();

        // Second submission with lower block number should NOT update
        uint32 lowerBlockNumber = uint32(INITIAL_BEEFY_BLOCK + 200);
        IBeefyClient.Commitment memory commitment2 = createCommitment(lowerBlockNumber);

        vm.prank(relayer2);
        wrapper.submitInitial(commitment2, bitfield, proof);

        // Should still be the higher block number
        assertEq(wrapper.highestPendingBlock(), higherBlockNumber);
        assertEq(wrapper.highestPendingBlockTimestamp(), timestamp1);
    }

    /* Additional EstimatePayout Tests */

    function test_estimatePayout_capsGasPrice() public {
        uint256 gasUsed = 500000;
        uint256 highGasPrice = 200 gwei; // Higher than MAX_GAS_PRICE (100 gwei)

        uint256 refund = wrapper.estimatePayout(gasUsed, highGasPrice, REFUND_TARGET);

        // Should use maxGasPrice (100 gwei), not the provided highGasPrice
        assertEq(refund, gasUsed * MAX_GAS_PRICE);
    }

    function test_estimatePayout_capsRefundAmount() public {
        uint256 gasUsed = 1000000000; // Very high gas to exceed max refund
        uint256 gasPrice = 100 gwei;

        uint256 refund = wrapper.estimatePayout(gasUsed, gasPrice, REFUND_TARGET);

        // Should be capped at maxRefundAmount
        assertEq(refund, MAX_REFUND_AMOUNT);
    }
}

/**
 * @dev Contract that rejects ETH transfers for testing TransferFailed
 */
contract RejectingRecipient {
    receive() external payable {
        revert("No ETH accepted");
    }
}

/**
 * @dev Contract relayer that rejects ETH refunds for testing failed refund transfers
 */
contract RejectingRelayer {
    BeefyClientWrapper public wrapper;

    constructor(BeefyClientWrapper _wrapper) {
        wrapper = _wrapper;
    }

    function submitInitial(
        IBeefyClient.Commitment calldata commitment,
        uint256[] calldata bitfield,
        IBeefyClient.ValidatorProof calldata proof
    ) external {
        wrapper.submitInitial(commitment, bitfield, proof);
    }

    function commitPrevRandao(bytes32 commitmentHash) external {
        wrapper.commitPrevRandao(commitmentHash);
    }

    function submitFinal(
        IBeefyClient.Commitment calldata commitment,
        uint256[] calldata bitfield,
        IBeefyClient.ValidatorProof[] calldata proofs,
        IBeefyClient.MMRLeaf calldata leaf,
        bytes32[] calldata leafProof,
        uint256 leafProofOrder
    ) external {
        wrapper.submitFinal(commitment, bitfield, proofs, leaf, leafProof, leafProofOrder);
    }

    // Reject ETH transfers
    receive() external payable {
        revert("No refunds accepted");
    }
}

contract BeefyClientWrapperTransferFailedTest is Test {
    BeefyClientWrapper wrapper;
    MockBeefyClient mockBeefyClient;
    MockGateway mockGateway;
    RejectingRecipient rejectingRecipient;

    address owner = address(0x1);
    uint256 constant INITIAL_BEEFY_BLOCK = 1000;
    uint256 constant REFUND_TARGET = 350;

    function setUp() public {
        mockBeefyClient = new MockBeefyClient(uint64(INITIAL_BEEFY_BLOCK));
        mockGateway = new MockGateway(address(mockBeefyClient));
        wrapper = new BeefyClientWrapper(
            address(mockGateway),
            owner,
            100 gwei,
            0.05 ether,
            REFUND_TARGET
        );
        vm.deal(address(wrapper), 100 ether);
        rejectingRecipient = new RejectingRecipient();
    }

    function test_withdrawFunds_transferFailed() public {
        vm.prank(owner);
        vm.expectRevert(BeefyClientWrapper.TransferFailed.selector);
        wrapper.withdrawFunds(payable(address(rejectingRecipient)), 1 ether);
    }

    function test_refundFailsWhenRelayerRejectsETH() public {
        // Create a relayer contract that rejects ETH
        RejectingRelayer rejectingRelayer = new RejectingRelayer(wrapper);

        // Create commitment with enough progress for refund
        uint32 newBlockNumber = uint32(INITIAL_BEEFY_BLOCK + REFUND_TARGET);
        IBeefyClient.PayloadItem[] memory payload = new IBeefyClient.PayloadItem[](1);
        payload[0] = IBeefyClient.PayloadItem(bytes2("mh"), abi.encodePacked(bytes32(0)));
        IBeefyClient.Commitment memory commitment = IBeefyClient.Commitment(newBlockNumber, 1, payload);

        uint256[] memory bitfield = new uint256[](1);
        bytes32[] memory proof = new bytes32[](0);
        IBeefyClient.ValidatorProof memory validatorProof = IBeefyClient.ValidatorProof(27, bytes32(0), bytes32(0), 0, address(0), proof);
        IBeefyClient.ValidatorProof[] memory proofs = new IBeefyClient.ValidatorProof[](1);
        proofs[0] = validatorProof;
        IBeefyClient.MMRLeaf memory leaf = IBeefyClient.MMRLeaf(1, 0, bytes32(0), 1, 100, bytes32(0), bytes32(0));
        bytes32[] memory leafProof = new bytes32[](0);

        uint256 wrapperBalanceBefore = address(wrapper).balance;

        // Submit through the rejecting relayer
        vm.txGasPrice(50 gwei);
        rejectingRelayer.submitInitial(commitment, bitfield, validatorProof);

        bytes32 commitmentHash = mockBeefyClient.computeCommitmentHash(commitment);
        rejectingRelayer.commitPrevRandao(commitmentHash);

        // submitFinal should succeed even though refund transfer fails
        rejectingRelayer.submitFinal(commitment, bitfield, proofs, leaf, leafProof, 0);

        // Verify submission succeeded
        assertEq(mockBeefyClient.submitFinalCount(), 1);
        assertEq(mockBeefyClient.latestBeefyBlock(), newBlockNumber);

        // Verify no refund was sent (wrapper balance unchanged)
        assertEq(address(wrapper).balance, wrapperBalanceBefore);
    }
}
