// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.34;

import {Test} from "forge-std/Test.sol";
import {WETH9} from "canonical-weth/WETH9.sol";

import {AgentExecutor} from "../src/AgentExecutor.sol";
import {Gateway} from "../src/Gateway.sol";
import {IUpgradable} from "../src/interfaces/IUpgradable.sol";
import {Gateway202610} from "../src/upgrade/Gateway202610.sol";
import {IGatewayV1} from "../src/v1/IGateway.sol";
import {
    ChannelID,
    Command,
    InboundMessage,
    MintForeignTokenParams,
    ParaID,
    UpgradeParams
} from "../src/v1/Types.sol";
import {MultiAddress, multiAddressFromBytes32} from "../src/v1/MultiAddress.sol";
import {IGatewayV2} from "../src/v2/IGateway.sol";
import {Asset, makeRawXCM, Payload} from "../src/v2/Types.sol";
import {Verification} from "../src/Verification.sol";
import {
    ForkTestFixtures,
    SubmitMessageFixture,
    SubmitV2MessageFixture
} from "../test/utils/ForkTestFixtures.sol";

/// @notice Exercises the candidate and deployed 202610 implementations against the production Gateway proxy.
contract ForkUpgrade202610Test is Test {
    address private constant GATEWAY_PROXY = 0x27ca963C279c93801941e1eB8799c23f407d68e7;
    address private constant OLD_BEEFY_CLIENT = 0x7cfc5C8b341991993080Af67D940B6aD19a010E1;
    address private constant BEEFY_CLIENT_202610 = 0xbCBEbc20a0FDF9EfD53B5852a71869b0cF25283f;
    address private constant GATEWAY_LOGIC_202610 = 0x4aD0577C76AfaEf8AaC23C661974340bB131426F;
    bytes32 private constant GATEWAY_CODEHASH_202610 =
        0x7082f0a78744653164d1a8cf7525634ea7fc009e27e0a81b5146e399415af5a8;
    address private constant AGENT_EXECUTOR_202610 = 0x77C58FA061ADE2382f9A90f3FE8DCC87CAfD39a6;

    // Verification library linked into the Gateway logic at the historical fixture blocks
    address private constant VERIFICATION_V1 = 0x4058bE4f048F55F1e9C88da09738070C0a7F1593;
    // Verification library linked into the 202602 Gateway logic live on mainnet
    address private constant VERIFICATION_202602 = 0x53fe58beFD174f513871519655c120436C128152;
    bytes4 private constant VERIFICATION_SELECTOR = 0x2db72616;

    ChannelID private constant GOVERNANCE_CHANNEL =
        ChannelID.wrap(0x0000000000000000000000000000000000000000000000000000000000000001);
    ChannelID private constant ASSETHUB_CHANNEL =
        ChannelID.wrap(0xc173fac324158e77fb5840738a1a541f633cbec8884c6a601c567d2b376a0539);
    bytes32 private constant ASSETHUB_AGENT_ID =
        0x81c5ab2571199e3188135178f3c2c8e2d268be1313d029b30f534fa579b69b79;
    address private constant ASSETHUB_AGENT = 0xd803472c47a87D7B63E888DE53f03B4191B846a8;
    address private constant DOT = 0x196C20DA81Fbc324EcdF55501e95Ce9f0bD84d14;
    bytes32 private constant DOT_ID =
        0x4e241583d94b5d48a27a22064cd49b2ed6f5231d2d950e432f9b7c2e0ade52b2;
    address private constant WETH = 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2;
    WETH9 public weth = WETH9(payable(WETH));

    string private constant FORK_RPC_URL =
        "https://virtual.mainnet.eu.rpc.tenderly.co/61589e0a-d204-449b-b095-64366ea949cb";

    event Transfer(address indexed from, address indexed to, uint256 value);

    function upgradeGateway() internal {
        AgentExecutor executor = new AgentExecutor();
        Gateway202610 newLogic = new Gateway202610(OLD_BEEFY_CLIENT, address(executor));
        _upgradeTo(VERIFICATION_V1, address(newLogic), OLD_BEEFY_CLIENT, address(executor));
    }

    function upgradeToDeployed202610() internal {
        assertEq(GATEWAY_LOGIC_202610.codehash, GATEWAY_CODEHASH_202610);
        _upgradeTo(
            VERIFICATION_202602, GATEWAY_LOGIC_202610, BEEFY_CLIENT_202610, AGENT_EXECUTOR_202610
        );
    }

    function _upgradeTo(
        address verificationAddr,
        address newLogic,
        address expectedBeefyClient,
        address expectedExecutor
    ) private {
        (uint64 inbound, uint64 outbound) =
            IGatewayV1(GATEWAY_PROXY).channelNoncesOf(ASSETHUB_CHANNEL);
        uint256 agentBalance = ASSETHUB_AGENT.balance;

        vm.mockCall(
            verificationAddr, abi.encodeWithSelector(VERIFICATION_SELECTOR), abi.encode(true)
        );

        UpgradeParams memory params = UpgradeParams({
            impl: newLogic, implCodeHash: newLogic.codehash, initParams: bytes("")
        });

        (bytes32[] memory proof1, Verification.Proof memory proof2) =
            ForkTestFixtures.makeMockProofs();
        (uint64 nonce,) = IGatewayV1(GATEWAY_PROXY).channelNoncesOf(GOVERNANCE_CHANNEL);

        vm.expectEmit();
        emit IUpgradable.Upgraded(newLogic);
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

        (uint64 nonceAfter,) = IGatewayV1(GATEWAY_PROXY).channelNoncesOf(GOVERNANCE_CHANNEL);
        assertEq(nonceAfter, nonce + 1);

        (uint64 inboundAfter, uint64 outboundAfter) =
            IGatewayV1(GATEWAY_PROXY).channelNoncesOf(ASSETHUB_CHANNEL);
        assertEq(inboundAfter, inbound);
        assertEq(outboundAfter, outbound);
        assertEq(IGatewayV1(GATEWAY_PROXY).agentOf(ASSETHUB_AGENT_ID), ASSETHUB_AGENT);
        assertEq(ASSETHUB_AGENT.balance, agentBalance);

        assertEq(Gateway(payable(GATEWAY_PROXY)).BEEFY_CLIENT(), expectedBeefyClient);
        assertEq(Gateway(payable(GATEWAY_PROXY)).AGENT_EXECUTOR(), expectedExecutor);
    }

    function testUpgradedGatewayAcceptsV1UnlockNativeEther() public {
        vm.createSelectFork(FORK_RPC_URL, 24_681_921);
        upgradeGateway();
        SubmitMessageFixture memory fixture = ForkTestFixtures.makeSubmitMessageFixture(
            "/test/data/mainnet-gateway-submitv1-unlock-ether-202603.json"
        );

        vm.expectEmit(true, true, true, true);
        emit IGatewayV1.InboundMessageDispatched(
            fixture.message.channelID, fixture.message.nonce, fixture.message.id, true
        );

        address relayer = makeAddr("relayer");
        vm.deal(relayer, 10 ether);
        vm.prank(relayer);
        IGatewayV1(GATEWAY_PROXY).submitV1(fixture.message, fixture.leafProof, fixture.headerProof);
    }

    function testUpgradedGatewayAcceptsV2UnlockNativeEther() public {
        vm.createSelectFork(FORK_RPC_URL, 24_677_447);
        upgradeGateway();
        SubmitV2MessageFixture memory fixture = ForkTestFixtures.makeSubmitV2MessageFixture(
            "/test/data/mainnet-gateway-submitv2-unlock-ether.json"
        );

        vm.expectEmit(true, true, true, true);
        emit IGatewayV2.InboundMessageDispatched(
            fixture.message.nonce, fixture.message.topic, true, fixture.rewardAddress
        );

        address relayer = makeAddr("relayer");
        vm.deal(relayer, 10 ether);
        vm.prank(relayer);
        IGatewayV2(GATEWAY_PROXY)
            .v2_submit(
                fixture.message, fixture.leafProof, fixture.headerProof, fixture.rewardAddress
            );
    }

    function testDeployedGateway202610UpgradeAndOutbound() public {
        vm.createSelectFork(vm.envString("MAINNET_ARCHIVE_RPC_URL"), 26_304_988);
        upgradeToDeployed202610();

        address user = makeAddr("user");
        uint128 amount = 1;
        ParaID paraID = ParaID.wrap(1000);
        MultiAddress memory recipientAddress32 = multiAddressFromBytes32(keccak256("recipient"));

        uint128 fee = uint128(IGatewayV1(GATEWAY_PROXY).quoteSendTokenFee(address(0), paraID, 1));
        assertTrue(fee > 0 && fee < 0.01 ether);

        uint256 balanceBefore = ASSETHUB_AGENT.balance;
        vm.expectEmit();
        emit IGatewayV1.TokenSent(address(0), user, paraID, recipientAddress32, amount);
        hoax(user, amount + fee);
        IGatewayV1(GATEWAY_PROXY).sendToken{value: amount + fee}(
            address(0), paraID, recipientAddress32, 1, amount
        );
        assertEq(ASSETHUB_AGENT.balance, balanceBefore + amount);

        uint128 v2Amount = uint128(0.1 ether);
        uint128 executionFee = 0.01 ether;
        uint128 relayerFee = 0.01 ether;
        uint128 v2Fee = executionFee + relayerFee;
        uint64 nonceBefore = IGatewayV2(GATEWAY_PROXY).v2_outboundNonce();

        vm.expectEmit(true, false, false, false);
        emit IGatewayV2.OutboundMessageAccepted(
            1,
            Payload({
                origin: user,
                assets: new Asset[](0),
                xcm: makeRawXCM(""),
                claimer: "",
                value: v2Amount,
                executionFee: executionFee,
                relayerFee: relayerFee
            })
        );
        hoax(user, v2Amount + v2Fee);
        IGatewayV2(payable(GATEWAY_PROXY)).v2_sendMessage{value: v2Amount + v2Fee}(
            "", new bytes[](0), "", executionFee, relayerFee
        );
        assertEq(IGatewayV2(GATEWAY_PROXY).v2_outboundNonce(), nonceBefore + 1);
    }

    function testDeployedGateway202610CanMintAndSendDOT() public {
        vm.createSelectFork(vm.envString("MAINNET_ARCHIVE_RPC_URL"), 26_304_988);
        upgradeToDeployed202610();

        address user = 0x302F0B71B8aD3CF6dD90aDb668E49b2168d652fd;
        vm.deal(user, 1 ether);

        uint128 amount = 100;
        MintForeignTokenParams memory params =
            MintForeignTokenParams({foreignTokenID: DOT_ID, recipient: user, amount: amount});

        vm.expectEmit(true, true, false, false);
        emit Transfer(address(0), user, amount);

        ParaID paraID = ParaID.wrap(1000);
        vm.prank(GATEWAY_PROXY);
        Gateway(GATEWAY_PROXY).v1_handleMintForeignToken(paraID.into(), abi.encode(params));

        MultiAddress memory recipientAddress32 = multiAddressFromBytes32(keccak256("recipient"));
        uint128 fee = uint128(IGatewayV1(GATEWAY_PROXY).quoteSendTokenFee(DOT, paraID, 1));
        assertTrue(fee > 0 && fee < 0.01 ether);

        vm.prank(user);
        vm.expectEmit();
        emit IGatewayV1.TokenSent(DOT, user, paraID, recipientAddress32, amount);
        IGatewayV1(GATEWAY_PROXY).sendToken{value: fee}(
            DOT, paraID, recipientAddress32, 1, amount
        );
    }
}
