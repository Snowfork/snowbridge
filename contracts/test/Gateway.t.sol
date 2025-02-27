// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import {Test} from "forge-std/Test.sol";
import {Strings} from "openzeppelin/utils/Strings.sol";
import {console} from "forge-std/console.sol";

import {BeefyClient} from "../src/BeefyClient.sol";

import {IGateway} from "../src/interfaces/IGateway.sol";
import {IInitializable} from "../src/interfaces/IInitializable.sol";
import {IUpgradable} from "../src/interfaces/IUpgradable.sol";
import {Gateway} from "../src/Gateway.sol";
import {MockGateway} from "./mocks/MockGateway.sol";
import {MockGatewayV2} from "./mocks/MockGatewayV2.sol";
import {ReantrantAttacker} from "./mocks/ReantrantAttacker.sol";
import {GatewayProxy} from "../src/GatewayProxy.sol";

import {AgentExecutor} from "../src/AgentExecutor.sol";
import {Agent} from "../src/Agent.sol";
import {Verification} from "../src/Verification.sol";
import {Assets} from "../src/Assets.sol";
import {SubstrateTypes} from "./../src/SubstrateTypes.sol";
import {MultiAddress} from "../src/MultiAddress.sol";
import {Channel, InboundMessage, OperatingMode, ParaID, Command, ChannelID, MultiAddress} from "../src/Types.sol";

import {NativeTransferFailed, SafeNativeTransfer} from "../src/utils/SafeTransfer.sol";
import {PricingStorage} from "../src/storage/PricingStorage.sol";
import {IERC20} from "../src/interfaces/IERC20.sol";
import {TokenLib} from "../src/TokenLib.sol";
import {Token} from "../src/Token.sol";

import {
    UpgradeParams,
    CreateAgentParams,
    AgentExecuteParams,
    CreateChannelParams,
    UpdateChannelParams,
    SetOperatingModeParams,
    TransferNativeFromAgentParams,
    SetTokenTransferFeesParams,
    SetPricingParametersParams,
    RegisterForeignTokenParams,
    TransferNativeTokenParams,
    MintForeignTokenParams
} from "../src/Params.sol";

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
import {UD60x18, ud60x18, convert} from "prb/math/src/UD60x18.sol";

