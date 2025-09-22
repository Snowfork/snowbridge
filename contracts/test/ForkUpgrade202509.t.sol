// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import {Vm} from "forge-std/Vm.sol";
import {Test} from "forge-std/Test.sol";
import {stdJson} from "forge-std/StdJson.sol";
import {console} from "forge-std/console.sol";
import {UD60x18, ud60x18, unwrap} from "prb/math/src/UD60x18.sol";

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
    address private constant BEEFY_CLIENT = 0x9FC6a0eEf52BC839cF1A37554044f11782E4e2d3;

    // NOTE: Can use tenderly transaction debugger to retrieve existing library address
    address private constant VERIFICATION_ADDR_V1 = 0x90c7F378e9ceD5dD268f0dF987c0838469846Da1;
    bytes4 private constant VERIFICATION_SELECTOR_V1 = 0xbc9535d4;

    ChannelID constant internal GOVERNANCE_CHANNEL = ChannelID.wrap(0x0000000000000000000000000000000000000000000000000000000000000001);
    ChannelID constant internal ASSETHUB_CHANNEL = ChannelID.wrap(0xc173fac324158e77fb5840738a1a541f633cbec8884c6a601c567d2b376a0539);
    bytes32 constant internal ASSETHUB_AGENT_ID =
        0x81c5ab2571199e3188135178f3c2c8e2d268be1313d029b30f534fa579b69b79;
    address constant internal assetHubAgent = 0xd803472c47a87D7B63E888DE53f03B4191B846a8;

    function setUp() public {
        vm.createSelectFork("https://virtual.mainnet.eu.rpc.tenderly.co/83874628-cd7c-4179-96ad-52dc2710318c", 23419580);

        (UD60x18 exchangeRate, uint128 deliveryCost) = IGatewayV1(GATEWAY_PROXY).pricingParameters();
        assertEq(unwrap(exchangeRate), 2288329519450801);
        assertEq(deliveryCost, 223_565_000);

        (uint64 inbound, uint64 outbound) = IGatewayV1(GATEWAY_PROXY).channelNoncesOf(ASSETHUB_CHANNEL);

        uint256 balance = assetHubAgent.balance;

        // Mock call to Verification.verifyCommitment to bypass BEEFY verification.
        // Note that after the gateway is upgraded, the gateway will be linked to a new Verification
        // library, essentially undoing this mock.
        vm.mockCall(VERIFICATION_ADDR_V1, abi.encodeWithSelector(VERIFICATION_SELECTOR_V1), abi.encode(true));
        
        // Deploy new implementation contract
        Gateway202509 newLogic = new Gateway202509(
            BEEFY_CLIENT,
            address(new AgentExecutor())
        );

        // Prepare upgrade command
        UpgradeParams memory params = UpgradeParams({
            impl: address(newLogic),
            implCodeHash: address(newLogic).codehash,
            initParams: bytes("")
        });

        (bytes32[] memory proof1, Verification.Proof memory proof2) = ForkTestFixtures.makeMockProofs();
        (uint64 nonce,) = IGatewayV1(GATEWAY_PROXY).channelNoncesOf(GOVERNANCE_CHANNEL);

        vm.expectEmit();
        emit IUpgradable.Upgraded(address(newLogic));

        // Issue the upgrade
        IGatewayV1(GATEWAY_PROXY).submitV1(
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

        
        // Pricing parameters should be unchanged
        (UD60x18 exchangeRateAfter, uint128 deliveryCostAfter) = IGatewayV1(GATEWAY_PROXY).pricingParameters();
        assertEq(unwrap(exchangeRateAfter), unwrap(exchangeRate));
        assertEq(deliveryCostAfter, deliveryCost);

        // Governance channel inbound nonce should have incremented by 1
        (uint64 nonceAfter,) = IGatewayV1(GATEWAY_PROXY).channelNoncesOf(GOVERNANCE_CHANNEL);
        assertEq(nonceAfter, nonce + 1);

        // Asset hub channel nonces should be unchanged
        (uint64 inboundAfter, uint64 outboundAfter) = IGatewayV1(GATEWAY_PROXY).channelNoncesOf(ASSETHUB_CHANNEL);
        assertEq(inboundAfter, inbound);
        assertEq(outboundAfter, outbound);

        address agent = IGatewayV1(GATEWAY_PROXY).agentOf(ASSETHUB_AGENT_ID);
        assertEq(agent, assetHubAgent);

        // Asset hub agent balance should be unchanged
        uint256 balanceAfter = assetHubAgent.balance;
        assertEq(balanceAfter, balance);
    }

    // Submit a cross-chain message to the upgraded Gateway, using a real-world data
    // captured from mainnet. Verifies that cross-chain signalling is not broken by the upgrade.
    function testUpgradedGatewayStillAcceptsMessages() public {
       SubmitMessageFixture memory fixture = ForkTestFixtures.makeSubmitMessageFixture("/test/data/mainnet-gateway-submitv1.json");

       // Expect the gateway to emit InboundMessageDispatched event
       vm.expectEmit(true, true, true, true);
       emit IGatewayV1.InboundMessageDispatched(
           fixture.message.channelID,
           fixture.message.nonce,
           fixture.message.id,
           true
       );

       address relayer = makeAddr("relayer");
       vm.deal(relayer, 10 ether);

       vm.prank(relayer);
       IGatewayV1(address(GATEWAY_PROXY)).submitV1(
           fixture.message,
           fixture.leafProof,
           fixture.headerProof
       );
    }

    // Send token can work with the upgraded Gateway
    function testUpgradedGatewayStillCanSendToken() public {
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
  
        vm.expectEmit();
        emit IGatewayV1.TokenSent(address(0), user, paraID, recipientAddress32, amount);
        vm.expectEmit(true, false, false, false);
        emit IGatewayV1.OutboundMessageAccepted(paraID.into(), 1, bytes32('0x'), hex"");
        hoax(user, amount + fee);
        IGatewayV1(address(GATEWAY_PROXY)).sendToken{value: amount + fee}(
            address(0), paraID, recipientAddress32, 1, amount
        );
        assertEq(user.balance, 0);
    }
}
