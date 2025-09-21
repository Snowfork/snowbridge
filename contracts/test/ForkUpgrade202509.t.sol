// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import {Vm} from "forge-std/Vm.sol";
import {Test} from "forge-std/Test.sol";
import {stdJson} from "forge-std/StdJson.sol";
import {console} from "forge-std/console.sol";

import {IUpgradable} from "../src/interfaces/IUpgradable.sol";
import {IGatewayV1} from "../src/v1/IGateway.sol";
import {Verification} from "../src/Verification.sol";
import {Gateway} from "../src/Gateway.sol";
import {Gateway202509} from "../src/upgrade/Gateway202509.sol";
import {AgentExecutor} from "../src/AgentExecutor.sol";
import {UpgradeParams, SetOperatingModeParams, OperatingMode, RegisterForeignTokenParams} from "../src/v1/Types.sol";
import {ChannelID, ParaID, OperatingMode, InboundMessage, Command, TokenInfo} from "../src/v1/Types.sol";
import {MultiAddress, multiAddressFromBytes32} from "../src/v1/MultiAddress.sol";
import {ForkTestFixtures, SubmitMessageFixture} from "./utils/ForkTestFixtures.sol";

contract ForkUpgradeTest is Test {
    address private constant GATEWAY_PROXY = 0x27ca963C279c93801941e1eB8799c23f407d68e7;
    address private constant BEEFY_CLIENT = 0x6eD05bAa904df3DE117EcFa638d4CB84e1B8A00C;

    // NOTE: Can use tenderly transaction debugger to retrieve existing library address
    address private constant VERIFICATION_ADDR = 0x90c7f378e9ced5dd268f0df987c0838469846da1;

    ChannelID constant internal GOVERNANCE_CHANNEL = ChannelID.wrap(0x0000000000000000000000000000000000000000000000000000000000000001);

    function setUp() public {
        vm.createSelectFork("https://rpc.tenderly.co/fork/cdff755d-46fc-47e2-8a9d-b1269fa86e72", 21945142);

        // Mock call to Verification.verifyCommitment to bypass BEEFY verification.
        // Note that after the gateway is upgraded, the gateway will be linked to a new Verification
        // library, essentially undoing this mock.
        vm.mockCall(VERIFICATION_ADDR, abi.encodeWithSelector(Verification.verifyCommitment.selector), abi.encode(true));

        // Deploy new implementation contract
        Gateway202509 newLogic = new Gateway202509(
            BEEFY_CLIENT,
            address(new AgentExecutor()),
        );

        // Prepare upgrade command
        UpgradeParams memory params = UpgradeParams({
            impl: address(newLogic),
            implCodeHash: address(newLogic).codehash,
            initParams: bytes("")
        });

        (bytes32[] memory proof1, Verification.Proof memory proof2) = ForkTestFixtures.makeMockProofs();
        (uint64 nonce,) = IGateway(GATEWAY_PROXY).channelNoncesOf(PRIMARY_GOVERNANCE_CHANNEL);

        vm.expectEmit();
        emit IUpgradable.Upgraded(address(newLogic));

        // Issue the upgrade
        IGateway(GATEWAY_PROXY).submitV1(
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
    }

    // Submit a cross-chain message to the upgraded Gateway, using a real-world data
    // captured from mainnet. Verifies that cross-chain signalling is not broken by the upgrade.
    function testUpgradedGatewayStillAcceptsMessages() public {
        SubmitMessageFixture memory fixture = ForkTestFixtures.makeSubmitMessageFixture("/test/data/mainnet-gateway-submitv1.json");

        // Expect the gateway to emit InboundMessageDispatched event
        vm.expectEmit(true, true, true, true);
        emit IGateway.InboundMessageDispatched(
            fixture.message.channelID,
            fixture.message.nonce,
            fixture.message.id,
            true
        );

        address relayer = makeAddr("relayer");
        vm.deal(relayer, 10 ether);

        vm.prank(relayer);
        IGateway(address(GATEWAY_PROXY)).submitV1(
            fixture.message,
            fixture.leafProof,
            fixture.headerProof
        );
    }
}
