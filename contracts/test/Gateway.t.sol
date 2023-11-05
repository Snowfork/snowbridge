// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.22;

import {Test} from "forge-std/Test.sol";
import {Strings} from "openzeppelin/utils/Strings.sol";
import {console} from "forge-std/console.sol";

import {BeefyClient} from "../src/BeefyClient.sol";

import {IGateway} from "../src/interfaces/IGateway.sol";
import {IInitializable} from "../src/interfaces/IInitializable.sol";
import {Gateway} from "../src/Gateway.sol";
import {GatewayMock, GatewayV2} from "./mocks/GatewayMock.sol";

import {GatewayProxy} from "../src/GatewayProxy.sol";

import {AgentExecutor} from "../src/AgentExecutor.sol";
import {Agent} from "../src/Agent.sol";
import {Verification} from "../src/Verification.sol";
import {Assets} from "../src/Assets.sol";
import {SubstrateTypes} from "./../src/SubstrateTypes.sol";

import {NativeTransferFailed} from "../src/utils/SafeTransfer.sol";

import {AgentExecuteCommand, InboundMessage, OperatingMode, ParaID, Config, Command} from "../src/Types.sol";

import {WETH9} from "canonical-weth/WETH9.sol";
import "./mocks/GatewayUpgradeMock.sol";

