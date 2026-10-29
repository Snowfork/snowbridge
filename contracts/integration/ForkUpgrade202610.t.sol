// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.34;

import {Test} from "forge-std/Test.sol";

import {AgentExecutor} from "../src/AgentExecutor.sol";
import {Gateway} from "../src/Gateway.sol";
import {IUpgradable} from "../src/interfaces/IUpgradable.sol";
import {Gateway202610} from "../src/upgrade/Gateway202610.sol";
import {IGatewayV1} from "../src/v1/IGateway.sol";
import {ChannelID, Command, InboundMessage, UpgradeParams} from "../src/v1/Types.sol";
import {IGatewayV2} from "../src/v2/IGateway.sol";
import {Verification} from "../src/Verification.sol";
import {
    ForkTestFixtures,
    SubmitMessageFixture,
    SubmitV2MessageFixture
} from "../test/utils/ForkTestFixtures.sol";

/// @notice Exercises the candidate implementation against the production Gateway proxy.
/// @dev Historical fixtures use the current production BeefyClient. The separately deployed
///      replacement is observed live before its address is supplied to the deployment script.
contract ForkUpgrade202610Test is Test {
    address private constant GATEWAY_PROXY = 0x27ca963C279c93801941e1eB8799c23f407d68e7;
    address private constant BEEFY_CLIENT = 0x7cfc5C8b341991993080Af67D940B6aD19a010E1;
    address private constant VERIFICATION = 0x4058bE4f048F55F1e9C88da09738070C0a7F1593;
    bytes4 private constant VERIFICATION_SELECTOR = 0x2db72616;

    ChannelID private constant GOVERNANCE_CHANNEL =
        ChannelID.wrap(0x0000000000000000000000000000000000000000000000000000000000000001);
    ChannelID private constant ASSETHUB_CHANNEL =
        ChannelID.wrap(0xc173fac324158e77fb5840738a1a541f633cbec8884c6a601c567d2b376a0539);
    bytes32 private constant ASSETHUB_AGENT_ID =
        0x81c5ab2571199e3188135178f3c2c8e2d268be1313d029b30f534fa579b69b79;
    address private constant ASSETHUB_AGENT = 0xd803472c47a87D7B63E888DE53f03B4191B846a8;

    string private constant FORK_RPC_URL =
        "https://virtual.mainnet.eu.rpc.tenderly.co/61589e0a-d204-449b-b095-64366ea949cb";

    function upgradeGateway() internal {
        (uint64 inbound, uint64 outbound) =
            IGatewayV1(GATEWAY_PROXY).channelNoncesOf(ASSETHUB_CHANNEL);
        uint256 agentBalance = ASSETHUB_AGENT.balance;

        vm.mockCall(VERIFICATION, abi.encodeWithSelector(VERIFICATION_SELECTOR), abi.encode(true));

        AgentExecutor executor = new AgentExecutor();
        Gateway202610 newLogic = new Gateway202610(BEEFY_CLIENT, address(executor));
        UpgradeParams memory params = UpgradeParams({
            impl: address(newLogic),
            implCodeHash: address(newLogic).codehash,
            initParams: bytes("")
        });

        (bytes32[] memory proof1, Verification.Proof memory proof2) =
            ForkTestFixtures.makeMockProofs();
        (uint64 nonce,) = IGatewayV1(GATEWAY_PROXY).channelNoncesOf(GOVERNANCE_CHANNEL);

        vm.expectEmit();
        emit IUpgradable.Upgraded(address(newLogic));
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

        assertEq(Gateway(payable(GATEWAY_PROXY)).BEEFY_CLIENT(), BEEFY_CLIENT);
        assertEq(Gateway(payable(GATEWAY_PROXY)).AGENT_EXECUTOR(), address(executor));
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
}
