// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.34;

import {Test} from "forge-std/Test.sol";
import {WETH9} from "canonical-weth/WETH9.sol";

import {IUpgradable} from "../src/interfaces/IUpgradable.sol";
import {IGatewayV1} from "../src/v1/IGateway.sol";
import {IGatewayV2} from "../src/v2/IGateway.sol";
import {Verification} from "../src/Verification.sol";
import {Gateway} from "../src/Gateway.sol";
import {
    UpgradeParams,
    ChannelID,
    ParaID,
    InboundMessage,
    Command,
    MintForeignTokenParams
} from "../src/v1/Types.sol";
import {MultiAddress, multiAddressFromBytes32} from "../src/v1/MultiAddress.sol";
import {Payload, Asset, makeRawXCM} from "../src/v2/Types.sol";
import {
    ForkTestFixtures,
    SubmitMessageFixture,
    SubmitV2MessageFixture
} from "../test/utils/ForkTestFixtures.sol";

contract ForkUpgrade202603Test is Test {
    address private constant GATEWAY_PROXY = 0x27ca963C279c93801941e1eB8799c23f407d68e7;
    address private constant BEEFY_CLIENT = 0x7cfc5C8b341991993080Af67D940B6aD19a010E1;
    address private constant GATEWAY_LOGIC_202602 = 0x36e74FCAAcb07773b144Ca19Ef2e32Fc972aC50b;

    // NOTE: Can use tenderly transaction debugger to retrieve existing library address
    address private constant VERIFICATION_ADDR_V1 = 0x4058bE4f048F55F1e9C88da09738070C0a7F1593;
    bytes4 private constant VERIFICATION_SELECTOR_V1 = 0x2db72616;

    ChannelID internal constant GOVERNANCE_CHANNEL =
        ChannelID.wrap(0x0000000000000000000000000000000000000000000000000000000000000001);
    ChannelID internal constant ASSETHUB_CHANNEL =
        ChannelID.wrap(0xc173fac324158e77fb5840738a1a541f633cbec8884c6a601c567d2b376a0539);
    bytes32 internal constant ASSETHUB_AGENT_ID =
        0x81c5ab2571199e3188135178f3c2c8e2d268be1313d029b30f534fa579b69b79;
    address internal constant assetHubAgent = 0xd803472c47a87D7B63E888DE53f03B4191B846a8;
    address internal constant DOT = 0x196C20DA81Fbc324EcdF55501e95Ce9f0bD84d14;
    bytes32 internal constant DOT_ID =
        0x4e241583d94b5d48a27a22064cd49b2ed6f5231d2d950e432f9b7c2e0ade52b2;
    address internal constant WETH = 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2;
    WETH9 public weth = WETH9(payable(WETH));

    event Transfer(address indexed from, address indexed to, uint256 value);

    function setUp() public {}

    function selectFork24677447() public {
        vm.createSelectFork(
            "https://virtual.mainnet.eu.rpc.tenderly.co/a2b5dc8d-c06a-40a9-b893-d86dc7c9ecd4",
            24_677_447
        );
    }

    function selectFork24681921() public {
        vm.createSelectFork(
            "https://virtual.mainnet.eu.rpc.tenderly.co/390efc23-bb26-460c-b641-b5275f790bd7",
            24_681_921
        );
    }

    function selectFork24683314() public {
        vm.createSelectFork(
            "https://virtual.mainnet.eu.rpc.tenderly.co/61589e0a-d204-449b-b095-64366ea949cb",
            24_683_314
        );
    }

    // This function is used to upgrade the gateway to the new implementation. It can be called
    // at the beginning of other tests in this contract to ensure that the gateway is upgraded
    // before running the test logic.
    function upgradeTo202602() public {
        (uint64 inbound, uint64 outbound) =
            IGatewayV1(GATEWAY_PROXY).channelNoncesOf(ASSETHUB_CHANNEL);

        uint256 balance = assetHubAgent.balance;

        // Mock call to Verification.verifyCommitment to bypass BEEFY verification.
        // Note that after the gateway is upgraded, the gateway will be linked to a new Verification
        // library, essentially undoing this mock.
        vm.mockCall(
            VERIFICATION_ADDR_V1,
            abi.encodeWithSelector(VERIFICATION_SELECTOR_V1),
            abi.encode(true)
        );

        // Use already deployed implementation contract
        address newLogic = GATEWAY_LOGIC_202602;

        // Prepare upgrade command
        UpgradeParams memory params = UpgradeParams({
            impl: newLogic, implCodeHash: newLogic.codehash, initParams: bytes("")
        });

        (bytes32[] memory proof1, Verification.Proof memory proof2) =
            ForkTestFixtures.makeMockProofs();
        (uint64 nonce,) = IGatewayV1(GATEWAY_PROXY).channelNoncesOf(GOVERNANCE_CHANNEL);

        vm.expectEmit();
        emit IUpgradable.Upgraded(newLogic);

        // Issue the upgrade
        IGatewayV1(GATEWAY_PROXY)
            .submitV1(
                InboundMessage(
                    GOVERNANCE_CHANNEL,
                    nonce + 1,
                    Command.Upgrade,
                    abi.encode(params),
                    100_000,
                    block.basefee,
                    0,
                    keccak256("message-id")
                ),
                proof1,
                proof2
            );

        // Governance channel inbound nonce should have incremented by 1
        (uint64 nonceAfter,) = IGatewayV1(GATEWAY_PROXY).channelNoncesOf(GOVERNANCE_CHANNEL);
        assertEq(nonceAfter, nonce + 1);

        // Asset hub channel nonces should be unchanged
        (uint64 inboundAfter, uint64 outboundAfter) =
            IGatewayV1(GATEWAY_PROXY).channelNoncesOf(ASSETHUB_CHANNEL);
        assertEq(inboundAfter, inbound);
        assertEq(outboundAfter, outbound);

        address agent = IGatewayV1(GATEWAY_PROXY).agentOf(ASSETHUB_AGENT_ID);
        assertEq(agent, assetHubAgent);

        // Asset hub agent balance should be unchanged
        uint256 balanceAfter = assetHubAgent.balance;
        assertEq(balanceAfter, balance);
    }

    function testUpgradedGateway202602StillAcceptsV2UnlockNativeEther() public {
        selectFork24677447();
        upgradeTo202602();
        SubmitV2MessageFixture memory fixture = ForkTestFixtures.makeSubmitV2MessageFixture(
            "/test/data/mainnet-gateway-submitv2-unlock-ether.json"
        );

        // Expect the gateway to emit InboundMessageDispatched event
        vm.expectEmit(true, true, true, true);
        emit IGatewayV2.InboundMessageDispatched(
            fixture.message.nonce, fixture.message.topic, true, fixture.rewardAddress
        );

        address relayer = makeAddr("relayer");
        vm.deal(relayer, 10 ether);

        vm.prank(relayer);
        IGatewayV2(address(GATEWAY_PROXY))
            .v2_submit(
                fixture.message, fixture.leafProof, fixture.headerProof, fixture.rewardAddress
            );
    }

    function testUpgradedGateway202602StillAcceptsV1UnlockNativeEther() public {
        selectFork24681921();
        upgradeTo202602();
        SubmitMessageFixture memory fixture = ForkTestFixtures.makeSubmitMessageFixture(
            "/test/data/mainnet-gateway-submitv1-unlock-ether-202603.json"
        );

        // Expect the gateway to emit InboundMessageDispatched event
        vm.expectEmit(true, true, true, true);
        emit IGatewayV1.InboundMessageDispatched(
            fixture.message.channelID, fixture.message.nonce, fixture.message.id, true
        );

        address relayer = makeAddr("relayer");
        vm.deal(relayer, 10 ether);

        vm.prank(relayer);
        IGatewayV1(address(GATEWAY_PROXY))
            .submitV1(fixture.message, fixture.leafProof, fixture.headerProof);
    }

    function testUpgradedGateway202602StillCanSendEtherWithV1SendToken() public {
        selectFork24681921();
        upgradeTo202602();
        // Create a mock user
        address user = makeAddr("user");
        uint128 amount = 1;
        ParaID paraID = ParaID.wrap(1000);

        MultiAddress memory recipientAddress32 = multiAddressFromBytes32(keccak256("recipient"));

        uint128 fee =
            uint128(IGatewayV1(address(GATEWAY_PROXY)).quoteSendTokenFee(address(0), paraID, 1));
        assertTrue(fee > 0);
        // This addresses the missing AssetsStorage.Layout migration bug, which was causing the fee to be unreasonably high.
        assertTrue(fee < 0.01 ether);

        uint256 balanceBefore = assetHubAgent.balance;

        vm.expectEmit();
        emit IGatewayV1.TokenSent(address(0), user, paraID, recipientAddress32, amount);
        vm.expectEmit(true, false, false, false);
        emit IGatewayV1.OutboundMessageAccepted(paraID.into(), 1, bytes32("0x"), hex"");
        hoax(user, amount + fee);
        IGatewayV1(address(GATEWAY_PROXY))
        .sendToken{value: amount + fee}(address(0), paraID, recipientAddress32, 1, amount);
        // After the sendToken call, the asset hub agent should have received the amount, and the user's balance should be 0.
        uint256 balanceAfter = assetHubAgent.balance;
        assertEq(balanceAfter, balanceBefore + amount);
        assertEq(user.balance, 0);
    }

    function testUpgradedGateway202602StillCanSendEtherWithV2SendMessage() public {
        selectFork24681921();
        upgradeTo202602();
        // Create a mock user
        address user = makeAddr("user");
        uint128 amount = uint128(0.1 ether);
        ParaID paraID = ParaID.wrap(1000);

        uint128 executionFee = 0.01 ether;
        uint128 relayerFee = 0.01 ether;
        uint128 fee = executionFee + relayerFee;

        uint256 balanceBefore = assetHubAgent.balance;
        uint64 nonceBefore = IGatewayV2(address(GATEWAY_PROXY)).v2_outboundNonce();

        vm.expectEmit(true, false, false, false);
        emit IGatewayV2.OutboundMessageAccepted(
            1,
            Payload({
                origin: user,
                assets: new Asset[](0),
                xcm: makeRawXCM(""),
                claimer: "",
                value: amount,
                executionFee: executionFee,
                relayerFee: relayerFee})
        );

        hoax(user, amount + fee);
        IGatewayV2(payable(address(GATEWAY_PROXY)))
        .v2_sendMessage{value: amount + fee}("", new bytes[](0), "", executionFee, relayerFee);

        // Verify asset balances
        assertEq(assetHubAgent.balance, balanceBefore + amount + fee);
        assertEq(IGatewayV2(address(GATEWAY_PROXY)).v2_outboundNonce(), nonceBefore + 1);
    }

    function testUpgradedGateway202602StillAcceptsV1MintDOT() public {
        selectFork24683314();
        upgradeTo202602();
        SubmitMessageFixture memory fixture = ForkTestFixtures.makeSubmitMessageFixture(
            "/test/data/mainnet-gateway-submitv1-mint-dot-202603.json"
        );

        // Expect the gateway to emit InboundMessageDispatched event
        vm.expectEmit(true, true, true, true);
        emit IGatewayV1.InboundMessageDispatched(
            fixture.message.channelID, fixture.message.nonce, fixture.message.id, true
        );

        address relayer = makeAddr("relayer");
        vm.deal(relayer, 10 ether);

        vm.prank(relayer);
        IGatewayV1(address(GATEWAY_PROXY))
            .submitV1(fixture.message, fixture.leafProof, fixture.headerProof);
    }

    // Send DOT can work with the upgraded Gateway
    function testUpgradedGatewayStillCanSendDOT() public {
        selectFork24683314();
        upgradeTo202602();
        address user = 0x302F0B71B8aD3CF6dD90aDb668E49b2168d652fd;
        vm.deal(user, 1 ether);

        vm.prank(address(GATEWAY_PROXY));
        uint128 amount = 100;
        MintForeignTokenParams memory params =
            MintForeignTokenParams({foreignTokenID: DOT_ID, recipient: user, amount: amount});

        vm.expectEmit(true, true, false, false);
        emit Transfer(address(0), user, amount);

        ParaID paraID = ParaID.wrap(1000);
        Gateway(address(GATEWAY_PROXY))
            .v1_handleMintForeignToken(paraID.into(), abi.encode(params));

        MultiAddress memory recipientAddress32 = multiAddressFromBytes32(keccak256("recipient"));

        uint128 fee = uint128(IGatewayV1(address(GATEWAY_PROXY)).quoteSendTokenFee(DOT, paraID, 1));
        assertTrue(fee > 0);
        assertTrue(fee < 0.01 ether);

        vm.prank(user);
        vm.expectEmit();
        emit IGatewayV1.TokenSent(DOT, user, paraID, recipientAddress32, amount);
        vm.expectEmit(true, true, true, false);
        emit IGatewayV1.OutboundMessageAccepted(
            paraID.into(),
            11_110,
            bytes32(0x688eefdb5afe6d8dfbccd6746853dd9ec90695bccaa7a37ea624f1b4d951fbdd),
            hex""
        );
        IGatewayV1(address(GATEWAY_PROXY))
        .sendToken{value: fee}(DOT, paraID, recipientAddress32, 1, amount);
    }
}