contract GatewayTest is Test {
    event InboundMessageDispatched(ParaID indexed origin, uint64 nonce, bool result);
    event OutboundMessageAccepted(ParaID indexed dest, uint64 nonce, bytes payload);
    event NativeTokensUnlocked(address token, address recipient, uint256 amount);
    event TokenRegistrationSent(address token);
    event TokenSent(
        address indexed sender, address indexed token, ParaID destinationChain, bytes destinationAddress, uint128 amount
    );
    event AgentCreated(bytes32 agentID, address agent);
    event ChannelCreated(ParaID indexed paraID);
    event ChannelUpdated(ParaID indexed paraID);

    event Upgraded(address indexed implementation);
    event Initialized(uint256 d0, uint256 d1);

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

    uint256 public baseFee = 1 ether;
    uint256 public registerNativeTokenFee = 1 ether;
    uint256 public sendNativeTokenFee = 1 ether;

    function setUp() public {
        AgentExecutor executor = new AgentExecutor();
        gatewayLogic = new GatewayMock(
            address(0),
            address(executor),
            bridgeHubParaID,
            bridgeHubAgentID,
            assetHubParaID,
            assetHubAgentID
        );
        gateway = new GatewayProxy(
            address(gatewayLogic),
            abi.encode(
                baseFee,
                registerNativeTokenFee,
                sendNativeTokenFee
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
        deal(bridgeHubAgent, 50 ether);

        (Command command, bytes memory params) = makeCreateAgentCommand();

        // Expect the gateway to emit `InboundMessageDispatched`
        vm.expectEmit(true, false, false, false);
        emit IGateway.InboundMessageDispatched(bridgeHubParaID, 1, true);

        hoax(relayer, 1 ether);
        IGateway(address(gateway)).submitInbound(
            InboundMessage(bridgeHubParaID, 1, command, params, maxDispatchGas, maxRefund, reward),
            proof,
            makeMockProof()
        );
    }

    function testSubmitFailInvalidNonce() public {
        deal(bridgeHubAgent, 50 ether);

        (Command command, bytes memory params) = makeCreateAgentCommand();

        hoax(relayer, 1 ether);
        IGateway(address(gateway)).submitInbound(
            InboundMessage(bridgeHubParaID, 1, command, params, maxDispatchGas, maxRefund, reward),
            proof,
            makeMockProof()
        );

        // try to replay the message
        vm.expectRevert(Gateway.InvalidNonce.selector);
        hoax(relayer, 1 ether);
        IGateway(address(gateway)).submitInbound(
            InboundMessage(bridgeHubParaID, 1, command, params, maxDispatchGas, maxRefund, reward),
            proof,
            makeMockProof()
        );
    }

    function testSubmitFailInvalidChannel() public {
        (Command command,) = makeCreateAgentCommand();

        vm.expectRevert(Gateway.ChannelDoesNotExist.selector);
        hoax(relayer);
        IGateway(address(gateway)).submitInbound(
            InboundMessage(ParaID.wrap(42), 1, command, "", maxDispatchGas, maxRefund, reward), proof, makeMockProof()
        );
    }

    function testSubmitFailInvalidProof() public {
        deal(bridgeHubAgent, 50 ether);

        (Command command, bytes memory params) = makeCreateAgentCommand();

        GatewayMock(address(gateway)).setCommitmentsAreVerified(false);
        vm.expectRevert(Gateway.InvalidProof.selector);

        hoax(relayer, 1 ether);
        IGateway(address(gateway)).submitInbound(
            InboundMessage(bridgeHubParaID, 1, command, params, maxDispatchGas, maxRefund, reward),
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
        deal(bridgeHubAgent, 50 ether);

        uint256 relayerBalanceBefore = address(relayer).balance;
        uint256 agentBalanceBefore = address(bridgeHubAgent).balance;

        uint256 startGas = gasleft();
        IGateway(address(gateway)).submitInbound(
            InboundMessage(bridgeHubParaID, 1, command, params, maxDispatchGas, maxRefund, reward),
            proof,
            makeMockProof()
        );
        uint256 endGas = gasleft();
        uint256 estimatedActualRefundAmount = (startGas - endGas) * tx.gasprice;
        assertLt(estimatedActualRefundAmount, maxRefund);

        // Check that agent balance decreased and relayer balance increases
        assertLt(address(bridgeHubAgent).balance, agentBalanceBefore);
        assertGt(relayer.balance, relayerBalanceBefore);

        // The total amount paid to the relayer
        uint256 totalPaid = agentBalanceBefore - address(bridgeHubAgent).balance;

        // Since we know that the actual refund amount is less than the max refund,
        // the total amount paid to the relayer is less.
        assertLt(totalPaid, maxRefund + reward);
    }

    // In this case, the agent has no funds to reward the relayer
    function testRelayerNotRewarded() public {
        (Command command, bytes memory params) = makeCreateAgentCommand();

        hoax(relayer, 1 ether);
        IGateway(address(gateway)).submitInbound(
            InboundMessage(bridgeHubParaID, 1, command, params, maxDispatchGas, maxRefund, reward),
            proof,
            makeMockProof()
        );

        assertEq(address(bridgeHubAgent).balance, 0 ether);
        assertEq(relayer.balance, 1 ether);
    }

    // Users should pay fees to send outbound messages
    function testUserPaysFees() public {
        // Create a mock user
        address user = makeAddr("user");
        deal(address(token), user, 1);

        // Let gateway lock up to 1 tokens
        hoax(user);
        token.approve(address(gateway), 1);

        hoax(user, 2 ether);
        IGateway(address(gateway)).sendToken{value: 2 ether}(address(token), ParaID.wrap(0), "", 1);

        assertEq(user.balance, 0 ether);
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
        IGateway(address(gateway)).sendToken{value: 0.5 ether}(address(token), ParaID.wrap(0), "", 1);

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

        Gateway.CreateChannelParams memory params = Gateway.CreateChannelParams({paraID: paraID, agentID: agentID});

        vm.expectEmit(true, false, false, true);
        emit IGateway.ChannelCreated(paraID);
        GatewayMock(address(gateway)).createChannelPublic(abi.encode(params));
    }

    function testCreateChannelFailsAgentDoesNotExist() public {
        ParaID paraID = ParaID.wrap(3042);
        bytes32 agentID = keccak256("3042");

        Gateway.CreateChannelParams memory params = Gateway.CreateChannelParams({paraID: paraID, agentID: agentID});

        vm.expectRevert(Gateway.AgentDoesNotExist.selector);
        GatewayMock(address(gateway)).createChannelPublic(abi.encode(params));
    }

    function testCreateChannelFailsChannelAlreadyExists() public {
        ParaID paraID = ParaID.wrap(3042);
        bytes32 agentID = keccak256("3042");

        GatewayMock(address(gateway)).createAgentPublic(abi.encode(Gateway.CreateAgentParams({agentID: agentID})));

        Gateway.CreateChannelParams memory params = Gateway.CreateChannelParams({paraID: paraID, agentID: agentID});

        GatewayMock(address(gateway)).createChannelPublic(abi.encode(params));

        vm.expectRevert(Gateway.ChannelAlreadyCreated.selector);
        GatewayMock(address(gateway)).createChannelPublic(abi.encode(params));
    }

    function testUpdateChannel() public {
        bytes memory params = abi.encode(
            Gateway.UpdateChannelParams({
                paraID: assetHubParaID,
                mode: OperatingMode.RejectingOutboundMessages,
                fee: 2 ether,
                reward: 2 ether
            })
        );

        vm.expectEmit(true, false, false, true);
        emit IGateway.ChannelUpdated(assetHubParaID);
        GatewayMock(address(gateway)).updateChannelPublic(params);

        uint256 fee = IGateway(address(gateway)).channelFeeOf(assetHubParaID);
        assertEq(fee, 2 ether);
    }

    function testUpdateChannelFailDoesNotExist() public {
        bytes memory params = abi.encode(
            Gateway.UpdateChannelParams({
                paraID: ParaID.wrap(5956),
                mode: OperatingMode.RejectingOutboundMessages,
                fee: 2 ether,
                reward: 2 ether
            })
        );

        vm.expectRevert(Gateway.ChannelDoesNotExist.selector);
        GatewayMock(address(gateway)).updateChannelPublic(params);
    }

    function testUpdateChannelSanityChecksForBridgeHubChannel() public {
        bytes memory params = abi.encode(
            Gateway.UpdateChannelParams({
                paraID: bridgeHubParaID,
                mode: OperatingMode.Normal,
                fee: 100000000 ether,
                reward: 100000000 ether
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

        vm.expectRevert(IInitializable.InitializationFailed.selector);
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
        emit TokenRegistrationSent(address(token));

        vm.expectEmit(true, false, false, false);
        emit IGateway.OutboundMessageAccepted(assetHubParaID, 1, SubstrateTypes.RegisterToken(address(token)));

        IGateway(address(gateway)).registerToken{value: 2 ether}(address(token));
    }

    function testRegisterTokenReimbursesExcessFees() public {
        vm.expectEmit(false, false, false, true);
        emit IGateway.TokenRegistrationSent(address(token));

        vm.expectEmit(true, false, false, false);
        emit IGateway.OutboundMessageAccepted(assetHubParaID, 1, SubstrateTypes.RegisterToken(address(token)));

        uint256 totalFee = baseFee + registerNativeTokenFee;
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
        bytes32 destAddress = keccak256("/Alice");

        vm.expectEmit(true, true, false, true);
        emit IGateway.TokenSent(address(this), address(token), destPara, abi.encodePacked(destAddress), 1);

        // Expect the gateway to emit `OutboundMessageAccepted`
        vm.expectEmit(true, false, false, false);
        emit IGateway.OutboundMessageAccepted(
            assetHubParaID, 1, SubstrateTypes.SendToken(address(token), destPara, destAddress, 1)
        );

        IGateway(address(gateway)).sendToken{value: 2 ether}(address(token), destPara, destAddress, 1);
    }

    function testSendTokenAddress32ToAssetHub() public {
        // Let gateway lock up to 1 tokens
        token.approve(address(gateway), 1);

        // Multilocation for recipient
        ParaID destPara = assetHubParaID;
        bytes32 destAddress = keccak256("/Alice");

        vm.expectEmit(true, true, false, true);
        emit IGateway.TokenSent(address(this), address(token), destPara, abi.encodePacked(destAddress), 1);

        // Expect the gateway to emit `OutboundMessageAccepted`
        vm.expectEmit(true, false, false, false);
        emit IGateway.OutboundMessageAccepted(
            assetHubParaID, 1, SubstrateTypes.SendToken(address(token), destAddress, 1)
        );

        IGateway(address(gateway)).sendToken{value: 2 ether}(address(token), destPara, destAddress, 1);
    }

    function testSendTokenAddress20() public {
        // Let gateway lock up to 1 tokens
        token.approve(address(gateway), 1);

        // Multilocation for recipient
        ParaID destPara = ParaID.wrap(2043);
        address destAddress = makeAddr("/Alice");

        vm.expectEmit(true, true, false, true);
        emit IGateway.TokenSent(address(this), address(token), destPara, abi.encodePacked(destAddress), 1);

        // Expect the gateway to emit `OutboundMessageAccepted`
        vm.expectEmit(true, false, false, false);
        emit IGateway.OutboundMessageAccepted(assetHubParaID, 1, hex"");

        IGateway(address(gateway)).sendToken{value: 2 ether}(address(token), destPara, destAddress, 1);
    }

    function testSendTokenAddress20FailsInvalidDestination() public {
        // Let gateway lock up to 1 tokens
        token.approve(address(gateway), 1);

        ParaID destPara = assetHubParaID;
        address destAddress = makeAddr("/Alice");

        // Should fail to send tokens to AssetHub
        vm.expectRevert(Assets.InvalidDestination.selector);
        IGateway(address(gateway)).sendToken{value: 2 ether}(address(token), destPara, destAddress, 1);
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
                paraID: assetHubParaID,
                mode: OperatingMode.RejectingOutboundMessages,
                fee: 1 ether,
                reward: 1 ether
            })
        );
        GatewayMock(address(gateway)).updateChannelPublic(params);

        OperatingMode mode = IGateway(address(gateway)).channelOperatingModeOf(assetHubParaID);
        assertEq(uint256(mode), 1);

        // Now all outbound messaging should be disabled

        vm.expectRevert(Gateway.Disabled.selector);
        IGateway(address(gateway)).registerToken{value: 1 ether}(address(token));

        vm.expectRevert(Gateway.Disabled.selector);
        IGateway(address(gateway)).sendToken{value: 1 ether}(address(token), ParaID.wrap(0), "", 1);
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

        OperatingMode channelMode = gw.channelOperatingModeOf(bridgeHubParaID);
        assertEq(uint256(channelMode), 0);

        (uint256 fee) = gw.channelFeeOf(bridgeHubParaID);
        assertEq(fee, 1 ether);

        (uint64 inbound, uint64 outbound) = gw.channelNoncesOf(bridgeHubParaID);
        assertEq(inbound, 0);
        assertEq(outbound, 0);

        address agent = gw.agentOf(bridgeHubAgentID);
        assertEq(agent, bridgeHubAgent);

        address implementation = gw.implementation();
        assertEq(implementation, address(gatewayLogic));
    }

    function testCreateAgentWithNotEnoughGas() public {
        deal(bridgeHubAgent, 50 ether);

        (Command command, bytes memory params) = makeCreateAgentCommand();

        hoax(relayer, 1 ether);

        vm.expectEmit(true, false, false, true);
        // Expect dispatch result as false for `OutOfGas`
        emit IGateway.InboundMessageDispatched(bridgeHubParaID, 1, false);
        // maxDispatchGas as 1 for `create_agent` is definitely not enough
        IGateway(address(gateway)).submitInbound(
            InboundMessage(bridgeHubParaID, 1, command, params, 1, maxRefund, reward), proof, makeMockProof()
        );
    }

    function testSetTokenFees() public {
        GatewayMock(address(gateway)).setTokenTransferFeesPublic(
            abi.encode(Gateway.SetTokenTransferFeesParams({register: 1, send: 1}))
        );
        (uint256 register, uint256 send) = IGateway(address(gateway)).tokenTransferFees();
        assertEq(register, 1);
        assertEq(send, 1);
    }
}
