// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import {Vm} from "forge-std/Vm.sol";
import {Test} from "forge-std/Test.sol";
import {stdJson} from "forge-std/StdJson.sol";
import {console} from "forge-std/console.sol";

import {IUpgradable} from "../src/interfaces/IUpgradable.sol";
import {IGateway} from "../src/interfaces/IGateway.sol";
import {Verification} from "../src/Verification.sol";
import {Gateway} from "../src/Gateway.sol";
import {Gateway202502} from "../src/upgrades/Gateway202502.sol";
import {AgentExecutor} from "../src/AgentExecutor.sol";
import {UpgradeParams, SetOperatingModeParams, OperatingMode, RegisterForeignTokenParams} from "../src/Params.sol";
import {ChannelID, ParaID, OperatingMode, InboundMessage, Command, TokenInfo} from "../src/Types.sol";
import {MultiAddress, multiAddressFromBytes32} from "../src/MultiAddress.sol";
import {ForkTestFixtures, SubmitMessageFixture} from "./utils/ForkTestFixtures.sol";

contract ForkUpgradeTest is Test {
    address private constant GATEWAY_PROXY = 0x27ca963C279c93801941e1eB8799c23f407d68e7;
    address private constant BEEFY_CLIENT = 0x6eD05bAa904df3DE117EcFa638d4CB84e1B8A00C;
    address private constant VERIFICATION_ADDR = 0x515c0817005b2F3383B7D8837d6DCc15c0d71C56;
    bytes32 private constant BRIDGE_HUB_AGENT_ID = 0x03170a2e7597b7b7e3d84c05391d139a62b157e78786d8c082f29dcf4c111314;

    ChannelID constant internal PRIMARY_GOVERNANCE_CHANNEL = ChannelID.wrap(0x0000000000000000000000000000000000000000000000000000000000000001);
    ChannelID constant internal SECONDARY_GOVERNANCE_CHANNEL = ChannelID.wrap(0x0000000000000000000000000000000000000000000000000000000000000002);

    uint256 mainnetForkBlock21945142;
    uint256 mainnetForkBlock21960630;

    function setUp() public {
        mainnetForkBlock21945142 = vm.createFork("https://rpc.tenderly.co/fork/cdff755d-46fc-47e2-8a9d-b1269fa86e72", 21945142);
        mainnetForkBlock21960630 = vm.createFork("https://rpc.tenderly.co/fork/f0404b3b-e58f-4429-88b6-ea87414be30c", 21960630);
    }

    // Submit a cross-chain message to the upgraded Gateway, using a real-world data
    // captured from mainnet. Verifies that cross-chain signalling is not broken by the upgrade.
    function testUpgradedGatewayStillAcceptsMessages() public {
        vm.selectFork(mainnetForkBlock21945142);

        // Mock call to Verification.verifyCommitment to bypass BEEFY verification.
        // Note that after the gateway is upgraded, the gateway will be linked to a new Verification
        // library, essentially undoing this mock.
        vm.mockCall(VERIFICATION_ADDR, abi.encodeWithSelector(Verification.verifyCommitment.selector), abi.encode(true));

        // Deploy new implementation contract
        Gateway202502 newLogic = new Gateway202502(
            BEEFY_CLIENT,
            address(new AgentExecutor()),
            ParaID.wrap(1002),
            BRIDGE_HUB_AGENT_ID,
            10,
            20_000_000_000 // 2 DOT
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
                PRIMARY_GOVERNANCE_CHANNEL,
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

    // Test the upgrade with the new gateway implementation contract: 0x4a4559CCD9195C3CABBd4Da00854A434E8dd2Ea3
    function testUpgrade() public {
        vm.selectFork(mainnetForkBlock21960630);

        // Mock call to Verification.verifyCommitment to bypass BEEFY verification.
        // Note that after the gateway is upgraded, the gateway will be linked to a new Verification
        // library, essentially undoing this mock.
        vm.mockCall(VERIFICATION_ADDR, abi.encodeWithSelector(Verification.verifyCommitment.selector), abi.encode(true));

        // Prepare upgrade command
        UpgradeParams memory params = UpgradeParams({
            impl: address(0x4a4559CCD9195C3CABBd4Da00854A434E8dd2Ea3),
            implCodeHash: 0xe3fcabb76657fab30cf73de98691f7d89fc04f82a8559257a40047c4e2fad623,
            initParams: bytes("")
        });

        (bytes32[] memory proof1, Verification.Proof memory proof2) = ForkTestFixtures.makeMockProofs();
        (uint64 nonce,) = IGateway(GATEWAY_PROXY).channelNoncesOf(PRIMARY_GOVERNANCE_CHANNEL);

        vm.expectEmit();
        emit IUpgradable.Upgraded(address(0x4a4559CCD9195C3CABBd4Da00854A434E8dd2Ea3));

        // Issue the upgrade
        IGateway(GATEWAY_PROXY).submitV1(
            InboundMessage(
                PRIMARY_GOVERNANCE_CHANNEL,
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
}