contract GatewayTest is Test {
    // Emitted when token minted/burnt/transfered
    event Transfer(address indexed from, address indexed to, uint256 value);

    ParaID public bridgeHubParaID = ParaID.wrap(1013);
    bytes32 public bridgeHubAgentID = 0xbc0eb42478cf1d8f1542c986e5e6a513f926c4552d7b7ab6084b97e192c622fa;
    address public bridgeHubAgent;

    ParaID public assetHubParaID = ParaID.wrap(1000);
    bytes32 public assetHubAgentID = 0xc173fac324158e77fb5840738a1a541f633cbec8884c6a601c567d2b376a0539;
    address public assetHubAgent;

    address public relayer;

    bytes32[] public proof = [bytes32(0x2f9ee6cfdf244060dc28aa46347c5219e303fc95062dd672b4e406ca5c29764b)];
    bytes public parachainHeaderProof = bytes("validProof");

    MockGateway public gatewayLogic;
    GatewayProxy public gateway;

    WETH9 public token;

    address public account1;
    address public account2;

    uint64 public maxDispatchGas = 500_000;
    uint256 public maxRefund = 1 ether;
    uint256 public reward = 1 ether;
    bytes32 public messageID = keccak256("cabbage");

    // remote fees in DOT
    uint128 public outboundFee = 1e10;
    uint128 public registerTokenFee = 0;
    uint128 public sendTokenFee = 1e10;
    uint128 public createTokenFee = 1e10;
    uint128 public maxDestinationFee = 1e11;

    MultiAddress public recipientAddress32;
    MultiAddress public recipientAddress20;

    // For DOT
    uint8 public foreignTokenDecimals = 10;

    // ETH/DOT exchange rate
    UD60x18 public exchangeRate = ud60x18(0.0025e18);
    UD60x18 public multiplier = ud60x18(1e18);

    // tokenID for DOT
    bytes32 public dotTokenID;

    function setUp() public {
        AgentExecutor executor = new AgentExecutor();
        gatewayLogic = new MockGateway(
            address(0), address(executor), bridgeHubParaID, bridgeHubAgentID, foreignTokenDecimals, maxDestinationFee
        );
        Gateway.Config memory config = Gateway.Config({
            mode: OperatingMode.Normal,
            deliveryCost: outboundFee,
            registerTokenFee: registerTokenFee,
            assetHubParaID: assetHubParaID,
            assetHubAgentID: assetHubAgentID,
            assetHubCreateAssetFee: createTokenFee,
            assetHubReserveTransferFee: sendTokenFee,
            exchangeRate: exchangeRate,
            multiplier: multiplier,
            rescueOperator: 0x4B8a782D4F03ffcB7CE1e95C5cfe5BFCb2C8e967
        });
        gateway = new GatewayProxy(address(gatewayLogic), abi.encode(config));
        MockGateway(address(gateway)).setCommitmentsAreVerified(true);

        SetOperatingModeParams memory params = SetOperatingModeParams({mode: OperatingMode.Normal});
        MockGateway(address(gateway)).setOperatingModePublic(abi.encode(params));

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

        dotTokenID = bytes32(uint256(1));
    }

    function makeCreateAgentCommand() public pure returns (Command, bytes memory) {
        return (Command.CreateAgent, abi.encode((keccak256("6666"))));
    }

    function makeLegacyUnlockTokenCommand(bytes32 agentID, address token_, address recipient, uint128 amount)
        public
        pure
        returns (Command, bytes memory)
    {
        bytes memory payload = abi.encode(token_, recipient, amount);
        AgentExecuteParams memory params =
            AgentExecuteParams({agentID: agentID, payload: abi.encode(AgentExecuteCommand.TransferToken, payload)});
        return (Command.AgentExecute, abi.encode(params));
    }

    function makeUnlockTokenCommand(bytes32 agentID, address token_, address recipient, uint128 amount)
        public
        pure
        returns (Command, bytes memory)
    {
        TransferNativeTokenParams memory params =
            TransferNativeTokenParams({agentID: agentID, token: token_, recipient: recipient, amount: amount});
        return (Command.TransferNativeToken, abi.encode(params));
    }

    function makeTransferNativeFromAgentCommand(bytes32 agentID, address recipient, uint128 amount)
        public
        pure
        returns (Command, bytes memory)
    {
        TransferNativeFromAgentParams memory params =
            TransferNativeFromAgentParams({agentID: agentID, recipient: recipient, amount: amount});
        return (Command.TransferNativeFromAgent, abi.encode(params));
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
        (Command command, bytes memory params) = makeCreateAgentCommand();

        // Expect the gateway to emit `InboundMessageDispatched`
        vm.expectEmit();
        emit IGateway.InboundMessageDispatched(assetHubParaID.into(), 1, messageID, true);

        hoax(relayer, 1 ether);
        IGateway(address(gateway)).submitV1(
            InboundMessage(assetHubParaID.into(), 1, command, params, maxDispatchGas, maxRefund, reward, messageID),
            proof,
            makeMockProof()
        );
    }

    function testLegacyUnlockWethHappyPath() public {
        address recipient = makeAddr("test_recipeint");
        uint128 amount = 1;

        hoax(assetHubAgent, amount);
        token.deposit{value: amount}();

        (Command command, bytes memory params) =
            makeLegacyUnlockTokenCommand(assetHubAgentID, address(token), recipient, amount);

        assertEq(token.balanceOf(assetHubAgent), amount);
        assertEq(token.balanceOf(recipient), 0);

        // Expect WETH.Transfer event.
        vm.expectEmit();
        emit WETH9.Transfer(assetHubAgent, recipient, amount);

        // Expect the gateway to emit `InboundMessageDispatched`
        vm.expectEmit();
        emit IGateway.InboundMessageDispatched(assetHubParaID.into(), 1, messageID, true);

        hoax(relayer, 1 ether);
        IGateway(address(gateway)).submitV1(
            InboundMessage(assetHubParaID.into(), 1, command, params, maxDispatchGas, maxRefund, reward, messageID),
            proof,
            makeMockProof()
        );

        assertEq(token.balanceOf(assetHubAgent), 0);
        assertEq(token.balanceOf(recipient), amount);
    }

    function testUnlockWethHappyPath() public {
        address recipient = makeAddr("test_recipeint");
        uint128 amount = 1;

        hoax(assetHubAgent, amount);
        token.deposit{value: amount}();

        (Command command, bytes memory params) =
            makeUnlockTokenCommand(assetHubAgentID, address(token), recipient, amount);

        assertEq(token.balanceOf(assetHubAgent), amount);
        assertEq(token.balanceOf(recipient), 0);

        // Expect WETH.Transfer event.
        vm.expectEmit();
        emit WETH9.Transfer(assetHubAgent, recipient, amount);

        // Expect the gateway to emit `InboundMessageDispatched`
        vm.expectEmit(true, false, false, true);
        emit IGateway.InboundMessageDispatched(assetHubParaID.into(), 1, messageID, true);

        hoax(relayer, 1 ether);
        IGateway(address(gateway)).submitV1(
            InboundMessage(assetHubParaID.into(), 1, command, params, maxDispatchGas, maxRefund, reward, messageID),
            proof,
            makeMockProof()
        );

        assertEq(token.balanceOf(assetHubAgent), 0);
        assertEq(token.balanceOf(recipient), amount);
    }

    function testLegacyUnlockEthHappyPath() public {
        address recipient = makeAddr("test_recipeint");
        uint128 amount = 1;

        deal(assetHubAgent, amount);

        (Command command, bytes memory params) =
            makeLegacyUnlockTokenCommand(assetHubAgentID, address(0), recipient, amount);

        assertEq(assetHubAgent.balance, amount);
        assertEq(recipient.balance, 0);

        // Expect the gateway to emit `InboundMessageDispatched`
        vm.expectEmit();
        emit IGateway.InboundMessageDispatched(assetHubParaID.into(), 1, messageID, true);

        hoax(relayer, 1 ether);
        IGateway(address(gateway)).submitV1(
            InboundMessage(assetHubParaID.into(), 1, command, params, maxDispatchGas, maxRefund, reward, messageID),
            proof,
            makeMockProof()
        );

        assertEq(assetHubAgent.balance, 0);
        assertEq(recipient.balance, amount);
    }

    function testUnlockEthHappyPath() public {
        address recipient = makeAddr("test_recipeint");
        uint128 amount = 1;

        deal(assetHubAgent, amount);

        (Command command, bytes memory params) = makeUnlockTokenCommand(assetHubAgentID, address(0), recipient, amount);

        assertEq(assetHubAgent.balance, amount);
        assertEq(recipient.balance, 0);

        // Expect the gateway to emit `InboundMessageDispatched`
        vm.expectEmit(true, false, false, true);
        emit IGateway.InboundMessageDispatched(assetHubParaID.into(), 1, messageID, true);

        hoax(relayer, 1 ether);
        IGateway(address(gateway)).submitV1(
            InboundMessage(assetHubParaID.into(), 1, command, params, maxDispatchGas, maxRefund, reward, messageID),
            proof,
            makeMockProof()
        );

        assertEq(assetHubAgent.balance, 0);
        assertEq(recipient.balance, amount);
    }

    function testSubmitFailInvalidNonce() public {
        (Command command, bytes memory params) = makeCreateAgentCommand();

        hoax(relayer, 1 ether);
        IGateway(address(gateway)).submitV1(
            InboundMessage(assetHubParaID.into(), 1, command, params, maxDispatchGas, maxRefund, reward, messageID),
            proof,
            makeMockProof()
        );

        // try to replay the message
        vm.expectRevert(Gateway.InvalidNonce.selector);
        hoax(relayer, 1 ether);
        IGateway(address(gateway)).submitV1(
            InboundMessage(assetHubParaID.into(), 1, command, params, maxDispatchGas, maxRefund, reward, messageID),
            proof,
            makeMockProof()
        );
    }

    function testSubmitFailInvalidChannel() public {
        (Command command,) = makeCreateAgentCommand();

        vm.expectRevert(Gateway.ChannelDoesNotExist.selector);
        hoax(relayer);
        IGateway(address(gateway)).submitV1(
            InboundMessage(ParaID.wrap(42).into(), 1, command, "", maxDispatchGas, maxRefund, reward, messageID),
            proof,
            makeMockProof()
        );
    }

    function testSubmitFailInvalidProof() public {
        (Command command, bytes memory params) = makeCreateAgentCommand();

        MockGateway(address(gateway)).setCommitmentsAreVerified(false);
        vm.expectRevert(Gateway.InvalidProof.selector);

        hoax(relayer, 1 ether);
        IGateway(address(gateway)).submitV1(
            InboundMessage(assetHubParaID.into(), 1, command, params, maxDispatchGas, maxRefund, reward, messageID),
            proof,
            makeMockProof()
        );
    }

    /**
     * Fees & Rewards
     */

    // Test that the Gateway Proxy can receive funds to act as a wallet to pay out rewards and refunds
    function testGatewayProxyCanReceiveFunds() public {
        uint256 amount = 1 ether;
        address deployer = makeAddr("deployer");
        hoax(deployer, amount);

        assertEq(address(gateway).balance, 0);

        vm.expectRevert(GatewayProxy.NativeCurrencyNotAccepted.selector);
        SafeNativeTransfer.safeNativeTransfer(payable(gateway), amount);

        IGateway(address(gateway)).depositEther{value: amount}();

        assertEq(address(gateway).balance, amount);
    }

    // Message relayer should be rewarded from the agent for a channel
    function testRelayerRewardedFromGateway() public {
        (Command command, bytes memory params) = makeCreateAgentCommand();

        vm.txGasPrice(10 gwei);
        hoax(relayer, 1 ether);
        deal(address(gateway), 50 ether);

        uint256 relayerBalanceBefore = address(relayer).balance;
        uint256 gatewayBalanceBefore = address(address(gateway)).balance;
        uint256 agentBalanceBefore = address(assetHubAgent).balance;

        uint256 startGas = gasleft();
        IGateway(address(gateway)).submitV1(
            InboundMessage(assetHubParaID.into(), 1, command, params, maxDispatchGas, maxRefund, reward, messageID),
            proof,
            makeMockProof()
        );
        uint256 endGas = gasleft();
        uint256 estimatedActualRefundAmount = (startGas - endGas) * tx.gasprice;
        assertLt(estimatedActualRefundAmount, maxRefund);

        // Agents do not pay reward+refund so no balance should change.
        assertEq(address(assetHubAgent).balance, agentBalanceBefore);
        // Relayer balance has increased
        assertLt(address(gateway).balance, gatewayBalanceBefore);
        // Relayer balance has increased
        assertGt(relayer.balance, relayerBalanceBefore);

        // The total amount paid to the relayer
        uint256 totalPaid = gatewayBalanceBefore - address(gateway).balance;

        // Since we know that the actual refund amount is less than the max refund,
        // the total amount paid to the relayer is less.
        assertLt(totalPaid, maxRefund + reward);
    }

    // In this case, the gateway has no funds to reward the relayer
    function testRelayerNotRewarded() public {
        (Command command, bytes memory params) = makeCreateAgentCommand();

        hoax(relayer, 1 ether);
        IGateway(address(gateway)).submitV1(
            InboundMessage(assetHubParaID.into(), 1, command, params, maxDispatchGas, maxRefund, reward, messageID),
            proof,
            makeMockProof()
        );

        assertEq(address(assetHubAgent).balance, 0 ether);
        assertEq(relayer.balance, 1 ether);
    }

    function testSendingWethWithFeeSucceeds() public {
        // Create a mock user
        address user = makeAddr("user");
        deal(address(token), user, 1);

        // register token first
        uint256 fee = IGateway(address(gateway)).quoteRegisterTokenFee();
        IGateway(address(gateway)).registerToken{value: fee}(address(token));

        fee = IGateway(address(gateway)).quoteSendTokenFee(address(token), ParaID.wrap(0), 1);

        uint256 gatewayBeforeBalance = address(gateway).balance;

        // Let gateway lock up to 1 tokens
        hoax(user);
        token.approve(address(gateway), 1);

        hoax(user, fee);
        IGateway(address(gateway)).sendToken{value: fee}(address(token), ParaID.wrap(0), recipientAddress32, 1, 1);

        assertEq(address(gateway).balance - gatewayBeforeBalance, fee);
        assertEq(user.balance, 0);
    }

    function testSendingEthWithAmountAndFeeSucceeds() public {
        // Create a mock user
        address user = makeAddr("user");
        uint128 amount = 1;
        ParaID paraID = ParaID.wrap(1000);

        uint128 fee = uint128(IGateway(address(gateway)).quoteSendTokenFee(address(0), paraID, 1));
        uint256 gatewayBeforeBalance = address(gateway).balance;
        uint256 assetHubBeforeBalance = address(assetHubAgent).balance;

        vm.expectEmit();
        emit IGateway.TokenSent(address(0), user, paraID, recipientAddress32, amount);
        vm.expectEmit(true, false, false, false);
        emit IGateway.OutboundMessageAccepted(paraID.into(), 1, messageID, hex"");
        hoax(user, amount + fee);
        IGateway(address(gateway)).sendToken{value: amount + fee}(address(0), paraID, recipientAddress32, 1, amount);

        assertEq(address(gateway).balance - gatewayBeforeBalance, fee);
        assertEq(address(assetHubAgent).balance - assetHubBeforeBalance, amount);
        assertEq(user.balance, 0);
    }

    function testSendingEthWithAmountFeeAndExtraSucceedsWithRefund() public {
        // Create a mock user
        address user = makeAddr("user");
        uint128 amount = 1 ether;
        uint128 extra = 2 ether;
        ParaID paraID = ParaID.wrap(1000);

        uint128 fee = uint128(IGateway(address(gateway)).quoteSendTokenFee(address(0), paraID, 1));
        uint256 gatewayBeforeBalance = address(gateway).balance;
        uint256 assetHubBeforeBalance = address(assetHubAgent).balance;

        vm.expectEmit();
        emit IGateway.TokenSent(address(0), user, paraID, recipientAddress32, amount);
        vm.expectEmit(true, false, false, false);
        emit IGateway.OutboundMessageAccepted(paraID.into(), 1, messageID, hex"");
        hoax(user, amount + fee + extra);
        IGateway(address(gateway)).sendToken{value: amount + fee + extra}(
            address(0), paraID, recipientAddress32, 1, amount
        );

        assertEq(address(gateway).balance - gatewayBeforeBalance, fee);
        assertEq(address(assetHubAgent).balance - assetHubBeforeBalance, amount);
        assertEq(user.balance, extra);
    }

    function testSendingEtherWithInsufficientEther1() public {
        // Create a mock user
        address user = makeAddr("user");
        uint128 amount = 1;
        ParaID paraID = ParaID.wrap(1000);

        uint128 fee = uint128(IGateway(address(gateway)).quoteSendTokenFee(address(0), paraID, 1));

        vm.expectEmit();
        emit IGateway.TokenSent(address(0), user, paraID, recipientAddress32, amount);
        vm.expectRevert(Gateway.InsufficientEther.selector);
        hoax(user, amount + fee);
        IGateway(address(gateway)).sendToken{value: amount}(address(0), paraID, recipientAddress32, 1, amount);
    }

    function testSendingEtherWithInsufficientEther2() public {
        // Create a mock user
        address user = makeAddr("user");
        uint128 amount = 1 ether;
        ParaID paraID = ParaID.wrap(1000);

        uint128 fee = uint128(IGateway(address(gateway)).quoteSendTokenFee(address(0), paraID, amount));

        vm.expectRevert(Gateway.InsufficientEther.selector);
        hoax(user, amount + fee);
        IGateway(address(gateway)).sendToken{value: amount - 1}(address(0), paraID, recipientAddress32, 1, amount);
    }

    // User doesn't have enough funds to send message
    function testUserDoesNotProvideEnoughFees() public {
        // register token first
        uint256 fee = IGateway(address(gateway)).quoteRegisterTokenFee();
        IGateway(address(gateway)).registerToken{value: fee}(address(token));

        // Create a mock user
        address user = makeAddr("user");
        deal(address(token), user, 1);

        // Let gateway lock up to 1 tokens
        hoax(user);
        token.approve(address(gateway), 1);

        vm.expectRevert(Gateway.InsufficientEther.selector);
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

        TransferNativeTokenParams memory params = TransferNativeTokenParams({
            agentID: assetHubAgentID,
            token: address(token),
            recipient: account2,
            amount: 10
        });

        bytes memory encodedParams = abi.encode(params);
        MockGateway(address(gateway)).transferNativeTokenPublic(encodedParams);
    }

    function testAgentExecutionBadPayload() public {
        AgentExecuteParams memory params = AgentExecuteParams({agentID: assetHubAgentID, payload: ""});

        vm.expectRevert(Gateway.InvalidAgentExecutionPayload.selector);
        MockGateway(address(gateway)).agentExecutePublic(abi.encode(params));
    }

    function testCreateAgent() public {
        bytes32 agentID = keccak256("123");
        CreateAgentParams memory params = CreateAgentParams({agentID: agentID});

        vm.expectEmit(false, false, false, false, address(gateway));
        emit IGateway.AgentCreated(agentID, address(0));

        MockGateway(address(gateway)).createAgentPublic(abi.encode(params));
    }

    function testCreateAgentAlreadyCreated() public {
        bytes32 agentID = keccak256("123");
        CreateAgentParams memory params = CreateAgentParams({agentID: agentID});

        MockGateway(address(gateway)).createAgentPublic(abi.encode(params));

        vm.expectRevert(Gateway.AgentAlreadyCreated.selector);
        MockGateway(address(gateway)).createAgentPublic(abi.encode(params));
    }

    function testCreateChannel() public {
        ParaID paraID = ParaID.wrap(3042);
        bytes32 agentID = keccak256("3042");

        MockGateway(address(gateway)).createAgentPublic(abi.encode(CreateAgentParams({agentID: agentID})));

        CreateChannelParams memory params =
            CreateChannelParams({channelID: paraID.into(), agentID: agentID, mode: OperatingMode.Normal});

        vm.expectEmit(true, false, false, true);
        emit IGateway.ChannelCreated(paraID.into());
        MockGateway(address(gateway)).createChannelPublic(abi.encode(params));
    }

    function testCreateChannelFailsAgentDoesNotExist() public {
        ParaID paraID = ParaID.wrap(3042);
        bytes32 agentID = keccak256("3042");

        CreateChannelParams memory params =
            CreateChannelParams({channelID: paraID.into(), mode: OperatingMode.Normal, agentID: agentID});

        vm.expectRevert(Gateway.AgentDoesNotExist.selector);
        MockGateway(address(gateway)).createChannelPublic(abi.encode(params));
    }

    function testCreateChannelFailsChannelAlreadyExists() public {
        ParaID paraID = ParaID.wrap(3042);
        bytes32 agentID = keccak256("3042");

        MockGateway(address(gateway)).createAgentPublic(abi.encode(CreateAgentParams({agentID: agentID})));

        CreateChannelParams memory params =
            CreateChannelParams({channelID: paraID.into(), agentID: agentID, mode: OperatingMode.Normal});

        MockGateway(address(gateway)).createChannelPublic(abi.encode(params));

        vm.expectRevert(Gateway.ChannelAlreadyCreated.selector);
        MockGateway(address(gateway)).createChannelPublic(abi.encode(params));
    }

    function testUpdateChannel() public {
        // get current fee (0.0025 ether)
        PricingStorage.Layout storage pricing = PricingStorage.layout();
        uint256 fee = pricing.deliveryCost;

        bytes memory params = abi.encode(
            UpdateChannelParams({channelID: assetHubParaID.into(), mode: OperatingMode.RejectingOutboundMessages})
        );

        vm.expectEmit(true, false, false, true);
        emit IGateway.ChannelUpdated(assetHubParaID.into());
        MockGateway(address(gateway)).updateChannelPublic(params);

        // Due to the new exchange rate, new fee is halved
        uint256 newFee = pricing.deliveryCost;
        assertEq(fee / 2, newFee);
    }

    function testUpdateChannelFailDoesNotExist() public {
        bytes memory params = abi.encode(
            UpdateChannelParams({channelID: ParaID.wrap(5956).into(), mode: OperatingMode.RejectingOutboundMessages})
        );

        vm.expectRevert(Gateway.ChannelDoesNotExist.selector);
        MockGateway(address(gateway)).updateChannelPublic(params);
    }

    function testUpdateChannelSanityChecksForPrimaryGovernanceChannel() public {
        bytes memory params = abi.encode(
            UpdateChannelParams({
                channelID: ChannelID.wrap(bytes32(uint256(1))),
                mode: OperatingMode.RejectingOutboundMessages
            })
        );

        vm.expectRevert(Gateway.InvalidChannelUpdate.selector);
        MockGateway(address(gateway)).updateChannelPublic(params);
    }

    function testUpgrade() public {
        // Upgrade to this new logic contract
        MockGatewayV2 newLogic = new MockGatewayV2();

        UpgradeParams memory params = UpgradeParams({
            impl: address(newLogic),
            implCodeHash: address(newLogic).codehash,
            initParams: abi.encode(42)
        });

        // Expect the gateway to emit `Upgraded`
        vm.expectEmit(true, false, false, false);
        emit IUpgradable.Upgraded(address(newLogic));

        MockGateway(address(gateway)).upgradePublic(abi.encode(params));

        // Verify that the MockGatewayV2.initialize was called
        assertEq(MockGatewayV2(address(gateway)).getValue(), 42);
    }

    function testUpgradeFailOnInitializationFailure() public {
        MockGatewayV2 newLogic = new MockGatewayV2();

        UpgradeParams memory params = UpgradeParams({
            impl: address(newLogic),
            implCodeHash: address(newLogic).codehash,
            initParams: abi.encode(666)
        });

        vm.expectRevert("initialize failed");
        MockGateway(address(gateway)).upgradePublic(abi.encode(params));
    }

    function testUpgradeFailCodeHashMismatch() public {
        MockGatewayV2 newLogic = new MockGatewayV2();

        UpgradeParams memory params =
            UpgradeParams({impl: address(newLogic), implCodeHash: bytes32(0), initParams: abi.encode(42)});

        vm.expectRevert(IUpgradable.InvalidCodeHash.selector);
        MockGateway(address(gateway)).upgradePublic(abi.encode(params));
    }

    function testSetOperatingMode() public {
        SetOperatingModeParams memory params = SetOperatingModeParams({mode: OperatingMode.RejectingOutboundMessages});

        OperatingMode mode = IGateway(address(gateway)).operatingMode();
        assertEq(uint256(mode), 0);

        MockGateway(address(gateway)).setOperatingModePublic(abi.encode(params));

        mode = IGateway(address(gateway)).operatingMode();
        assertEq(uint256(mode), 1);
    }

    function testWithdrawAgentFundIsIgnored() public {
        address recipient = makeAddr("test_recipeint");
        uint128 amount = 1;

        deal(assetHubAgent, amount);

        (Command command, bytes memory params) = makeTransferNativeFromAgentCommand(assetHubAgentID, recipient, amount);

        assertEq(address(assetHubAgent).balance, amount);
        assertEq(recipient.balance, 0);

        // Expect the gateway to emit `InboundMessageDispatched`
        vm.expectEmit();
        emit IGateway.InboundMessageDispatched(assetHubParaID.into(), 1, messageID, true);

        hoax(relayer, 1 ether);
        IGateway(address(gateway)).submitV1(
            InboundMessage(assetHubParaID.into(), 1, command, params, maxDispatchGas, maxRefund, reward, messageID),
            proof,
            makeMockProof()
        );

        assertEq(address(assetHubAgent).balance, amount);
        assertEq(recipient.balance, 0);
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

        uint256 totalFee = MockGateway(address(gateway)).quoteRegisterTokenFee();

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

        // register token first
        uint256 fee = IGateway(address(gateway)).quoteRegisterTokenFee();
        IGateway(address(gateway)).registerToken{value: fee}(address(token));

        fee = IGateway(address(gateway)).quoteSendTokenFee(address(token), destPara, 1);

        vm.expectEmit(true, true, false, true);
        emit IGateway.TokenSent(address(token), address(this), destPara, recipientAddress32, 1);

        // Expect the gateway to emit `OutboundMessageAccepted`
        vm.expectEmit(true, false, false, false);
        emit IGateway.OutboundMessageAccepted(assetHubParaID.into(), 1, messageID, bytes(""));

        IGateway(address(gateway)).sendToken{value: fee}(address(token), destPara, recipientAddress32, 1, 1);
    }

    function testSendTokenAddress32ToAssetHub() public {
        // Let gateway lock up to 1 tokens
        token.approve(address(gateway), 1);

        // Multilocation for recipient
        ParaID destPara = assetHubParaID;

        // register token first
        uint256 fee = IGateway(address(gateway)).quoteRegisterTokenFee();
        IGateway(address(gateway)).registerToken{value: fee}(address(token));

        fee = IGateway(address(gateway)).quoteSendTokenFee(address(token), destPara, 1);

        vm.expectEmit(true, true, false, true);
        emit IGateway.TokenSent(address(token), address(this), destPara, recipientAddress32, 1);

        // Expect the gateway to emit `OutboundMessageAccepted`
        vm.expectEmit(true, false, false, false);
        emit IGateway.OutboundMessageAccepted(assetHubParaID.into(), 1, messageID, bytes(""));

        IGateway(address(gateway)).sendToken{value: fee}(address(token), destPara, recipientAddress32, 1, 1);
    }

    function testSendTokenAddress20() public {
        // Let gateway lock up to 1 tokens
        token.approve(address(gateway), 1);

        // Multilocation for recipient
        ParaID destPara = ParaID.wrap(2043);

        // register token first
        uint256 fee = IGateway(address(gateway)).quoteRegisterTokenFee();
        IGateway(address(gateway)).registerToken{value: fee}(address(token));

        fee = IGateway(address(gateway)).quoteSendTokenFee(address(token), destPara, 1);

        vm.expectEmit(true, true, false, true);
        emit IGateway.TokenSent(address(token), address(this), destPara, recipientAddress20, 1);

        // Expect the gateway to emit `OutboundMessageAccepted`
        vm.expectEmit(true, false, false, false);
        emit IGateway.OutboundMessageAccepted(assetHubParaID.into(), 1, messageID, bytes(""));

        IGateway(address(gateway)).sendToken{value: fee}(address(token), destPara, recipientAddress20, 1, 1);
    }

    function testSendTokenAddress20FailsInvalidDestination() public {
        // Let gateway lock up to 1 tokens
        token.approve(address(gateway), 1);

        ParaID destPara = assetHubParaID;

        // register token first
        uint256 fee = IGateway(address(gateway)).quoteRegisterTokenFee();
        IGateway(address(gateway)).registerToken{value: fee}(address(token));

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

        MockGateway(address(gateway)).setOperatingModePublic(
            abi.encode(SetOperatingModeParams({mode: OperatingMode.RejectingOutboundMessages}))
        );

        OperatingMode mode = IGateway(address(gateway)).operatingMode();
        assertEq(uint256(mode), 1);
    }

    function testDisableOutboundMessagingForChannel() public {
        // Let gateway lock up to 1 tokens
        token.approve(address(gateway), 1);

        MockGateway(address(gateway)).setOperatingModePublic(
            abi.encode(SetOperatingModeParams({mode: OperatingMode.Normal}))
        );

        // register token first
        uint256 fee = IGateway(address(gateway)).quoteRegisterTokenFee();
        IGateway(address(gateway)).registerToken{value: fee}(address(token));

        bytes memory params = abi.encode(
            UpdateChannelParams({channelID: assetHubParaID.into(), mode: OperatingMode.RejectingOutboundMessages})
        );
        MockGateway(address(gateway)).updateChannelPublic(params);

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
        MockGateway(address(gatewayLogic)).initialize("");
    }

    // Handler functions should not be externally callable
    function testHandlersNotExternallyCallable() public {
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
        Gateway(address(gateway)).mintForeignToken(ParaID.wrap(3042).into(), "");
    }

    function testGetters() public {
        IGateway gw = IGateway(address(gateway));

        OperatingMode mode = gw.operatingMode();
        assertEq(uint256(mode), 0);

        OperatingMode channelMode = gw.channelOperatingModeOf(assetHubParaID.into());
        assertEq(uint256(channelMode), 0);

        (, uint128 fee) = gw.pricingParameters();
        assertEq(fee, 10000000000);

        (uint64 inbound, uint64 outbound) = gw.channelNoncesOf(assetHubParaID.into());
        assertEq(inbound, 0);
        assertEq(outbound, 0);

        address agent = gw.agentOf(assetHubAgentID);
        assertEq(agent, assetHubAgent);

        address implementation = gw.implementation();
        assertEq(implementation, address(gatewayLogic));
    }

    function testCreateAgentWithNotEnoughGas() public {
        (Command command, bytes memory params) = makeCreateAgentCommand();

        hoax(relayer, 1 ether);

        vm.expectEmit(true, false, false, true);
        // Expect dispatch result as false for `OutOfGas`
        emit IGateway.InboundMessageDispatched(assetHubParaID.into(), 1, messageID, false);
        // maxDispatchGas as 1 for `create_agent` is definitely not enough
        IGateway(address(gateway)).submitV1(
            InboundMessage(assetHubParaID.into(), 1, command, params, 1, maxRefund, reward, messageID),
            proof,
            makeMockProof()
        );
    }

    function testSetTokenFees() public {
        uint256 fee = IGateway(address(gateway)).quoteRegisterTokenFee();
        assertEq(fee, 5000000000000000);
        // Double the assetHubCreateAssetFee
        MockGateway(address(gateway)).setTokenTransferFeesPublic(
            abi.encode(
                SetTokenTransferFeesParams({
                    assetHubCreateAssetFee: createTokenFee * 2,
                    registerTokenFee: registerTokenFee,
                    assetHubReserveTransferFee: sendTokenFee * 3
                })
            )
        );
        fee = IGateway(address(gateway)).quoteRegisterTokenFee();
        // since deliveryCost not changed, so the total fee increased only by 50%
        assertEq(fee, 7500000000000000);
    }

    bytes32 public expectChannelIDBytes = bytes32(0xc173fac324158e77fb5840738a1a541f633cbec8884c6a601c567d2b376a0539);

    function testDeriveChannelID() public {
        ParaID para_id = ParaID.wrap(1000);
        ChannelID channel_id = para_id.into();
        assertEq(ChannelID.unwrap(channel_id), expectChannelIDBytes);
    }

    function testSetPricingParameters() public {
        uint256 fee = IGateway(address(gateway)).quoteRegisterTokenFee();
        assertEq(fee, 5000000000000000);
        // Double both the exchangeRate and multiplier. Should lead to an 4x fee increase
        MockGateway(address(gateway)).setPricingParametersPublic(
            abi.encode(
                SetPricingParametersParams({
                    exchangeRate: exchangeRate.mul(convert(2)),
                    multiplier: multiplier.mul(convert(2)),
                    deliveryCost: outboundFee
                })
            )
        );
        // Should expect 4x fee increase
        fee = IGateway(address(gateway)).quoteRegisterTokenFee();
        assertEq(fee, 20000000000000001);
    }

    function testSendTokenWithZeroDestinationFee() public {
        // Let gateway lock up to 1 tokens
        token.approve(address(gateway), 1);

        // Multilocation for recipient
        ParaID destPara = ParaID.wrap(2043);

        // register token first
        uint256 fee = IGateway(address(gateway)).quoteRegisterTokenFee();
        IGateway(address(gateway)).registerToken{value: fee}(address(token));
        fee = IGateway(address(gateway)).quoteSendTokenFee(address(token), destPara, 0);

        vm.expectRevert(Assets.InvalidDestinationFee.selector);
        IGateway(address(gateway)).sendToken{value: fee}(address(token), destPara, recipientAddress32, 0, 1);
    }

    function testSendTokenWithLargeDestinationFee() public {
        // Let gateway lock up to 1 tokens
        token.approve(address(gateway), 1);

        // Multilocation for recipient
        ParaID destPara = ParaID.wrap(2043);

        // register token first
        uint256 fee = IGateway(address(gateway)).quoteRegisterTokenFee();
        IGateway(address(gateway)).registerToken{value: fee}(address(token));

        vm.expectRevert(Assets.InvalidDestinationFee.selector);
        IGateway(address(gateway)).quoteSendTokenFee(address(token), destPara, maxDestinationFee + 1);

        vm.expectRevert(Assets.InvalidDestinationFee.selector);
        IGateway(address(gateway)).sendToken{value: fee}(
            address(token), destPara, recipientAddress32, maxDestinationFee + 1, 1
        );
    }

    function testRegisterForeignToken() public {
        RegisterForeignTokenParams memory params =
            RegisterForeignTokenParams({foreignTokenID: dotTokenID, name: "DOT", symbol: "DOT", decimals: 10});

        vm.expectEmit(true, true, false, false);
        emit IGateway.ForeignTokenRegistered(bytes32(uint256(1)), address(0));

        MockGateway(address(gateway)).registerForeignTokenPublic(abi.encode(params));
    }

    function testRegisterForeignTokenDuplicateFail() public {
        testRegisterForeignToken();

        RegisterForeignTokenParams memory params =
            RegisterForeignTokenParams({foreignTokenID: dotTokenID, name: "DOT", symbol: "DOT", decimals: 10});

        vm.expectRevert(Assets.TokenAlreadyRegistered.selector);

        MockGateway(address(gateway)).registerForeignTokenPublic(abi.encode(params));
    }

    function testMintForeignToken() public {
        testRegisterForeignToken();

        uint256 amount = 1000;

        MintForeignTokenParams memory params =
            MintForeignTokenParams({foreignTokenID: bytes32(uint256(1)), recipient: account1, amount: amount});

        vm.expectEmit(true, true, false, false);
        emit Transfer(address(0), account1, 1000);

        MockGateway(address(gateway)).mintForeignTokenPublic(assetHubParaID.into(), abi.encode(params));

        address dotToken = MockGateway(address(gateway)).tokenAddressOf(dotTokenID);

        uint256 balance = Token(dotToken).balanceOf(account1);

        assertEq(balance, amount);
    }

    function testMintNotRegisteredTokenWillFail() public {
        MintForeignTokenParams memory params =
            MintForeignTokenParams({foreignTokenID: bytes32(uint256(1)), recipient: account1, amount: 1000});

        vm.expectRevert(Assets.TokenNotRegistered.selector);

        MockGateway(address(gateway)).mintForeignTokenPublic(assetHubParaID.into(), abi.encode(params));
    }

    function testMintFromParachainOtherThanAssetHubWillFail() public {
        MintForeignTokenParams memory params =
            MintForeignTokenParams({foreignTokenID: bytes32(uint256(1)), recipient: account1, amount: 1000});

        vm.expectRevert(Assets.TokenMintFailed.selector);

        MockGateway(address(gateway)).mintForeignTokenPublic(bridgeHubParaID.into(), abi.encode(params));
    }

    function testSendRelayTokenToAssetHubWithAddress32() public {
        // Register and then mint some DOT to account1
        testMintForeignToken();

        address dotToken = MockGateway(address(gateway)).tokenAddressOf(dotTokenID);

        ParaID destPara = assetHubParaID;

        vm.prank(account1);

        vm.expectEmit(true, true, false, true);
        emit IGateway.TokenSent(address(dotToken), account1, destPara, recipientAddress32, 1);

        // Expect the gateway to emit `OutboundMessageAccepted`
        vm.expectEmit(true, false, false, false);
        emit IGateway.OutboundMessageAccepted(assetHubParaID.into(), 1, messageID, bytes(""));

        IGateway(address(gateway)).sendToken{value: 0.1 ether}(address(dotToken), destPara, recipientAddress32, 1, 1);
    }

    function testSendRelayTokenToAssetHubWithAddress20() public {
        // Register and then mint some DOT to account1
        testMintForeignToken();

        address dotToken = MockGateway(address(gateway)).tokenAddressOf(dotTokenID);

        ParaID destPara = assetHubParaID;

        vm.prank(account1);

        vm.expectRevert(Assets.Unsupported.selector);
        IGateway(address(gateway)).sendToken{value: 0.1 ether}(address(dotToken), destPara, recipientAddress20, 1, 1);
    }

    function testSendRelayTokenToDestinationChainWithAddress32() public {
        // Register and then mint some DOT to account1
        testMintForeignToken();

        address dotToken = MockGateway(address(gateway)).tokenAddressOf(dotTokenID);

        ParaID destPara = ParaID.wrap(2043);

        vm.prank(account1);

        vm.expectRevert(Assets.Unsupported.selector);
        IGateway(address(gateway)).sendToken{value: 0.1 ether}(address(dotToken), destPara, recipientAddress32, 1, 1);
    }

    function testSendRelayTokenToDestinationChainWithAddress20() public {
        // Register and then mint some DOT to account1
        testMintForeignToken();

        address dotToken = MockGateway(address(gateway)).tokenAddressOf(dotTokenID);

        ParaID destPara = ParaID.wrap(2043);

        vm.prank(account1);

        vm.expectRevert(Assets.Unsupported.selector);
        IGateway(address(gateway)).sendToken{value: 0.1 ether}(address(dotToken), destPara, recipientAddress20, 1, 1);
    }

    function testSendNotRegisteredTokenWillFail() public {
        ParaID destPara = assetHubParaID;

        vm.expectRevert(Assets.TokenNotRegistered.selector);

        IGateway(address(gateway)).sendToken{value: 0.1 ether}(address(0x1), destPara, recipientAddress32, 1, 1);
    }

    function testSendTokenFromNotMintedAccountWillFail() public {
        testRegisterForeignToken();

        address dotToken = MockGateway(address(gateway)).tokenAddressOf(dotTokenID);

        ParaID destPara = assetHubParaID;

        vm.prank(account1);

        vm.expectRevert(abi.encodeWithSelector(IERC20.InsufficientBalance.selector, account1, 0, 1));

        IGateway(address(gateway)).sendToken{value: 0.1 ether}(address(dotToken), destPara, recipientAddress32, 1, 1);
    }

    function testLegacyAgentExecutionForCompatibility() public {
        token.transfer(address(assetHubAgent), 200);

        AgentExecuteParams memory params = AgentExecuteParams({
            agentID: assetHubAgentID,
            payload: abi.encode(AgentExecuteCommand.TransferToken, abi.encode(address(token), address(account2), 10))
        });

        bytes memory encodedParams = abi.encode(params);
        MockGateway(address(gateway)).agentExecutePublic(encodedParams);
    }

    function testRegisterForeignTokenAsNativeTokenWillFail() public {
        testRegisterForeignToken();
        address dotToken = MockGateway(address(gateway)).tokenAddressOf(dotTokenID);
        uint256 fee = IGateway(address(gateway)).quoteRegisterTokenFee();
        vm.expectRevert(Assets.TokenAlreadyRegistered.selector);
        IGateway(address(gateway)).registerToken{value: fee}(dotToken);
    }

    function testReantrancyGuardReverts() public {
        testRegisterToken();

        ReantrantAttacker attacker = new ReantrantAttacker(address(gateway), address(token));
        // Fund attacker
        deal(address(attacker), 1 ether);
        deal(address(token), address(attacker), 5);

        uint128 amount = 1;
        uint128 extra = 1;
        uint128 destinationFee = 1;
        ParaID paraID = ParaID.wrap(1000);

        uint128 fee = uint128(IGateway(address(gateway)).quoteSendTokenFee(address(token), paraID, 0));

        hoax(address(attacker));
        token.approve(address(gateway), 5);

        vm.expectRevert(NativeTransferFailed.selector);
        hoax(address(attacker));
        IGateway(address(gateway)).sendToken{value: fee + extra}(
            address(token), paraID, recipientAddress32, destinationFee, amount
        );
    }

    function testRegisterTokenWithEthWillReturnInvalidToken() public {
        uint256 fee = IGateway(address(gateway)).quoteRegisterTokenFee();
        vm.expectRevert(Assets.InvalidToken.selector);
        IGateway(address(gateway)).registerToken{value: fee}(address(0));
    }
}
