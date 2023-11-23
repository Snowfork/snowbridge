// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.22;

import {Test} from "forge-std/Test.sol";
import {Strings} from "openzeppelin/utils/Strings.sol";
import {console} from "forge-std/console.sol";

import {BeefyClient} from "../src/BeefyClient.sol";

import {IGateway} from "../src/interfaces/IGateway.sol";
import {IInitializable} from "../src/interfaces/IInitializable.sol";
import {Gateway} from "../src/Gateway.sol";
import {Fee} from "../src/Types.sol";
import {GatewayMock, GatewayV2} from "./mocks/GatewayMock.sol";

import {GatewayProxy} from "../src/GatewayProxy.sol";

import {AgentExecutor} from "../src/AgentExecutor.sol";
import {Agent} from "../src/Agent.sol";
import {Verification} from "../src/Verification.sol";
import {Assets} from "../src/Assets.sol";
import {SubstrateTypes} from "./../src/SubstrateTypes.sol";

import {NativeTransferFailed} from "../src/utils/SafeTransfer.sol";

import {
    AgentExecuteCommand,
    InboundMessage,
    OperatingMode,
    ParaID,
    Command,
    multiAddressFromBytes32,
    multiAddressFromBytes20
} from "../src/Types.sol";

import {WETH9} from "canonical-weth/WETH9.sol";
import "./mocks/GatewayUpgradeMock.sol";
import {UD60x18, ud60x18, convert} from "prb/math/src/UD60x18.sol";

