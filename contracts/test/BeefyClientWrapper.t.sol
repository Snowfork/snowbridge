// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import {Test} from "forge-std/Test.sol";
import {console} from "forge-std/console.sol";

import {BeefyClientWrapper} from "../src/BeefyClientWrapper.sol";
import {BeefyClientWrapperProxy} from "../src/BeefyClientWrapperProxy.sol";
import {IBeefyClient} from "../src/interfaces/IBeefyClient.sol";
import {IUpgradable} from "../src/interfaces/IUpgradable.sol";

/**
 * @title MockBeefyClient
 * @dev A simplified mock of BeefyClient for testing the refund proxy
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
        bytes32 commitmentHash = keccak256(abi.encode(commitment));
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
        bytes32 commitmentHash = keccak256(abi.encode(commitment));
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

    function setLatestBeefyBlock(uint64 _block) external {
        latestBeefyBlock = _block;
    }
}

contract BeefyClientWrapperTest is Test {
    BeefyClientWrapper implementation;
    BeefyClientWrapperProxy proxy;
    BeefyClientWrapper wrapper;
    MockBeefyClient mockBeefyClient;

    address owner = address(0x1);
    address relayer1 = address(0x2);
    address relayer2 = address(0x3);
    address relayer3 = address(0x4);
    address nonRelayer = address(0x5);

    uint256 constant MAX_GAS_PRICE = 100 gwei;
    uint256 constant GRACE_PERIOD_BLOCKS = 10;
    uint256 constant MIN_BLOCK_INCREMENT = 100;
    uint256 constant MAX_REFUND_AMOUNT = 1 ether;
    uint256 constant INITIAL_BEEFY_BLOCK = 1000;

    function setUp() public {
        // Deploy mock BeefyClient
        mockBeefyClient = new MockBeefyClient(uint64(INITIAL_BEEFY_BLOCK));

        // Deploy implementation
        implementation = new BeefyClientWrapper();

        // Encode initialization parameters
        bytes memory initParams = abi.encode(
            address(mockBeefyClient),
            owner,
            MAX_GAS_PRICE,
            GRACE_PERIOD_BLOCKS,
            MIN_BLOCK_INCREMENT,
            MAX_REFUND_AMOUNT
        );

        // Deploy proxy
        proxy = new BeefyClientWrapperProxy(address(implementation), initParams);

        // Get interface to proxy
        wrapper = BeefyClientWrapper(payable(address(proxy)));

        // Fund the proxy with ETH for refunds
        vm.deal(address(proxy), 100 ether);

        // Add relayers
        vm.startPrank(owner);
        wrapper.addRelayer(relayer1);
        wrapper.addRelayer(relayer2);
        wrapper.addRelayer(relayer3);
        vm.stopPrank();
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

    /* Initialization Tests */

    function test_initialization() public {
        assertEq(wrapper.owner(), owner);
        assertEq(address(wrapper.beefyClient()), address(mockBeefyClient));
        assertEq(wrapper.maxGasPrice(), MAX_GAS_PRICE);
        assertEq(wrapper.gracePeriodBlocks(), GRACE_PERIOD_BLOCKS);
        assertEq(wrapper.minBlockIncrement(), MIN_BLOCK_INCREMENT);
        assertEq(wrapper.maxRefundAmount(), MAX_REFUND_AMOUNT);
    }

    function test_cannotReinitialize() public {
        bytes memory initParams = abi.encode(
            address(mockBeefyClient),
            owner,
            MAX_GAS_PRICE,
            GRACE_PERIOD_BLOCKS,
            MIN_BLOCK_INCREMENT,
            MAX_REFUND_AMOUNT
        );

        // The proxy blocks initialize() calls with Unauthorized() error
        // This prevents reinitialization attempts through the proxy
        vm.expectRevert(BeefyClientWrapperProxy.Unauthorized.selector);
        wrapper.initialize(initParams);
    }

    /* Relayer Management Tests */

    function test_addRelayer() public {
        address newRelayer = address(0x100);

        vm.prank(owner);
        wrapper.addRelayer(newRelayer);

        assertTrue(wrapper.isRelayer(newRelayer));
        assertEq(wrapper.getRelayerCount(), 4);
    }

    function test_addRelayer_onlyOwner() public {
        vm.prank(nonRelayer);
        vm.expectRevert(BeefyClientWrapper.Unauthorized.selector);
        wrapper.addRelayer(address(0x100));
    }

    function test_addRelayer_alreadyExists() public {
        vm.prank(owner);
        vm.expectRevert(BeefyClientWrapper.RelayerAlreadyExists.selector);
        wrapper.addRelayer(relayer1);
    }

    function test_removeRelayer() public {
        vm.prank(owner);
        wrapper.removeRelayer(relayer2);

        assertFalse(wrapper.isRelayer(relayer2));
        assertEq(wrapper.getRelayerCount(), 2);
    }

    function test_removeRelayer_notFound() public {
        vm.prank(owner);
        vm.expectRevert(BeefyClientWrapper.RelayerNotFound.selector);
        wrapper.removeRelayer(nonRelayer);
    }

    /* Round-Robin Tests */

    function test_getCurrentTurnRelayer() public {
        address currentRelayer = wrapper.getCurrentTurnRelayer();
        assertEq(currentRelayer, relayer1); // First relayer at index 0
    }

    function test_onlyAssignedRelayerCanSubmit() public {
        uint32 newBlockNumber = uint32(INITIAL_BEEFY_BLOCK + MIN_BLOCK_INCREMENT + 1);
        IBeefyClient.Commitment memory commitment = createCommitment(newBlockNumber);
        uint256[] memory bitfield = new uint256[](1);
        IBeefyClient.ValidatorProof memory proof = createValidatorProof();

        // relayer2 is not the assigned relayer (relayer1 is)
        vm.prank(relayer2);
        vm.expectRevert(BeefyClientWrapper.NotYourTurn.selector);
        wrapper.submitInitial(commitment, bitfield, proof);
    }

    function test_gracePeriodAllowsAnyRelayer() public {
        // Advance past grace period
        vm.roll(block.number + GRACE_PERIOD_BLOCKS + 1);

        uint32 newBlockNumber = uint32(INITIAL_BEEFY_BLOCK + MIN_BLOCK_INCREMENT + 1);
        IBeefyClient.Commitment memory commitment = createCommitment(newBlockNumber);
        uint256[] memory bitfield = new uint256[](1);
        IBeefyClient.ValidatorProof memory proof = createValidatorProof();

        // relayer2 can submit during grace period
        vm.prank(relayer2);
        wrapper.submitInitial(commitment, bitfield, proof);

        assertEq(mockBeefyClient.submitInitialCount(), 1);
    }

    function test_isGracePeriodActive() public {
        assertFalse(wrapper.isGracePeriodActive());

        vm.roll(block.number + GRACE_PERIOD_BLOCKS + 1);
        assertTrue(wrapper.isGracePeriodActive());
    }

    /* Submission Flow Tests */

    function test_fullSubmissionFlow() public {
        uint32 newBlockNumber = uint32(INITIAL_BEEFY_BLOCK + MIN_BLOCK_INCREMENT + 1);
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
        bytes32 commitmentHash = keccak256(abi.encode(commitment));
        vm.prank(relayer1);
        wrapper.commitPrevRandao(commitmentHash);
        assertEq(mockBeefyClient.commitPrevRandaoCount(), 1);

        // Step 3: submitFinal
        vm.prank(relayer1);
        wrapper.submitFinal(commitment, bitfield, proofs, leaf, leafProof, 0, new uint32[](0));
        assertEq(mockBeefyClient.submitFinalCount(), 1);

        // Verify turn advanced
        assertEq(wrapper.currentTurnIndex(), 1);
        assertEq(wrapper.getCurrentTurnRelayer(), relayer2);
    }

    function test_onlyTicketOwnerCanCommitPrevRandao() public {
        uint32 newBlockNumber = uint32(INITIAL_BEEFY_BLOCK + MIN_BLOCK_INCREMENT + 1);
        IBeefyClient.Commitment memory commitment = createCommitment(newBlockNumber);
        uint256[] memory bitfield = new uint256[](1);
        IBeefyClient.ValidatorProof memory proof = createValidatorProof();

        // relayer1 submits initial
        vm.prank(relayer1);
        wrapper.submitInitial(commitment, bitfield, proof);

        // relayer2 tries to commit (should fail)
        bytes32 commitmentHash = keccak256(abi.encode(commitment));
        vm.prank(relayer2);
        vm.expectRevert(BeefyClientWrapper.NotTicketOwner.selector);
        wrapper.commitPrevRandao(commitmentHash);
    }

    function test_onlyTicketOwnerCanSubmitFinal() public {
        uint32 newBlockNumber = uint32(INITIAL_BEEFY_BLOCK + MIN_BLOCK_INCREMENT + 1);
        IBeefyClient.Commitment memory commitment = createCommitment(newBlockNumber);
        uint256[] memory bitfield = new uint256[](1);
        IBeefyClient.ValidatorProof memory proof = createValidatorProof();
        IBeefyClient.ValidatorProof[] memory proofs = createValidatorProofs(1);
        IBeefyClient.MMRLeaf memory leaf = createMMRLeaf();
        bytes32[] memory leafProof = new bytes32[](0);

        // relayer1 submits initial and commits
        vm.startPrank(relayer1);
        wrapper.submitInitial(commitment, bitfield, proof);
        bytes32 commitmentHash = keccak256(abi.encode(commitment));
        wrapper.commitPrevRandao(commitmentHash);
        vm.stopPrank();

        // relayer2 tries to submit final (should fail)
        vm.prank(relayer2);
        vm.expectRevert(BeefyClientWrapper.NotTicketOwner.selector);
        wrapper.submitFinal(commitment, bitfield, proofs, leaf, leafProof, 0, new uint32[](0));
    }

    /* Anti-Spam Tests */

    function test_minBlockIncrementEnforced() public {
        // Try to submit with insufficient block increment
        uint32 newBlockNumber = uint32(INITIAL_BEEFY_BLOCK + MIN_BLOCK_INCREMENT - 1);
        IBeefyClient.Commitment memory commitment = createCommitment(newBlockNumber);
        uint256[] memory bitfield = new uint256[](1);
        IBeefyClient.ValidatorProof memory proof = createValidatorProof();

        vm.prank(relayer1);
        vm.expectRevert(BeefyClientWrapper.InsufficientBlockIncrement.selector);
        wrapper.submitInitial(commitment, bitfield, proof);
    }

    function test_validBlockIncrementSucceeds() public {
        uint32 newBlockNumber = uint32(INITIAL_BEEFY_BLOCK + MIN_BLOCK_INCREMENT);
        IBeefyClient.Commitment memory commitment = createCommitment(newBlockNumber);
        uint256[] memory bitfield = new uint256[](1);
        IBeefyClient.ValidatorProof memory proof = createValidatorProof();

        vm.prank(relayer1);
        wrapper.submitInitial(commitment, bitfield, proof);

        assertEq(mockBeefyClient.submitInitialCount(), 1);
    }

    /* Refund Tests */

    function test_gasCreditedOnSubmitInitial() public {
        uint32 newBlockNumber = uint32(INITIAL_BEEFY_BLOCK + MIN_BLOCK_INCREMENT + 1);
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
        bytes32 commitmentHash = keccak256(abi.encode(commitment));
        assertGt(wrapper.getCreditedGas(commitmentHash), 0);
    }

    function test_refundSentOnlyAfterSubmitFinal() public {
        uint32 newBlockNumber = uint32(INITIAL_BEEFY_BLOCK + MIN_BLOCK_INCREMENT + 1);
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

        bytes32 commitmentHash = keccak256(abi.encode(commitment));
        wrapper.commitPrevRandao(commitmentHash);
        assertEq(relayer1.balance, relayerBalanceBefore);

        wrapper.submitFinal(commitment, bitfield, proofs, leaf, leafProof, 0, new uint32[](0));
        assertGt(relayer1.balance, relayerBalanceBefore);

        vm.stopPrank();

        assertEq(wrapper.getCreditedGas(commitmentHash), 0);
    }

    function test_refundCappedAtMaxGasPrice() public {
        uint32 newBlockNumber = uint32(INITIAL_BEEFY_BLOCK + MIN_BLOCK_INCREMENT + 1);
        IBeefyClient.Commitment memory commitment = createCommitment(newBlockNumber);
        uint256[] memory bitfield = new uint256[](1);
        IBeefyClient.ValidatorProof memory proof = createValidatorProof();
        IBeefyClient.ValidatorProof[] memory proofs = createValidatorProofs(1);
        IBeefyClient.MMRLeaf memory leaf = createMMRLeaf();
        bytes32[] memory leafProof = new bytes32[](0);

        uint256 relayerBalanceBefore = relayer1.balance;
        uint256 proxyBalanceBefore = address(proxy).balance;

        // Use gas price higher than max
        vm.startPrank(relayer1);
        vm.txGasPrice(200 gwei); // Higher than MAX_GAS_PRICE (100 gwei)
        wrapper.submitInitial(commitment, bitfield, proof);
        bytes32 commitmentHash = keccak256(abi.encode(commitment));
        wrapper.commitPrevRandao(commitmentHash);
        wrapper.submitFinal(commitment, bitfield, proofs, leaf, leafProof, 0, new uint32[](0));
        vm.stopPrank();

        uint256 refundAmount = relayer1.balance - relayerBalanceBefore;
        uint256 proxySpent = proxyBalanceBefore - address(proxy).balance;

        // Refund should be based on maxGasPrice, not actual tx.gasprice
        assertEq(refundAmount, proxySpent);
    }

    function test_noRefundWhenInsufficientBalance() public {
        // Drain wrapper balance
        vm.prank(owner);
        wrapper.withdrawFunds(payable(owner), address(proxy).balance);

        uint32 newBlockNumber = uint32(INITIAL_BEEFY_BLOCK + MIN_BLOCK_INCREMENT + 1);
        IBeefyClient.Commitment memory commitment = createCommitment(newBlockNumber);
        uint256[] memory bitfield = new uint256[](1);
        IBeefyClient.ValidatorProof memory proof = createValidatorProof();
        IBeefyClient.ValidatorProof[] memory proofs = createValidatorProofs(1);
        IBeefyClient.MMRLeaf memory leaf = createMMRLeaf();
        bytes32[] memory leafProof = new bytes32[](0);

        uint256 relayerBalanceBefore = relayer1.balance;

        vm.startPrank(relayer1);
        wrapper.submitInitial(commitment, bitfield, proof);
        bytes32 commitmentHash = keccak256(abi.encode(commitment));
        wrapper.commitPrevRandao(commitmentHash);
        wrapper.submitFinal(commitment, bitfield, proofs, leaf, leafProof, 0, new uint32[](0));
        vm.stopPrank();

        assertEq(relayer1.balance, relayerBalanceBefore); // No refund
        assertEq(mockBeefyClient.submitFinalCount(), 1); // Submission succeeded
    }

    function test_refundCappedAtMaxRefundAmount() public {
        // Set a very low maxRefundAmount to test capping
        vm.prank(owner);
        wrapper.setMaxRefundAmount(0.0001 ether);

        uint32 newBlockNumber = uint32(INITIAL_BEEFY_BLOCK + MIN_BLOCK_INCREMENT + 1);
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
        bytes32 commitmentHash = keccak256(abi.encode(commitment));
        wrapper.commitPrevRandao(commitmentHash);
        wrapper.submitFinal(commitment, bitfield, proofs, leaf, leafProof, 0, new uint32[](0));
        vm.stopPrank();

        uint256 refundAmount = relayer1.balance - relayerBalanceBefore;

        // Refund should be capped at maxRefundAmount
        assertEq(refundAmount, 0.0001 ether);
    }

    function test_creditedGasForfeitedOnClearTicket() public {
        uint32 newBlockNumber = uint32(INITIAL_BEEFY_BLOCK + MIN_BLOCK_INCREMENT + 1);
        IBeefyClient.Commitment memory commitment = createCommitment(newBlockNumber);
        uint256[] memory bitfield = new uint256[](1);
        IBeefyClient.ValidatorProof memory proof = createValidatorProof();

        vm.startPrank(relayer1);
        wrapper.submitInitial(commitment, bitfield, proof);

        bytes32 commitmentHash = keccak256(abi.encode(commitment));
        assertGt(wrapper.getCreditedGas(commitmentHash), 0);

        // Clear ticket - gas should be forfeited
        wrapper.clearTicket();
        assertEq(wrapper.getCreditedGas(commitmentHash), 0);
        vm.stopPrank();
    }

    /* Admin Function Tests */

    function test_setMaxGasPrice() public {
        vm.prank(owner);
        wrapper.setMaxGasPrice(200 gwei);

        assertEq(wrapper.maxGasPrice(), 200 gwei);
    }

    function test_setGracePeriod() public {
        vm.prank(owner);
        wrapper.setGracePeriod(20);

        assertEq(wrapper.gracePeriodBlocks(), 20);
    }

    function test_setMinBlockIncrement() public {
        vm.prank(owner);
        wrapper.setMinBlockIncrement(200);

        assertEq(wrapper.minBlockIncrement(), 200);
    }

    function test_setMaxRefundAmount() public {
        vm.prank(owner);
        wrapper.setMaxRefundAmount(2 ether);

        assertEq(wrapper.maxRefundAmount(), 2 ether);
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
        vm.startPrank(nonRelayer);

        vm.expectRevert(BeefyClientWrapper.Unauthorized.selector);
        wrapper.setMaxGasPrice(1);

        vm.expectRevert(BeefyClientWrapper.Unauthorized.selector);
        wrapper.setGracePeriod(1);

        vm.expectRevert(BeefyClientWrapper.Unauthorized.selector);
        wrapper.setMinBlockIncrement(1);

        vm.expectRevert(BeefyClientWrapper.Unauthorized.selector);
        wrapper.setMaxRefundAmount(1);

        vm.expectRevert(BeefyClientWrapper.Unauthorized.selector);
        wrapper.withdrawFunds(payable(nonRelayer), 1);

        vm.expectRevert(BeefyClientWrapper.Unauthorized.selector);
        wrapper.transferOwnership(nonRelayer);

        vm.stopPrank();
    }

    /* Deposit Tests */

    function test_acceptsDeposits() public {
        uint256 balanceBefore = address(proxy).balance;

        vm.deal(address(this), 1 ether);
        (bool success,) = address(proxy).call{value: 1 ether}("");

        assertTrue(success);
        assertEq(address(proxy).balance, balanceBefore + 1 ether);
    }

    /* Non-Relayer Tests */

    function test_nonRelayerCannotSubmit() public {
        uint32 newBlockNumber = uint32(INITIAL_BEEFY_BLOCK + MIN_BLOCK_INCREMENT + 1);
        IBeefyClient.Commitment memory commitment = createCommitment(newBlockNumber);
        uint256[] memory bitfield = new uint256[](1);
        IBeefyClient.ValidatorProof memory proof = createValidatorProof();

        vm.prank(nonRelayer);
        vm.expectRevert(BeefyClientWrapper.NotARelayer.selector);
        wrapper.submitInitial(commitment, bitfield, proof);
    }

    /* View Function Tests */

    function test_getRelayers() public {
        address[] memory allRelayers = wrapper.getRelayers();
        assertEq(allRelayers.length, 3);
        assertEq(allRelayers[0], relayer1);
        assertEq(allRelayers[1], relayer2);
        assertEq(allRelayers[2], relayer3);
    }

    function test_getBalance() public {
        assertEq(wrapper.getBalance(), 100 ether);
    }

    /* Upgrade Tests */

    function test_implementation() public {
        assertEq(wrapper.implementation(), address(implementation));
    }

    function test_upgradeTo() public {
        BeefyClientWrapper newImpl = new BeefyClientWrapper();

        vm.prank(owner);
        wrapper.upgradeTo(address(newImpl), address(newImpl).codehash);

        assertEq(wrapper.implementation(), address(newImpl));
    }

    function test_upgradeTo_invalidContract() public {
        vm.prank(owner);
        vm.expectRevert(IUpgradable.InvalidContract.selector);
        wrapper.upgradeTo(address(0x123), bytes32(0));
    }

    function test_upgradeTo_invalidCodeHash() public {
        BeefyClientWrapper newImpl = new BeefyClientWrapper();

        vm.prank(owner);
        vm.expectRevert(IUpgradable.InvalidCodeHash.selector);
        wrapper.upgradeTo(address(newImpl), bytes32(uint256(0x123)));
    }

    /* One Ticket Per Relayer Tests */

    function test_cannotSubmitSecondTicketWhileActive() public {
        uint32 newBlockNumber = uint32(INITIAL_BEEFY_BLOCK + MIN_BLOCK_INCREMENT + 1);
        IBeefyClient.Commitment memory commitment1 = createCommitment(newBlockNumber);
        uint256[] memory bitfield = new uint256[](1);
        IBeefyClient.ValidatorProof memory proof = createValidatorProof();

        // relayer1 submits first ticket
        vm.prank(relayer1);
        wrapper.submitInitial(commitment1, bitfield, proof);

        // Update mock to allow another submission
        mockBeefyClient.setLatestBeefyBlock(uint64(newBlockNumber));

        // relayer1 tries to submit second ticket (should fail)
        uint32 newBlockNumber2 = uint32(newBlockNumber + MIN_BLOCK_INCREMENT + 1);
        IBeefyClient.Commitment memory commitment2 = createCommitment(newBlockNumber2);

        vm.prank(relayer1);
        vm.expectRevert(BeefyClientWrapper.TicketAlreadyActive.selector);
        wrapper.submitInitial(commitment2, bitfield, proof);
    }

    function test_clearTicketAllowsNewSubmission() public {
        uint32 newBlockNumber = uint32(INITIAL_BEEFY_BLOCK + MIN_BLOCK_INCREMENT + 1);
        IBeefyClient.Commitment memory commitment1 = createCommitment(newBlockNumber);
        uint256[] memory bitfield = new uint256[](1);
        IBeefyClient.ValidatorProof memory proof = createValidatorProof();

        // relayer1 submits first ticket
        vm.prank(relayer1);
        wrapper.submitInitial(commitment1, bitfield, proof);

        // relayer1 clears their ticket
        vm.prank(relayer1);
        wrapper.clearTicket();

        // Now relayer1 can submit again
        uint32 newBlockNumber2 = uint32(newBlockNumber + 1);
        IBeefyClient.Commitment memory commitment2 = createCommitment(newBlockNumber2);

        vm.prank(relayer1);
        wrapper.submitInitial(commitment2, bitfield, proof);

        assertEq(mockBeefyClient.submitInitialCount(), 2);
    }

    function test_clearTicketRevertsIfNoActiveTicket() public {
        vm.prank(relayer1);
        vm.expectRevert(BeefyClientWrapper.InvalidTicket.selector);
        wrapper.clearTicket();
    }

    function test_submitFinalClearsActiveTicket() public {
        uint32 newBlockNumber = uint32(INITIAL_BEEFY_BLOCK + MIN_BLOCK_INCREMENT + 1);
        IBeefyClient.Commitment memory commitment = createCommitment(newBlockNumber);
        uint256[] memory bitfield = new uint256[](1);
        IBeefyClient.ValidatorProof memory proof = createValidatorProof();
        IBeefyClient.ValidatorProof[] memory proofs = createValidatorProofs(1);
        IBeefyClient.MMRLeaf memory leaf = createMMRLeaf();
        bytes32[] memory leafProof = new bytes32[](0);

        // Complete full flow
        vm.startPrank(relayer1);
        wrapper.submitInitial(commitment, bitfield, proof);

        bytes32 commitmentHash = keccak256(abi.encode(commitment));
        wrapper.commitPrevRandao(commitmentHash);
        wrapper.submitFinal(commitment, bitfield, proofs, leaf, leafProof, 0, new uint32[](0));
        vm.stopPrank();

        // Verify activeTicket is cleared
        assertEq(wrapper.activeTicket(relayer1), bytes32(0));

        // relayer1 can now submit again (it's relayer2's turn, so advance grace period)
        vm.roll(block.number + GRACE_PERIOD_BLOCKS + 1);

        uint32 newBlockNumber2 = uint32(newBlockNumber + MIN_BLOCK_INCREMENT + 1);
        IBeefyClient.Commitment memory commitment2 = createCommitment(newBlockNumber2);

        vm.prank(relayer1);
        wrapper.submitInitial(commitment2, bitfield, proof);

        assertEq(mockBeefyClient.submitInitialCount(), 2);
    }

    function test_activeTicketTracking() public {
        uint32 newBlockNumber = uint32(INITIAL_BEEFY_BLOCK + MIN_BLOCK_INCREMENT + 1);
        IBeefyClient.Commitment memory commitment = createCommitment(newBlockNumber);
        uint256[] memory bitfield = new uint256[](1);
        IBeefyClient.ValidatorProof memory proof = createValidatorProof();

        // Before submission, activeTicket should be empty
        assertEq(wrapper.activeTicket(relayer1), bytes32(0));

        vm.prank(relayer1);
        wrapper.submitInitial(commitment, bitfield, proof);

        // After submission, activeTicket should be set
        bytes32 commitmentHash = keccak256(abi.encode(commitment));
        assertEq(wrapper.activeTicket(relayer1), commitmentHash);
    }

    function test_addTip() public {
        uint32 targetBlock = uint32(INITIAL_BEEFY_BLOCK + MIN_BLOCK_INCREMENT + 1);

        vm.deal(address(this), 1 ether);
        wrapper.addTip{value: 0.5 ether}(targetBlock);

        assertEq(wrapper.getTip(targetBlock), 0.5 ether);
    }

    function test_addTip_accumulates() public {
        uint32 targetBlock = uint32(INITIAL_BEEFY_BLOCK + MIN_BLOCK_INCREMENT + 1);

        vm.deal(address(this), 2 ether);
        wrapper.addTip{value: 0.5 ether}(targetBlock);
        wrapper.addTip{value: 0.3 ether}(targetBlock);

        assertEq(wrapper.getTip(targetBlock), 0.8 ether);
    }

    function test_claimTips_singleBlock() public {
        uint32 newBlockNumber = uint32(INITIAL_BEEFY_BLOCK + MIN_BLOCK_INCREMENT + 1);

        // Add a tip for the target block
        vm.deal(address(this), 1 ether);
        wrapper.addTip{value: 0.5 ether}(newBlockNumber);

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
        bytes32 commitmentHash = keccak256(abi.encode(commitment));
        wrapper.commitPrevRandao(commitmentHash);

        // Claim the tip for the block we're submitting
        uint32[] memory claimBlocks = new uint32[](1);
        claimBlocks[0] = newBlockNumber;
        wrapper.submitFinal(commitment, bitfield, proofs, leaf, leafProof, 0, claimBlocks);
        vm.stopPrank();

        // Relayer should have received refund + tip
        uint256 relayerBalanceAfter = relayer1.balance;
        assertGt(relayerBalanceAfter - relayerBalanceBefore, 0.5 ether);

        // Tip should be cleared
        assertEq(wrapper.getTip(newBlockNumber), 0);
    }

    function test_claimTips_multipleBlocks() public {
        uint32 newBlockNumber = uint32(INITIAL_BEEFY_BLOCK + MIN_BLOCK_INCREMENT + 1);
        uint32 tipBlock1 = uint32(INITIAL_BEEFY_BLOCK + 50);
        uint32 tipBlock2 = uint32(INITIAL_BEEFY_BLOCK + 75);

        // Add tips for blocks that will be covered by the submission
        vm.deal(address(this), 2 ether);
        wrapper.addTip{value: 0.3 ether}(tipBlock1);
        wrapper.addTip{value: 0.2 ether}(tipBlock2);

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
        bytes32 commitmentHash = keccak256(abi.encode(commitment));
        wrapper.commitPrevRandao(commitmentHash);

        // Claim tips for both blocks
        uint32[] memory claimBlocks = new uint32[](2);
        claimBlocks[0] = tipBlock1;
        claimBlocks[1] = tipBlock2;
        wrapper.submitFinal(commitment, bitfield, proofs, leaf, leafProof, 0, claimBlocks);
        vm.stopPrank();

        // Relayer should have received refund + both tips
        uint256 relayerBalanceAfter = relayer1.balance;
        assertGt(relayerBalanceAfter - relayerBalanceBefore, 0.5 ether);

        // Tips should be cleared
        assertEq(wrapper.getTip(tipBlock1), 0);
        assertEq(wrapper.getTip(tipBlock2), 0);
    }

    function test_claimTips_ignoresFutureBlocks() public {
        uint32 newBlockNumber = uint32(INITIAL_BEEFY_BLOCK + MIN_BLOCK_INCREMENT + 1);
        uint32 futureBlock = uint32(newBlockNumber + 100); // Block after submission

        // Add tip for a future block
        vm.deal(address(this), 1 ether);
        wrapper.addTip{value: 0.5 ether}(futureBlock);

        IBeefyClient.Commitment memory commitment = createCommitment(newBlockNumber);
        uint256[] memory bitfield = new uint256[](1);
        IBeefyClient.ValidatorProof memory proof = createValidatorProof();
        IBeefyClient.ValidatorProof[] memory proofs = createValidatorProofs(1);
        IBeefyClient.MMRLeaf memory leaf = createMMRLeaf();
        bytes32[] memory leafProof = new bytes32[](0);

        vm.startPrank(relayer1);
        wrapper.submitInitial(commitment, bitfield, proof);
        bytes32 commitmentHash = keccak256(abi.encode(commitment));
        wrapper.commitPrevRandao(commitmentHash);

        // Try to claim the future block tip
        uint32[] memory claimBlocks = new uint32[](1);
        claimBlocks[0] = futureBlock;
        wrapper.submitFinal(commitment, bitfield, proofs, leaf, leafProof, 0, claimBlocks);
        vm.stopPrank();

        // Tip should NOT be claimed (block is in the future)
        assertEq(wrapper.getTip(futureBlock), 0.5 ether);
    }

    function test_claimTips_emptyArray() public {
        uint32 newBlockNumber = uint32(INITIAL_BEEFY_BLOCK + MIN_BLOCK_INCREMENT + 1);

        IBeefyClient.Commitment memory commitment = createCommitment(newBlockNumber);
        uint256[] memory bitfield = new uint256[](1);
        IBeefyClient.ValidatorProof memory proof = createValidatorProof();
        IBeefyClient.ValidatorProof[] memory proofs = createValidatorProofs(1);
        IBeefyClient.MMRLeaf memory leaf = createMMRLeaf();
        bytes32[] memory leafProof = new bytes32[](0);

        vm.startPrank(relayer1);
        wrapper.submitInitial(commitment, bitfield, proof);
        bytes32 commitmentHash = keccak256(abi.encode(commitment));
        wrapper.commitPrevRandao(commitmentHash);

        // Submit with empty claim array (should work fine)
        wrapper.submitFinal(commitment, bitfield, proofs, leaf, leafProof, 0, new uint32[](0));
        vm.stopPrank();

        assertEq(mockBeefyClient.submitFinalCount(), 1);
    }
}
