// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.34;

import {Vm} from "forge-std/Vm.sol";
import {Test} from "forge-std/Test.sol";
import {stdJson} from "forge-std/StdJson.sol";
import {console} from "forge-std/console.sol";
import {UD60x18, ud60x18, unwrap} from "prb/math/src/UD60x18.sol";
import {WETH9} from "canonical-weth/WETH9.sol";

import {IUpgradable} from "../src/interfaces/IUpgradable.sol";
import {IGatewayV1} from "../src/v1/IGateway.sol";
import {IGatewayV2} from "../src/v2/IGateway.sol";
import {Verification} from "../src/Verification.sol";
import {Gateway} from "../src/Gateway.sol";
import {Gateway202602} from "../src/upgrade/Gateway202602.sol";
import {AgentExecutor} from "../src/AgentExecutor.sol";
import {
    UpgradeParams,
    SetOperatingModeParams,
    OperatingMode,
    RegisterForeignTokenParams
} from "../src/v1/Types.sol";
import {
    ChannelID,
    ParaID,
    OperatingMode,
    InboundMessage,
    Command,
    TokenInfo,
    MintForeignTokenParams
} from "../src/v1/Types.sol";
import {MultiAddress, multiAddressFromBytes32} from "../src/v1/MultiAddress.sol";
import {
    ForkTestFixtures,
    SubmitMessageFixture,
    SubmitV2MessageFixture
} from "../test/utils/ForkTestFixtures.sol";

contract ForkUpgrade202603Test is Test {
    address private constant GATEWAY_PROXY = 0x27ca963C279c93801941e1eB8799c23f407d68e7;
    address private constant BEEFY_CLIENT = 0x7cfc5C8b341991993080Af67D940B6aD19a010E1;

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

        // Deploy new implementation contract
        Gateway202602 newLogic = new Gateway202602(BEEFY_CLIENT, address(new AgentExecutor()));

        // Prepare upgrade command
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

    function testUpgradedGatewayStillAcceptsV2UnlockNativeEther() public {
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

    function testUpgradedGatewayStillAcceptsV1UnlockNativeEther() public {
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
}
