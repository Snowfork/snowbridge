// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.34;

import {Test} from "forge-std/Test.sol";

import {Gateway} from "../src/Gateway.sol";
import {IERC20} from "../src/interfaces/IERC20.sol";
import {IUpgradable} from "../src/interfaces/IUpgradable.sol";
import {IGatewayV1} from "../src/v1/IGateway.sol";
import {ChannelID, Command, InboundMessage, UpgradeParams} from "../src/v1/Types.sol";
import {IGatewayV2} from "../src/v2/IGateway.sol";
import {
    Asset,
    Command as CommandV2,
    CommandKind,
    InboundMessage as InboundMessageV2,
    makeForeignAsset,
    makeRawXCM,
    MintForeignTokenParams,
    Payload,
    UnlockNativeTokenParams
} from "../src/v2/Types.sol";
import {Verification} from "../src/Verification.sol";
import {ForkTestFixtures} from "../test/utils/ForkTestFixtures.sol";

/// @notice Upgrades the production Gateway proxy to the deployed 202610 logic and exercises it.
contract ForkUpgrade202610Test is Test {
    address private constant GATEWAY_PROXY = 0x27ca963C279c93801941e1eB8799c23f407d68e7;
    address private constant BEEFY_CLIENT_NEW = 0xbCBEbc20a0FDF9EfD53B5852a71869b0cF25283f;
    address private constant GATEWAY_LOGIC_NEW = 0x4aD0577C76AfaEf8AaC23C661974340bB131426F;
    // Must match the code hash in the `upgrade-202610` preimage.
    bytes32 private constant GATEWAY_CODEHASH_NEW =
        0x7082f0a78744653164d1a8cf7525634ea7fc009e27e0a81b5146e399415af5a8;
    address private constant AGENT_EXECUTOR_NEW = 0x77C58FA061ADE2382f9A90f3FE8DCC87CAfD39a6;

    // Verification libraries linked into the live and the new Gateway logic.
    address private constant VERIFICATION = 0x53fe58beFD174f513871519655c120436C128152;
    address private constant VERIFICATION_NEW = 0xC83298032A03eBA586959Ddc70eb412646A83cB3;
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

    function setUp() public {
        vm.createSelectFork(vm.envString("MAINNET_ARCHIVE_RPC_URL"), 26_304_988);
        upgradeGateway();
    }

    // Governance submits the upgrade as a V1 command, as the preimage does.
    function upgradeGateway() internal {
        (uint64 inbound, uint64 outbound) =
            IGatewayV1(GATEWAY_PROXY).channelNoncesOf(ASSETHUB_CHANNEL);
        uint256 agentBalance = ASSETHUB_AGENT.balance;

        vm.mockCall(VERIFICATION, abi.encodeWithSelector(VERIFICATION_SELECTOR), abi.encode(true));

        UpgradeParams memory params = UpgradeParams({
            impl: GATEWAY_LOGIC_NEW, implCodeHash: GATEWAY_CODEHASH_NEW, initParams: bytes("")
        });

        (bytes32[] memory proof1, Verification.Proof memory proof2) =
            ForkTestFixtures.makeMockProofs();
        (uint64 nonce,) = IGatewayV1(GATEWAY_PROXY).channelNoncesOf(GOVERNANCE_CHANNEL);

        vm.expectEmit();
        emit IUpgradable.Upgraded(GATEWAY_LOGIC_NEW);
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

        assertEq(Gateway(payable(GATEWAY_PROXY)).BEEFY_CLIENT(), BEEFY_CLIENT_NEW);
        assertEq(Gateway(payable(GATEWAY_PROXY)).AGENT_EXECUTOR(), AGENT_EXECUTOR_NEW);
    }

    // BEEFY verification against the new BeefyClient is covered by the ForkBeefyClient202610 tests.
    function submitV2(uint64 nonce, CommandV2 memory command) internal {
        vm.mockCall(
            VERIFICATION_NEW, abi.encodeWithSelector(VERIFICATION_SELECTOR), abi.encode(true)
        );
        assertFalse(IGatewayV2(GATEWAY_PROXY).v2_isDispatched(nonce));

        CommandV2[] memory commands = new CommandV2[](1);
        commands[0] = command;
        InboundMessageV2 memory message = InboundMessageV2({
            origin: ASSETHUB_AGENT_ID, nonce: nonce, topic: keccak256("topic"), commands: commands
        });
        (bytes32[] memory leafProof, Verification.Proof memory headerProof) =
            ForkTestFixtures.makeMockProofs();
        bytes32 rewardAddress = keccak256("relayer");

        vm.expectEmit();
        emit IGatewayV2.InboundMessageDispatched(nonce, message.topic, true, rewardAddress);
        vm.prank(makeAddr("relayer"));
        IGatewayV2(GATEWAY_PROXY).v2_submit(message, leafProof, headerProof, rewardAddress);
        assertTrue(IGatewayV2(GATEWAY_PROXY).v2_isDispatched(nonce));
    }

    function testV2UnlockNativeEther() public {
        address recipient = makeAddr("recipient");
        uint128 amount = 0.1 ether;
        uint256 agentBalance = ASSETHUB_AGENT.balance;

        submitV2(
            type(uint64).max,
            CommandV2({
                kind: CommandKind.UnlockNativeToken,
                gas: 100_000,
                payload: abi.encode(
                    UnlockNativeTokenParams({
                        token: address(0), recipient: recipient, amount: amount
                    })
                )
            })
        );

        assertEq(recipient.balance, amount);
        assertEq(ASSETHUB_AGENT.balance, agentBalance - amount);
    }

    function testV2MintAndSendDOT() public {
        address user = makeAddr("user");
        uint128 amount = 100;

        submitV2(
            type(uint64).max,
            CommandV2({
                kind: CommandKind.MintForeignToken,
                gas: 100_000,
                payload: abi.encode(
                    MintForeignTokenParams({
                        foreignTokenID: DOT_ID, recipient: user, amount: amount
                    })
                )
            })
        );
        assertEq(IERC20(DOT).balanceOf(user), amount);

        uint128 executionFee = 0.01 ether;
        uint128 relayerFee = 0.01 ether;
        uint64 nonceBefore = IGatewayV2(GATEWAY_PROXY).v2_outboundNonce();
        bytes[] memory assets = new bytes[](1);
        assets[0] = abi.encode(0, DOT, amount);
        Asset[] memory expectedAssets = new Asset[](1);
        expectedAssets[0] = makeForeignAsset(DOT_ID, amount);

        vm.expectEmit();
        emit IGatewayV2.OutboundMessageAccepted(
            nonceBefore + 1,
            Payload({
                origin: user,
                assets: expectedAssets,
                xcm: makeRawXCM(""),
                claimer: "",
                value: 0,
                executionFee: executionFee,
                relayerFee: relayerFee
            })
        );
        hoax(user, executionFee + relayerFee);
        IGatewayV2(GATEWAY_PROXY).v2_sendMessage{value: executionFee + relayerFee}(
            "", assets, "", executionFee, relayerFee
        );

        assertEq(IGatewayV2(GATEWAY_PROXY).v2_outboundNonce(), nonceBefore + 1);
        assertEq(IERC20(DOT).balanceOf(user), 0);
    }

    function testV2SendEther() public {
        address user = makeAddr("user");
        uint128 value = 0.1 ether;
        uint128 executionFee = 0.01 ether;
        uint128 relayerFee = 0.01 ether;
        uint128 total = value + executionFee + relayerFee;
        uint64 nonceBefore = IGatewayV2(GATEWAY_PROXY).v2_outboundNonce();
        uint256 agentBalance = ASSETHUB_AGENT.balance;

        vm.expectEmit();
        emit IGatewayV2.OutboundMessageAccepted(
            nonceBefore + 1,
            Payload({
                origin: user,
                assets: new Asset[](0),
                xcm: makeRawXCM(""),
                claimer: "",
                value: value,
                executionFee: executionFee,
                relayerFee: relayerFee
            })
        );
        hoax(user, total);
        IGatewayV2(GATEWAY_PROXY).v2_sendMessage{value: total}(
            "", new bytes[](0), "", executionFee, relayerFee
        );

        assertEq(IGatewayV2(GATEWAY_PROXY).v2_outboundNonce(), nonceBefore + 1);
        assertEq(ASSETHUB_AGENT.balance, agentBalance + total);
    }
}
