// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.33;

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
import {ChannelID, ParaID, OperatingMode, InboundMessage, Command, TokenInfo, MintForeignTokenParams} from "../src/v1/Types.sol";
import {MultiAddress, multiAddressFromBytes32} from "../src/v1/MultiAddress.sol";
import {ForkTestFixtures, SubmitMessageFixture} from "./utils/ForkTestFixtures.sol";
import {WETH9} from "canonical-weth/WETH9.sol";

contract ForkUpgradeTest is Test {
    address private constant GATEWAY_PROXY = 0x27ca963C279c93801941e1eB8799c23f407d68e7;
    address private constant BEEFY_CLIENT = 0x1817874feAb3ce053d0F40AbC23870DB35C2AFfc;

    // NOTE: Can use tenderly transaction debugger to retrieve existing library address
    address private constant VERIFICATION_ADDR_V1 = 0x90c7F378e9ceD5dD268f0dF987c0838469846Da1;
    bytes4 private constant VERIFICATION_SELECTOR_V1 = 0xbc9535d4;

    ChannelID constant internal GOVERNANCE_CHANNEL = ChannelID.wrap(0x0000000000000000000000000000000000000000000000000000000000000001);
    ChannelID constant internal ASSETHUB_CHANNEL = ChannelID.wrap(0xc173fac324158e77fb5840738a1a541f633cbec8884c6a601c567d2b376a0539);
    bytes32 constant internal ASSETHUB_AGENT_ID =
        0x81c5ab2571199e3188135178f3c2c8e2d268be1313d029b30f534fa579b69b79;
    address constant internal assetHubAgent = 0xd803472c47a87D7B63E888DE53f03B4191B846a8;
    address constant internal DOT = 0x196C20DA81Fbc324EcdF55501e95Ce9f0bD84d14;
    bytes32 constant internal DOT_ID = 0x4e241583d94b5d48a27a22064cd49b2ed6f5231d2d950e432f9b7c2e0ade52b2;
    address constant internal WETH = 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2;
    WETH9 public weth = WETH9(payable(WETH));

    event Transfer(address indexed from, address indexed to, uint256 value);

    function setUp() public {

    }

    function setUp23432697() public {
        vm.createSelectFork("https://virtual.mainnet.eu.rpc.tenderly.co/dc7e68d7-5fac-4a88-a797-01ed4155f437", 23432697);

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
    function testUpgradedGatewayStillAcceptsMintDOT() public {
       setUp23432697();
       SubmitMessageFixture memory fixture = ForkTestFixtures.makeSubmitMessageFixture("/test/data/mainnet-gateway-submitv1-mint-dot.json");

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

    // Send Ether can work with the upgraded Gateway
    function testUpgradedGatewayStillCanSendEther() public {
        setUp23432697();
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

    function setUp23526799() public {
        vm.createSelectFork("https://virtual.mainnet.eu.rpc.tenderly.co/1d4ab8c5-01fe-45a7-8583-8b4925e5a435", 23526799);

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
    }

    function testUpgradedGatewayStillAcceptsUnlockNativeEther() public {
       setUp23526799();
       SubmitMessageFixture memory fixture = ForkTestFixtures.makeSubmitMessageFixture("/test/data/mainnet-gateway-submitv1-unlock-ether.json");

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

    function testUpgradedGatewayStillAcceptsUnlockWEth() public {
       testUpgradedGatewayStillAcceptsUnlockNativeEther();

       SubmitMessageFixture memory fixture = ForkTestFixtures.makeSubmitMessageFixture("/test/data/mainnet-gateway-submitv1-unlock-weth.json");

       // Expect the gateway to emit InboundMessageDispatched event
       vm.expectEmit(true, true, true, true);
       emit IGatewayV1.InboundMessageDispatched(
           fixture.message.channelID,
           fixture.message.nonce,
           fixture.message.id,
           true
       );

       address relayer = makeAddr("relayer");
       vm.prank(relayer);
       IGatewayV1(address(GATEWAY_PROXY)).submitV1(
           fixture.message,
           fixture.leafProof,
           fixture.headerProof
       );
    }

    // Send DOT can work with the upgraded Gateway
    function testUpgradedGatewayStillCanSendDOT() public {
        setUp23526799();
        address user = 0x302F0B71B8aD3CF6dD90aDb668E49b2168d652fd;
        vm.deal(user, 1 ether);

        vm.prank(address(GATEWAY_PROXY));
        uint128 amount = 100;
        MintForeignTokenParams memory params = MintForeignTokenParams({
            foreignTokenID: DOT_ID,
            recipient: user,
            amount: amount
        });

        vm.expectEmit(true, true, false, false);
        emit Transfer(address(0), user, amount);

        ParaID paraID = ParaID.wrap(1000);
        Gateway(address(GATEWAY_PROXY)).v1_handleMintForeignToken(
            paraID.into(), abi.encode(params)
        );

        MultiAddress memory recipientAddress32 = multiAddressFromBytes32(keccak256("recipient"));

        uint128 fee =
            uint128(IGatewayV1(address(GATEWAY_PROXY)).quoteSendTokenFee(DOT, paraID, 1));
        assertTrue(fee > 0);
        assertTrue(fee < 0.01 ether);

        vm.prank(user);
        vm.expectEmit();
        emit IGatewayV1.TokenSent(DOT, user, paraID, recipientAddress32, amount);
        vm.expectEmit(true, true, true, false);
        emit IGatewayV1.OutboundMessageAccepted(paraID.into(), 6847, bytes32(0x65b3c8970f6316e368291bddf21059f298f18ad356faafed0ac19244f15dc67f), hex"");
        IGatewayV1(address(GATEWAY_PROXY)).sendToken{value: fee}(
            DOT, paraID, recipientAddress32, 1, amount
        );
    }

    // Send WETH can work with the upgraded Gateway
    function testUpgradedGatewayStillCanSendWETH() public {
        setUp23526799();
        address user = 0x091229C0b1465DD8D9977f1C45D368BBC34b8405;

        MultiAddress memory recipientAddress32 = multiAddressFromBytes32(keccak256("recipient"));

        uint128 amount = 100;
        ParaID paraID = ParaID.wrap(1000);
        uint128 fee =
            uint128(IGatewayV1(address(GATEWAY_PROXY)).quoteSendTokenFee(WETH, paraID, 1));
        assertTrue(fee > 0);
        assertTrue(fee < 0.01 ether);

        vm.prank(user);
        weth.approve(address(GATEWAY_PROXY), amount);

        vm.prank(user);
        vm.expectEmit();
        emit IGatewayV1.TokenSent(WETH, user, paraID, recipientAddress32, amount);
        vm.expectEmit(true, false, false, false);
        emit IGatewayV1.OutboundMessageAccepted(paraID.into(), 1, bytes32("0x"), hex"");
        IGatewayV1(address(GATEWAY_PROXY)).sendToken{value: fee}(
            WETH, paraID, recipientAddress32, 1, amount
        );
    }
}
