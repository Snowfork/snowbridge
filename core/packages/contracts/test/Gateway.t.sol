// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.20;

import {Strings} from "openzeppelin/utils/Strings.sol";
import {Test} from "forge-std/Test.sol";
import {console} from "forge-std/console.sol";

import {ParachainClientMock} from "./mocks/ParachainClientMock.sol";
import {IParachainClient} from "../src/IParachainClient.sol";
import {BeefyClient} from "../src/BeefyClient.sol";

import {IGateway} from "../src/IGateway.sol";
import {Gateway} from "../src/Gateway.sol";
import {GatewayMock, GatewayV2} from "./mocks/GatewayMock.sol";
import {AgentExecutorMock} from "./mocks/AgentExecutorMock.sol";

import {GatewayProxy} from "../src/GatewayProxy.sol";

import {AgentExecutor} from "../src/AgentExecutor.sol";
import {Agent} from "../src/Agent.sol";
import {InboundMessage, OperatingMode, ParaID} from "../src/Types.sol";

import {WETH9} from "canonical-weth/WETH9.sol";

contract GatewayTest is Test {
    event InboundMessageDispatched(ParaID indexed origin, uint64 nonce, bool result);
    event OutboundMessageAccepted(ParaID indexed dest, uint64 nonce, bytes payload);
    event NativeTokensUnlocked(address token, address recipient, uint256 amount);
    event NativeTokensLocked(address token, ParaID destParaID, bytes recipient, uint128 amount);
    event AgentCreated(bytes32 agentID, address agent);
    event Upgraded(address indexed implementation);

    ParaID bridgeHubParaID = ParaID.wrap(1001);
    bytes32 public bridgeHubAgentID = keccak256("1001");
    address public bridgeHubAgent;

    ParaID assetHubParaID = ParaID.wrap(1002);
    bytes32 public assetHubAgentID = keccak256("1002");
    address public assetHubAgent;

    address relayer;

    bytes32[] public proof = [bytes32(0x2f9ee6cfdf244060dc28aa46347c5219e303fc95062dd672b4e406ca5c29764b)];
    bytes public parachainHeaderProof = bytes("validProof");

    GatewayMock gatewayLogic;
    GatewayProxy public gateway;

    WETH9 public token;

    address public account1;
    address public account2;

    IParachainClient public parachainClient;

    function setUp() public {
        parachainClient = new ParachainClientMock(BeefyClient(address(0)), 0);

        AgentExecutor executor = new AgentExecutor();

        Gateway.InitParams memory initParams = Gateway.InitParams({
            parachainClient: parachainClient,
            agentExecutor: address(executor),
            fee: 1 ether,
            reward: 1 ether,
            bridgeHubParaID: bridgeHubParaID,
            bridgeHubAgentID: bridgeHubAgentID,
            assetHubParaID: ParaID.wrap(1002),
            assetHubAgentID: keccak256("1002"),
            createTokenFee: 1,
            createTokenCallId: bytes2(0x3500),
            gasToForward: 500_000
        });

        gatewayLogic = new GatewayMock();
        gateway = new GatewayProxy(address(gatewayLogic), abi.encodeCall(Gateway.initialize, (initParams)));

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

    function makeCreateAgentCommand() public pure returns (bytes32, bytes memory) {
        return (keccak256("createAgent"), abi.encode((keccak256("6666"))));
    }

    /**
     * Message Verification
     */

    function testSubmitHappyPath() public {
        deal(bridgeHubAgent, 50 ether);

        (bytes32 command, bytes memory params) = makeCreateAgentCommand();

        // Expect the gateway to emit `InboundMessageDispatched`
        vm.expectEmit(true, false, false, false);
        emit InboundMessageDispatched(bridgeHubParaID, 1, true);

        hoax(relayer, 1 ether);
        IGateway(address(gateway)).submitInbound(
            InboundMessage(bridgeHubParaID, 1, command, params), proof, parachainHeaderProof
        );
    }

    function testSubmitFailInvalidProof() public {
        deal(bridgeHubAgent, 50 ether);

        (bytes32 command, bytes memory params) = makeCreateAgentCommand();

        vm.expectRevert(Gateway.InvalidProof.selector);

        hoax(relayer, 1 ether);
        IGateway(address(gateway)).submitInbound(
            InboundMessage(bridgeHubParaID, 1, command, params), proof, bytes("invalidProof")
        );
    }

    function testSubmitFailInvalidNonce() public {
        deal(bridgeHubAgent, 50 ether);

        (bytes32 command, bytes memory params) = makeCreateAgentCommand();

        hoax(relayer, 1 ether);
        IGateway(address(gateway)).submitInbound(
            InboundMessage(bridgeHubParaID, 1, command, params), proof, parachainHeaderProof
        );

        // try to replay the message
        vm.expectRevert(Gateway.InvalidNonce.selector);
        hoax(relayer, 1 ether);
        IGateway(address(gateway)).submitInbound(
            InboundMessage(bridgeHubParaID, 1, command, params), proof, parachainHeaderProof
        );
    }

    function testSubmitFailInvalidChannel() public {
        vm.expectRevert(Gateway.ChannelDoesNotExist.selector);
        hoax(relayer);
        IGateway(address(gateway)).submitInbound(
            InboundMessage(ParaID.wrap(42), 1, "", ""), proof, parachainHeaderProof
        );
    }

    // Handling of Out-of-Gas errors

    // Run with forge test -vvvv to verify that a nested call reverts with `EvmError: OutOfGas`
    function testSubmitSucceedsWhenHandlerOOG() public {
        deal(assetHubAgent, 50 ether);

        GatewayMock(address(gateway)).setAgentExecutor(address(new AgentExecutorMock()));

        hoax(relayer, 1 ether);
        IGateway(address(gateway)).submitInbound(
            InboundMessage(assetHubParaID, 1, keccak256("agentExecute"), abi.encode(assetHubAgentID, bytes("foo..."))),
            proof,
            parachainHeaderProof
        );
    }

    /**
     * Fees & Rewards
     */

    // Message relayer should be rewarded from the agent for a channel
    function testRelayerRewardedFromAgent() public {
        deal(bridgeHubAgent, 50 ether);

        (bytes32 command, bytes memory params) = makeCreateAgentCommand();

        hoax(relayer, 1 ether);
        IGateway(address(gateway)).submitInbound(
            InboundMessage(bridgeHubParaID, 1, command, params), proof, parachainHeaderProof
        );

        assertEq(address(bridgeHubAgent).balance, 49 ether);
        assertEq(relayer.balance, 2 ether);
    }

    // In this case, the agent has no funds to reward the relayer
    function testRelayerNotRewarded() public {
        (bytes32 command, bytes memory params) = makeCreateAgentCommand();

        vm.expectRevert(Agent.InsufficientBalance.selector);
        hoax(relayer, 1 ether);
        IGateway(address(gateway)).submitInbound(
            InboundMessage(bridgeHubParaID, 1, command, params), proof, parachainHeaderProof
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
        IGateway(address(gateway)).lockNativeTokens{value: 1 ether}(address(token), ParaID.wrap(0), "", 1);

        assertEq(user.balance, 1 ether);
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
        IGateway(address(gateway)).lockNativeTokens{value: 0.5 ether}(address(token), ParaID.wrap(0), "", 1);

        assertEq(user.balance, 2 ether);
    }

    /**
     * Handlers
     */

    function testHandlerAgentExecution() public {
        // first lock tokens so we can call unlockTokens later
        testFeatureLockTokens();

        bytes memory params = abi.encode(
            assetHubAgentID, abi.encode(keccak256("unlockTokens"), abi.encode(address(token), address(this), 1))
        );

        vm.expectEmit(false, false, false, true, assetHubAgent);
        emit NativeTokensUnlocked(address(token), address(this), 1);

        GatewayMock(address(gateway)).handleAgentExecutePublic(params);
    }

    function testHandlerAgentExecutionBadOrigin() public {
        bytes memory params = abi.encode(
            keccak256("foo"), abi.encode(keccak256("unlockTokens"), abi.encode(address(token), address(this), 1))
        );

        vm.expectRevert(Gateway.AgentDoesNotExist.selector);
        GatewayMock(address(gateway)).handleAgentExecutePublic(params);
    }

    function testHandlerAgentExecutionBadPayload() public {
        bytes memory params = abi.encode(assetHubAgentID, hex"");

        vm.expectRevert(Gateway.InvalidAgentExecutionPayload.selector);
        GatewayMock(address(gateway)).handleAgentExecutePublic(params);
    }

    function testHandlerCreateAgent() public {
        bytes32 agentID = keccak256("123");
        bytes memory params = abi.encode((agentID));

        vm.expectEmit(false, false, false, false, address(gateway));
        emit AgentCreated(agentID, address(0));

        GatewayMock(address(gateway)).handleCreateAgentPublic(params);
    }

    function testHandlerCreateAgentAlreadyCreated() public {
        bytes32 agentID = keccak256("123");
        bytes memory params = abi.encode((agentID));

        GatewayMock(address(gateway)).handleCreateAgentPublic(params);

        vm.expectRevert(Gateway.AgentAlreadyCreated.selector);
        GatewayMock(address(gateway)).handleCreateAgentPublic(params);
    }

    function testHandlerUpgrade() public {
        // Upgrade to this new logic contract
        Gateway newLogic = new GatewayV2();
        bytes memory params = abi.encode(address(newLogic), abi.encodeCall(GatewayV2.initializeV2, ()));

        // Expect the gateway to emit `Upgraded`
        vm.expectEmit(true, false, false, false);
        emit Upgraded(address(newLogic));

        GatewayMock(address(gateway)).handleUpgradePublic(params);

        // Verify that the GatewayV2.initializeV2 was called
        assertEq(GatewayV2(address(gateway)).getValue(), 42);
    }

    function testHandlerUpgradeFail() public {
        bytes memory params = abi.encode(address(1), hex"");

        // Upgrade should fail if a bad address is passed
        vm.expectRevert("ERC1967: new implementation is not a contract");
        GatewayMock(address(gateway)).handleUpgradePublic(params);
    }

    function testHandlerSetOperatingMode() public {
        bytes memory params = abi.encode((OperatingMode.RejectingOutboundMessages));

        OperatingMode mode = IGateway(address(gateway)).operatingMode();
        assertEq(uint256(mode), 0);

        GatewayMock(address(gateway)).handleSetOperatingModePublic(params);

        mode = IGateway(address(gateway)).operatingMode();
        assertEq(uint256(mode), 1);
    }

    /**
     * Misc checks
     */

    // Only cross-chain governance can initiate upgrades
    function testUpgradeIsPrivileged() public {
        vm.expectRevert(Gateway.Unauthorized.selector);
        Gateway(address(gateway)).upgradeTo(address(gatewayLogic));
    }

    // Handler functions should not be externally callable
    function testHandlersArePrivileged() public {
        vm.expectRevert(Gateway.Unauthorized.selector);
        Gateway(address(gateway)).handleAgentExecute("");

        vm.expectRevert(Gateway.Unauthorized.selector);
        Gateway(address(gateway)).handleCreateAgent("");

        vm.expectRevert(Gateway.Unauthorized.selector);
        Gateway(address(gateway)).handleCreateChannel("");

        vm.expectRevert(Gateway.Unauthorized.selector);
        Gateway(address(gateway)).handleUpdateChannel("");

        vm.expectRevert(Gateway.Unauthorized.selector);
        Gateway(address(gateway)).handleSetOperatingMode("");

        vm.expectRevert(Gateway.Unauthorized.selector);
        Gateway(address(gateway)).handleUpgrade("");

        vm.expectRevert(Gateway.Unauthorized.selector);
        Gateway(address(gateway)).handleWithdrawSovereignFunds("");

        vm.expectRevert(Gateway.Unauthorized.selector);
        Gateway(address(gateway)).handleConfigure("");
    }

    function testGetters() public {
        IGateway gw = IGateway(address(gateway));

        OperatingMode mode = gw.operatingMode();
        assertEq(uint256(mode), 0);

        OperatingMode channelMode = gw.channelOperatingModeOf(bridgeHubParaID);
        assertEq(uint256(channelMode), 0);

        (uint256 fee, uint256 reward) = gw.channelFeeRewardOf(bridgeHubParaID);
        assertEq(fee, 1 ether);
        assertEq(reward, 1 ether);

        (uint64 inbound, uint64 outbound) = gw.channelNoncesOf(bridgeHubParaID);
        assertEq(inbound, 0);
        assertEq(outbound, 0);

        address agent = gw.agentOf(bridgeHubAgentID);
        assertEq(agent, bridgeHubAgent);

        address pc = gw.parachainClient();
        assertEq(pc, address(parachainClient));
    }

    /**
     * Features
     */

    function testFeatureLockTokens() public {
        // Let gateway lock up to 1 tokens
        token.approve(address(gateway), 1);

        // Multilocation for recipient
        bytes memory recipient = "/Alice";

        vm.expectEmit();
        emit NativeTokensLocked(address(token), ParaID.wrap(0), recipient, 1);

        // Expect the gateway to emit `OutboundMessageAccepted`
        vm.expectEmit(true, false, false, false);
        emit OutboundMessageAccepted(assetHubParaID, 1, hex"");

        IGateway(address(gateway)).lockNativeTokens{value: 1 ether}(address(token), ParaID.wrap(0), recipient, 1);
    }

    /**
     * Operating Modes
     */

    function testDisableOutboundMessaging() public {
        // Let gateway lock up to 1 tokens
        token.approve(address(gateway), 1);

        GatewayMock(address(gateway)).setOperatingMode(OperatingMode.RejectingOutboundMessages);

        OperatingMode mode = IGateway(address(gateway)).operatingMode();
        assertEq(uint256(mode), 1);

        // Now all outbound messaging should be disabled

        vm.expectRevert(Gateway.Disabled.selector);
        IGateway(address(gateway)).registerNativeToken{value: 1 ether}(address(token));

        vm.expectRevert(Gateway.Disabled.selector);
        IGateway(address(gateway)).lockNativeTokens{value: 1 ether}(address(token), ParaID.wrap(0), "", 1);
    }

    function testDisableOutboundMessagingForChannel() public {
        // Let gateway lock up to 1 tokens
        token.approve(address(gateway), 1);

        GatewayMock(address(gateway)).setOperatingMode(OperatingMode.Normal);
        GatewayMock(address(gateway)).setChannelOperatingMode(assetHubParaID, OperatingMode.RejectingOutboundMessages);

        OperatingMode mode = IGateway(address(gateway)).channelOperatingModeOf(assetHubParaID);
        assertEq(uint256(mode), 1);

        // Now all outbound messaging should be disabled

        vm.expectRevert(Gateway.Disabled.selector);
        IGateway(address(gateway)).registerNativeToken{value: 1 ether}(address(token));

        vm.expectRevert(Gateway.Disabled.selector);
        IGateway(address(gateway)).lockNativeTokens{value: 1 ether}(address(token), ParaID.wrap(0), "", 1);
    }
}