contract GatewayTest is Test {
    ParaID public bridgeHubParaID = ParaID.wrap(1001);
    bytes32 public bridgeHubAgentID = keccak256("1001");
    address public bridgeHubAgent;

    ParaID public assetHubParaID = ParaID.wrap(1002);
    bytes32 public assetHubAgentID = keccak256("1002");
    address public assetHubAgent;

    address public relayer;

    bytes32[] public proof = [bytes32(0x2f9ee6cfdf244060dc28aa46347c5219e303fc95062dd672b4e406ca5c29764b)];
    bytes public parachainHeaderProof = bytes("validProof");

    GatewayMock public gatewayLogic;
    GatewayProxy public gateway;

    WETH9 public token;

    address public account1;
    address public account2;

    uint256 public maxDispatchGas = 500_000;
    uint256 public maxRefund = 1 ether;
    uint256 public reward = 1 ether;
    bytes32 public messageID = keccak256("cabbage");

    // remote fees in DOT
    uint128 public outboundFee = 1e10;
    uint128 public registerTokenFee = 1e10;
    uint128 public sendTokenFee = 1e10;

    MultiAddress public recipientAddress32;
    MultiAddress public recipientAddress20;

    // DOT amounts need to be multiplied by 10^(18 - 10) to have the same number
    // decimal places as ETH (18 decimal places)
    // UD60x18.convert(1e8) == ud60x18(1e26)
    UD60x18 public dotToEthDecimals = ud60x18(1e26);

    // ETH/DOT exchange rate
    UD60x18 public exchangeRate = ud60x18(0.0025e18);

    function setUp() public {
        AgentExecutor executor = new AgentExecutor();
        gatewayLogic = new GatewayMock(
            address(0),
            address(executor),
            bridgeHubParaID,
            bridgeHubAgentID,
            assetHubParaID,
            assetHubAgentID,
            dotToEthDecimals
        );
        gateway = new GatewayProxy(
            address(gatewayLogic),
            abi.encode(
                OperatingMode.Normal,
                outboundFee,
                registerTokenFee,
                sendTokenFee,
                exchangeRate
            )
        );
        GatewayMock(address(gateway)).setCommitmentsAreVerified(true);

        Gateway.SetOperatingModeParams memory params = Gateway.SetOperatingModeParams({mode: OperatingMode.Normal});
        GatewayMock(address(gateway)).setOperatingModePublic(abi.encode(params));

        bridgeHubAgent = IGateway(address(gateway)).agentOf(bridgeHubAgentID);
        assetHubAgent = IGateway(address(gateway)).agentOf(assetHubAgentID);

        // fund the message relayer account
        relayer = makeAddr("relayer");

        // Features

        token = new WETH9();

        account1 = makeAddr("account1");
        account2 = makeAddr("account2");

        // create tokens for account 1
        hoax(account1);
        token.deposit{value: 500}();

        // create tokens for account 2
        token.deposit{value: 500}();

        recipientAddress32 = multiAddressFromBytes32(keccak256("recipient"));
        recipientAddress20 = multiAddressFromBytes20(bytes20(keccak256("recipient")));
    }

    function makeCreateAgentCommand() public pure returns (Command, bytes memory) {
        return (Command.CreateAgent, abi.encode((keccak256("6666"))));
    }

    function makeMockProof() public pure returns (Verification.Proof memory) {
        return Verification.Proof({
            header: Verification.ParachainHeader({
                parentHash: bytes32(0),
                number: 0,
                stateRoot: bytes32(0),
                extrinsicsRoot: bytes32(0),
                digestItems: new Verification.DigestItem[](0)
            }),
            headProof: Verification.HeadProof({pos: 0, width: 0, proof: new bytes32[](0)}),
            leafPartial: Verification.MMRLeafPartial({
                version: 0,
                parentNumber: 0,
                parentHash: bytes32(0),
                nextAuthoritySetID: 0,
                nextAuthoritySetLen: 0,
                nextAuthoritySetRoot: 0
            }),
            leafProof: new bytes32[](0),
            leafProofOrder: 0
        });
    }

    fallback() external payable {}
    receive() external payable {}

    /**
     * Message Verification
     */

    function testSubmitHappyPath() public {
        deal(assetHubAgent, 50 ether);

        (Command command, bytes memory params) = makeCreateAgentCommand();

        // Expect the gateway to emit `InboundMessageDispatched`
        vm.expectEmit(true, false, false, false);
        emit IGateway.InboundMessageDispatched(assetHubParaID.into(), 1, messageID, true);

        hoax(relayer, 1 ether);
        IGateway(address(gateway)).submitInbound(
            InboundMessage(assetHubParaID.into(), 1, command, params, maxDispatchGas, maxRefund, reward, messageID),
            proof,
            makeMockProof()
        );
    }

    function testSubmitFailInvalidNonce() public {
        deal(assetHubAgent, 50 ether);

        (Command command, bytes memory params) = makeCreateAgentCommand();

        hoax(relayer, 1 ether);
        IGateway(address(gateway)).submitInbound(
            InboundMessage(assetHubParaID.into(), 1, command, params, maxDispatchGas, maxRefund, reward, messageID),
            proof,
            makeMockProof()
        );

        // try to replay the message
        vm.expectRevert(Gateway.InvalidNonce.selector);
        hoax(relayer, 1 ether);
        IGateway(address(gateway)).submitInbound(
            InboundMessage(assetHubParaID.into(), 1, command, params, maxDispatchGas, maxRefund, reward, messageID),
            proof,
            makeMockProof()
        );
    }

    function testSubmitFailInvalidChannel() public {
        (Command command,) = makeCreateAgentCommand();

        vm.expectRevert(Gateway.ChannelDoesNotExist.selector);
        hoax(relayer);
        IGateway(address(gateway)).submitInbound(
            InboundMessage(ParaID.wrap(42).into(), 1, command, "", maxDispatchGas, maxRefund, reward, messageID),
            proof,
            makeMockProof()
        );
    }

    function testSubmitFailInvalidProof() public {
        deal(assetHubAgent, 50 ether);

        (Command command, bytes memory params) = makeCreateAgentCommand();

        GatewayMock(address(gateway)).setCommitmentsAreVerified(false);
        vm.expectRevert(Gateway.InvalidProof.selector);

        hoax(relayer, 1 ether);
        IGateway(address(gateway)).submitInbound(
            InboundMessage(assetHubParaID.into(), 1, command, params, maxDispatchGas, maxRefund, reward, messageID),
            proof,
            makeMockProof()
        );
    }

    /**
     * Fees & Rewards
     */

    // Message relayer should be rewarded from the agent for a channel
    function testRelayerRewardedFromAgent() public {
        (Command command, bytes memory params) = makeCreateAgentCommand();

        vm.txGasPrice(10 gwei);
        hoax(relayer, 1 ether);
        deal(assetHubAgent, 50 ether);

        uint256 relayerBalanceBefore = address(relayer).balance;
        uint256 agentBalanceBefore = address(assetHubAgent).balance;

        uint256 startGas = gasleft();
        IGateway(address(gateway)).submitInbound(
            InboundMessage(assetHubParaID.into(), 1, command, params, maxDispatchGas, maxRefund, reward, messageID),
            proof,
            makeMockProof()
        );
        uint256 endGas = gasleft();
        uint256 estimatedActualRefundAmount = (startGas - endGas) * tx.gasprice;
        assertLt(estimatedActualRefundAmount, maxRefund);

        // Check that agent balance decreased and relayer balance increases
        assertLt(address(assetHubAgent).balance, agentBalanceBefore);
        assertGt(relayer.balance, relayerBalanceBefore);

        // The total amount paid to the relayer
        uint256 totalPaid = agentBalanceBefore - address(assetHubAgent).balance;

        // Since we know that the actual refund amount is less than the max refund,
        // the total amount paid to the relayer is less.
        assertLt(totalPaid, maxRefund + reward);
    }

    // In this case, the agent has no funds to reward the relayer
    function testRelayerNotRewarded() public {
        (Command command, bytes memory params) = makeCreateAgentCommand();

        hoax(relayer, 1 ether);
        IGateway(address(gateway)).submitInbound(
            InboundMessage(assetHubParaID.into(), 1, command, params, maxDispatchGas, maxRefund, reward, messageID),
            proof,
            makeMockProof()
        );

        assertEq(address(assetHubAgent).balance, 0 ether);
        assertEq(relayer.balance, 1 ether);
    }

    // Users should pay fees to send outbound messages
    function testUserPaysFees() public {
        // Create a mock user
        address user = makeAddr("user");
        deal(address(token), user, 1);

        Fee memory fee = IGateway(address(gateway)).sendTokenFee(address(token), ParaID.wrap(0), 1);

        // Let gateway lock up to 1 tokens
        hoax(user);
        token.approve(address(gateway), 1);

        hoax(user, fee.total());
        IGateway(address(gateway)).sendToken{value: fee.total()}(
            address(token), ParaID.wrap(0), recipientAddress32, 1, 1
        );

        assertEq(user.balance, 0);
    }

    // User doesn't have enough funds to send message
    function testUserDoesNotProvideEnoughFees() public {
        // Create a mock user
        address user = makeAddr("user");
        deal(address(token), user, 1);

        // Let gateway lock up to 1 tokens
        hoax(user);
        token.approve(address(gateway), 1);

        vm.expectRevert(Gateway.FeePaymentToLow.selector);
        hoax(user, 2 ether);
        IGateway(address(gateway)).sendToken{value: 0.002 ether}(
            address(token), ParaID.wrap(0), recipientAddress32, 1, 1
        );

        assertEq(user.balance, 2 ether);
    }

    /**
     * Handlers
     */

    function testAgentExecution() public {
        token.transfer(address(assetHubAgent), 200);

        Gateway.AgentExecuteParams memory params = Gateway.AgentExecuteParams({
            agentID: assetHubAgentID,
            payload: abi.encode(AgentExecuteCommand.TransferToken, abi.encode(address(token), address(account2), 10))
        });

        bytes memory encodedParams = abi.encode(params);
        GatewayMock(address(gateway)).agentExecutePublic(encodedParams);
    }

    function testAgentExecutionBadOrigin() public {
        Gateway.AgentExecuteParams memory params = Gateway.AgentExecuteParams({
            agentID: bytes32(0),
            payload: abi.encode(keccak256("transferNativeToken"), abi.encode(address(token), address(this), 1))
        });

        vm.expectRevert(Gateway.AgentDoesNotExist.selector);
        GatewayMock(address(gateway)).agentExecutePublic(abi.encode(params));
    }

    function testAgentExecutionBadPayload() public {
        Gateway.AgentExecuteParams memory params = Gateway.AgentExecuteParams({agentID: assetHubAgentID, payload: ""});

        vm.expectRevert(Gateway.InvalidAgentExecutionPayload.selector);
        GatewayMock(address(gateway)).agentExecutePublic(abi.encode(params));
    }

    function testCreateAgent() public {
        bytes32 agentID = keccak256("123");
        Gateway.CreateAgentParams memory params = Gateway.CreateAgentParams({agentID: agentID});

        vm.expectEmit(false, false, false, false, address(gateway));
        emit IGateway.AgentCreated(agentID, address(0));

        GatewayMock(address(gateway)).createAgentPublic(abi.encode(params));
    }

    function testCreateAgentAlreadyCreated() public {
        bytes32 agentID = keccak256("123");
        Gateway.CreateAgentParams memory params = Gateway.CreateAgentParams({agentID: agentID});

        GatewayMock(address(gateway)).createAgentPublic(abi.encode(params));

        vm.expectRevert(Gateway.AgentAlreadyCreated.selector);
        GatewayMock(address(gateway)).createAgentPublic(abi.encode(params));
    }

    function testCreateChannel() public {
        ParaID paraID = ParaID.wrap(3042);
        bytes32 agentID = keccak256("3042");

        GatewayMock(address(gateway)).createAgentPublic(abi.encode(Gateway.CreateAgentParams({agentID: agentID})));

        Gateway.CreateChannelParams memory params = Gateway.CreateChannelParams({
            channelID: paraID.into(),
            agentID: agentID,
            mode: OperatingMode.Normal,
            outboundFee: outboundFee
        });

        vm.expectEmit(true, false, false, true);
        emit IGateway.ChannelCreated(paraID.into());
        GatewayMock(address(gateway)).createChannelPublic(abi.encode(params));
    }

    function testCreateChannelFailsAgentDoesNotExist() public {
        ParaID paraID = ParaID.wrap(3042);
        bytes32 agentID = keccak256("3042");

        Gateway.CreateChannelParams memory params = Gateway.CreateChannelParams({
            channelID: paraID.into(),
            mode: OperatingMode.Normal,
            agentID: agentID,
            outboundFee: 1 ether
        });

        vm.expectRevert(Gateway.AgentDoesNotExist.selector);
        GatewayMock(address(gateway)).createChannelPublic(abi.encode(params));
    }

    function testCreateChannelFailsChannelAlreadyExists() public {
        ParaID paraID = ParaID.wrap(3042);
        bytes32 agentID = keccak256("3042");

        GatewayMock(address(gateway)).createAgentPublic(abi.encode(Gateway.CreateAgentParams({agentID: agentID})));

        Gateway.CreateChannelParams memory params = Gateway.CreateChannelParams({
            channelID: paraID.into(),
            agentID: agentID,
            mode: OperatingMode.Normal,
            outboundFee: 1 ether
        });

        GatewayMock(address(gateway)).createChannelPublic(abi.encode(params));

        vm.expectRevert(Gateway.ChannelAlreadyCreated.selector);
        GatewayMock(address(gateway)).createChannelPublic(abi.encode(params));
    }

    function testUpdateChannel() public {
        // get current fee (0.0025 ether)
        uint256 fee = IGateway(address(gateway)).channelFeeOf(assetHubParaID.into());

        bytes memory params = abi.encode(
            Gateway.UpdateChannelParams({
                channelID: assetHubParaID.into(),
                mode: OperatingMode.RejectingOutboundMessages,
                fee: outboundFee * 1,
                exchangeRateNumerator: 1,
                exchangeRateDenominator: 800
            })
        );

        vm.expectEmit(true, false, false, true);
        emit IGateway.ChannelUpdated(assetHubParaID.into());
        GatewayMock(address(gateway)).updateChannelPublic(params);

        // Due to the new exchange rate, new fee is halved
        uint256 newFee = IGateway(address(gateway)).channelFeeOf(assetHubParaID.into());
        assertEq(fee / 2, newFee);
    }

    function testUpdateChannelFailDoesNotExist() public {
        bytes memory params = abi.encode(
            Gateway.UpdateChannelParams({
                channelID: ParaID.wrap(5956).into(),
                mode: OperatingMode.RejectingOutboundMessages,
                fee: outboundFee * 2,
                exchangeRateNumerator: 1,
                exchangeRateDenominator: 800
            })
        );

        vm.expectRevert(Gateway.ChannelDoesNotExist.selector);
        GatewayMock(address(gateway)).updateChannelPublic(params);
    }

    function testUpdateChannelSanityChecksForPrimaryGovernanceChannel() public {
        bytes memory params = abi.encode(
            Gateway.UpdateChannelParams({
                channelID: ChannelID.wrap(bytes32(uint256(1))),
                mode: OperatingMode.RejectingOutboundMessages,
                fee: outboundFee * 2,
                exchangeRateNumerator: 1,
                exchangeRateDenominator: 800
            })
        );

        vm.expectRevert(Gateway.InvalidChannelUpdate.selector);
        GatewayMock(address(gateway)).updateChannelPublic(params);
    }

    function testUpgrade() public {
        // Upgrade to this new logic contract
        GatewayV2 newLogic = new GatewayV2();

        Gateway.UpgradeParams memory params = Gateway.UpgradeParams({
            impl: address(newLogic),
            implCodeHash: address(newLogic).codehash,
            initParams: abi.encode(42)
        });

        // Expect the gateway to emit `Upgraded`
        vm.expectEmit(true, false, false, false);
        emit IGateway.Upgraded(address(newLogic));

        GatewayMock(address(gateway)).upgradePublic(abi.encode(params));

        // Verify that the GatewayV2.setup was called
        assertEq(GatewayV2(address(gateway)).getValue(), 42);
    }

    function testUpgradeGatewayMock() public {
        GatewayUpgradeMock newLogic = new GatewayUpgradeMock();
        uint256 d0 = 99;
        uint256 d1 = 66;
        bytes memory initParams = abi.encode(d0, d1);
        console.logBytes(initParams);

        Gateway.UpgradeParams memory params = Gateway.UpgradeParams({
            impl: address(newLogic),
            implCodeHash: address(newLogic).codehash,
            initParams: initParams
        });

        // Expect the gateway to emit `Initialized`
        vm.expectEmit(true, false, false, true);
        emit GatewayUpgradeMock.Initialized(d0, d1);

        GatewayMock(address(gateway)).upgradePublic(abi.encode(params));
    }

    function testUpgradeFailOnInitializationFailure() public {
        GatewayV2 newLogic = new GatewayV2();

        Gateway.UpgradeParams memory params = Gateway.UpgradeParams({
            impl: address(newLogic),
            implCodeHash: address(newLogic).codehash,
            initParams: abi.encode(666)
        });

        vm.expectRevert("initialize failed");
        GatewayMock(address(gateway)).upgradePublic(abi.encode(params));
    }

    function testUpgradeFailCodeHashMismatch() public {
        GatewayV2 newLogic = new GatewayV2();

        Gateway.UpgradeParams memory params =
            Gateway.UpgradeParams({impl: address(newLogic), implCodeHash: bytes32(0), initParams: abi.encode(42)});

        vm.expectRevert(Gateway.InvalidCodeHash.selector);
        GatewayMock(address(gateway)).upgradePublic(abi.encode(params));
    }

    function testSetOperatingMode() public {
        Gateway.SetOperatingModeParams memory params =
            Gateway.SetOperatingModeParams({mode: OperatingMode.RejectingOutboundMessages});

        OperatingMode mode = IGateway(address(gateway)).operatingMode();
        assertEq(uint256(mode), 0);

        GatewayMock(address(gateway)).setOperatingModePublic(abi.encode(params));

        mode = IGateway(address(gateway)).operatingMode();
        assertEq(uint256(mode), 1);
    }

    function testWithdrawAgentFunds() public {
        deal(assetHubAgent, 50 ether);

        address recipient = makeAddr("recipient");

        bytes memory params = abi.encode(
            Gateway.TransferNativeFromAgentParams({agentID: assetHubAgentID, recipient: recipient, amount: 3 ether})
        );

        GatewayMock(address(gateway)).transferNativeFromAgentPublic(params);

        assertEq(assetHubAgent.balance, 47 ether);
        assertEq(recipient.balance, 3 ether);
    }

    /**
     * Assets
     */

    function testRegisterToken() public {
        vm.expectEmit(false, false, false, true);
        emit IGateway.TokenRegistrationSent(address(token));

        vm.expectEmit(true, false, false, false);
        emit IGateway.OutboundMessageAccepted(assetHubParaID.into(), 1, messageID, bytes(""));

        IGateway(address(gateway)).registerToken{value: 2 ether}(address(token));
    }

    function testRegisterTokenReimbursesExcessFees() public {
        vm.expectEmit(false, false, false, true);
        emit IGateway.TokenRegistrationSent(address(token));

        vm.expectEmit(true, false, false, false);
        emit IGateway.OutboundMessageAccepted(assetHubParaID.into(), 1, messageID, bytes(""));

        uint256 totalFee =
            GatewayMock(address(gateway)).calculateLocalFeePublic(exchangeRate, outboundFee + registerTokenFee);

        uint256 balanceBefore = address(this).balance;
        IGateway(address(gateway)).registerToken{value: totalFee + 1 ether}(address(token));
        uint256 balanceAfter = address(this).balance;

        // Check that the balance has decreased by the amount of gas used
        // channel.fee is baseFee & extraFee is registerNativeTokenFee
        uint256 etherUsed = balanceBefore - balanceAfter;
        assert(etherUsed == totalFee);
    }

    function testSendTokenAddress32() public {
        // Let gateway lock up to 1 tokens
        token.approve(address(gateway), 1);

        // Multilocation for recipient
        ParaID destPara = ParaID.wrap(2043);

        Fee memory fee = IGateway(address(gateway)).sendTokenFee(address(token), destPara, 1);

        vm.expectEmit(true, true, false, true);
        emit IGateway.TokenSent(address(this), address(token), destPara, recipientAddress32, 1);

        // Expect the gateway to emit `OutboundMessageAccepted`
        vm.expectEmit(true, false, false, false);
        emit IGateway.OutboundMessageAccepted(assetHubParaID.into(), 1, messageID, bytes(""));

        IGateway(address(gateway)).sendToken{value: fee.total()}(address(token), destPara, recipientAddress32, 1, 1);
    }

    function testSendTokenAddress32ToAssetHub() public {
        // Let gateway lock up to 1 tokens
        token.approve(address(gateway), 1);

        // Multilocation for recipient
        ParaID destPara = assetHubParaID;

        Fee memory fee = IGateway(address(gateway)).sendTokenFee(address(token), destPara, 1);

        vm.expectEmit(true, true, false, true);
        emit IGateway.TokenSent(address(this), address(token), destPara, recipientAddress32, 1);

        // Expect the gateway to emit `OutboundMessageAccepted`
        vm.expectEmit(true, false, false, false);
        emit IGateway.OutboundMessageAccepted(assetHubParaID.into(), 1, messageID, bytes(""));

        IGateway(address(gateway)).sendToken{value: fee.total()}(address(token), destPara, recipientAddress32, 1, 1);
    }

    function testSendTokenAddress20() public {
        // Let gateway lock up to 1 tokens
        token.approve(address(gateway), 1);

        // Multilocation for recipient
        ParaID destPara = ParaID.wrap(2043);

        Fee memory fee = IGateway(address(gateway)).sendTokenFee(address(token), destPara, 1);

        vm.expectEmit(true, true, false, true);
        emit IGateway.TokenSent(address(this), address(token), destPara, recipientAddress20, 1);

        // Expect the gateway to emit `OutboundMessageAccepted`
        vm.expectEmit(true, false, false, false);
        emit IGateway.OutboundMessageAccepted(assetHubParaID.into(), 1, messageID, bytes(""));

        IGateway(address(gateway)).sendToken{value: fee.total()}(address(token), destPara, recipientAddress20, 1, 1);
    }

    function testSendTokenAddress20FailsInvalidDestination() public {
        // Let gateway lock up to 1 tokens
        token.approve(address(gateway), 1);

        ParaID destPara = assetHubParaID;

        // Should fail to send tokens to AssetHub
        vm.expectRevert(Assets.Unsupported.selector);
        IGateway(address(gateway)).sendToken{value: 2 ether}(address(token), destPara, recipientAddress20, 1, 1);
    }

    /**
     * Operating Modes
     */

    function testDisableOutboundMessaging() public {
        // Let gateway lock up to 1 tokens
        token.approve(address(gateway), 1);

        GatewayMock(address(gateway)).setOperatingModePublic(
            abi.encode(Gateway.SetOperatingModeParams({mode: OperatingMode.RejectingOutboundMessages}))
        );

        OperatingMode mode = IGateway(address(gateway)).operatingMode();
        assertEq(uint256(mode), 1);
    }

    function testDisableOutboundMessagingForChannel() public {
        // Let gateway lock up to 1 tokens
        token.approve(address(gateway), 1);

        GatewayMock(address(gateway)).setOperatingModePublic(
            abi.encode(Gateway.SetOperatingModeParams({mode: OperatingMode.Normal}))
        );

        bytes memory params = abi.encode(
            Gateway.UpdateChannelParams({
                channelID: assetHubParaID.into(),
                mode: OperatingMode.RejectingOutboundMessages,
                fee: outboundFee * 2,
                exchangeRateNumerator: 1,
                exchangeRateDenominator: 800
            })
        );
        GatewayMock(address(gateway)).updateChannelPublic(params);

        OperatingMode mode = IGateway(address(gateway)).channelOperatingModeOf(assetHubParaID.into());
        assertEq(uint256(mode), 1);

        // Now all outbound messaging should be disabled

        vm.expectRevert(Gateway.Disabled.selector);
        IGateway(address(gateway)).registerToken{value: 1 ether}(address(token));

        vm.expectRevert(Gateway.Disabled.selector);
        IGateway(address(gateway)).sendToken{value: 1 ether}(address(token), ParaID.wrap(0), recipientAddress32, 1, 1);
    }

    /**
     * Misc checks
     */

    // Initialize function should not be externally callable on either proxy or implementation contract
    function testInitializeNotExternallyCallable() public {
        vm.expectRevert(Gateway.Unauthorized.selector);
        Gateway(address(gateway)).initialize("");

        vm.expectRevert(Gateway.Unauthorized.selector);
        GatewayMock(address(gatewayLogic)).initialize("");
    }

    // Handler functions should not be externally callable
    function testHandlersNotExternallyCallable() public {
        vm.expectRevert(Gateway.Unauthorized.selector);
        Gateway(address(gateway)).agentExecute("");

        vm.expectRevert(Gateway.Unauthorized.selector);
        Gateway(address(gateway)).createAgent("");

        vm.expectRevert(Gateway.Unauthorized.selector);
        Gateway(address(gateway)).createChannel("");

        vm.expectRevert(Gateway.Unauthorized.selector);
        Gateway(address(gateway)).updateChannel("");

        vm.expectRevert(Gateway.Unauthorized.selector);
        Gateway(address(gateway)).setOperatingMode("");

        vm.expectRevert(Gateway.Unauthorized.selector);
        Gateway(address(gateway)).upgrade("");

        vm.expectRevert(Gateway.Unauthorized.selector);
        Gateway(address(gateway)).transferNativeFromAgent("");
    }

    function testGetters() public {
        IGateway gw = IGateway(address(gateway));

        OperatingMode mode = gw.operatingMode();
        assertEq(uint256(mode), 0);

        OperatingMode channelMode = gw.channelOperatingModeOf(assetHubParaID.into());
        assertEq(uint256(channelMode), 0);

        uint256 fee = gw.channelFeeOf(assetHubParaID.into());
        assertEq(fee, 0.0025 ether);

        (uint64 inbound, uint64 outbound) = gw.channelNoncesOf(assetHubParaID.into());
        assertEq(inbound, 0);
        assertEq(outbound, 0);

        address agent = gw.agentOf(assetHubAgentID);
        assertEq(agent, assetHubAgent);

        address implementation = gw.implementation();
        assertEq(implementation, address(gatewayLogic));
    }

    function testCreateAgentWithNotEnoughGas() public {
        deal(assetHubAgent, 50 ether);

        (Command command, bytes memory params) = makeCreateAgentCommand();

        hoax(relayer, 1 ether);

        vm.expectEmit(true, false, false, true);
        // Expect dispatch result as false for `OutOfGas`
        emit IGateway.InboundMessageDispatched(assetHubParaID.into(), 1, messageID, false);
        // maxDispatchGas as 1 for `create_agent` is definitely not enough
        IGateway(address(gateway)).submitInbound(
            InboundMessage(assetHubParaID.into(), 1, command, params, 1, maxRefund, reward, messageID),
            proof,
            makeMockProof()
        );
    }

    function testCalculateLocalFee() public {
        // 400 DOT = 1 ETH
        uint256 remoteFee = 400e10;
        uint256 localFee = GatewayMock(address(gateway)).calculateLocalFeePublic(exchangeRate, remoteFee);
        assertEq(localFee, 1 ether);

        // 1 DOT = 0.0025 ETH
        remoteFee = 1e10;
        localFee = GatewayMock(address(gateway)).calculateLocalFeePublic(exchangeRate, remoteFee);
        assertEq(localFee, 0.0025 ether);
    }

    function testSetTokenFees() public {
        // Double the fees

        Fee memory fee = IGateway(address(gateway)).registerTokenFee();
        assertEq(fee.xcm, 0.0025 ether);
        GatewayMock(address(gateway)).setTokenTransferFeesPublic(
            abi.encode(Gateway.SetTokenTransferFeesParams({register: registerTokenFee * 2, send: sendTokenFee * 2}))
        );
        fee = IGateway(address(gateway)).registerTokenFee();
        assertEq(fee.xcm, 0.005e18);
    }

    bytes32 public expectChannelIDBytes = bytes32(0xc173fac324158e77fb5840738a1a541f633cbec8884c6a601c567d2b376a0539);

    function testDeriveChannelID() public {
        ParaID para_id = ParaID.wrap(1000);
        ChannelID channel_id = para_id.into();
        assertEq(ChannelID.unwrap(channel_id), expectChannelIDBytes);
    }
}
