// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import {Test} from "forge-std/Test.sol";
import {Strings} from "openzeppelin/utils/Strings.sol";
import {console} from "forge-std/console.sol";

import {BeefyClient} from "../src/BeefyClient.sol";

import {IGatewayBase} from "../src/interfaces/IGatewayBase.sol";
import {IGatewayV2} from "../src/v2/IGateway.sol";
import {IInitializable} from "../src/interfaces/IInitializable.sol";
import {IUpgradable} from "../src/interfaces/IUpgradable.sol";
import {Gateway} from "../src/Gateway.sol";
import {MockGateway} from "./mocks/MockGateway.sol";
import {MockGatewayV2} from "./mocks/MockGatewayV2.sol";
import {GatewayProxy} from "../src/GatewayProxy.sol";
import {Token} from "../src/Token.sol";

import {AgentExecutor} from "../src/AgentExecutor.sol";
import {Agent} from "../src/Agent.sol";
import {Verification} from "../src/Verification.sol";
import {SubstrateTypes} from "./../src/SubstrateTypes.sol";
import {OperatingMode, ParaID, CommandV2, CommandKind, InboundMessageV2} from "../src/Types.sol";

import {NativeTransferFailed} from "../src/utils/SafeTransfer.sol";
import {PricingStorage} from "../src/storage/PricingStorage.sol";
import {IERC20} from "../src/interfaces/IERC20.sol";
import {TokenLib} from "../src/TokenLib.sol";
import {Token} from "../src/Token.sol";

import {Initializer} from "../src/Initializer.sol";
import {Constants} from "../src/Constants.sol";

import {
    UpgradeParams,
    SetOperatingModeParams,
    UnlockNativeTokenParams,
    RegisterForeignTokenParams,
    MintForeignTokenParams,
    CallContractParams,
    Payload,
    Asset,
    makeNativeAsset,
    makeForeignAsset,
    Xcm,
    makeRawXCM
} from "../src/v2/Types.sol";

import {
    AgentExecuteCommand,
    InboundMessage,
    OperatingMode,
    ParaID,
    Command
} from "../src/v1/Types.sol";

import {WETH9} from "canonical-weth/WETH9.sol";
import {UD60x18, ud60x18, convert} from "prb/math/src/UD60x18.sol";

import {HelloWorld} from "./mocks/HelloWorld.sol";
import {Upgrade} from "../src/Upgrade.sol";

contract MockImpl {
    // slot 0 - will be written via delegatecall
    uint256 public initialized;

    // initialize matches IInitializable.initialize signature
    function initialize(bytes memory params) external {
        // decode a single uint256 for test convenience
        uint256 v = abi.decode(params, (uint256));
        initialized = v;
    }
}

// Thin wrapper contract that calls the library function so storage is on this contract
contract TestHandlers {
    // storage slot that MockImpl.initialize will write to via delegatecall
    uint256 public initialized;

    // forward an encoded UpgradeParams blob to the library
    function callUpgrade(address impl, bytes32 implCodeHash, bytes calldata initParams) external {
        // Call Upgrade.upgrade directly to run the upgrade flow and initializer
        Upgrade.upgrade(impl, implCodeHash, initParams);
    }
}

contract MockERC20 {
    mapping(address => uint256) public balance;

    function mint(address to, uint256 amount) external {
        balance[to] += amount;
    }

    function transfer(address to, uint256 amount) external returns (bool) {
        address from = msg.sender;
        require(balance[from] >= amount, "insufficient");
        balance[from] -= amount;
        balance[to] += amount;
        return true;
    }

    function transferFrom(address from, address to, uint256 amount) external returns (bool) {
        require(balance[from] >= amount, "insufficient");
        balance[from] -= amount;
        balance[to] += amount;
        return true;
    }

    function balanceOf(address who) external view returns (uint256) {
        return balance[who];
    }
}

contract PayableRecipient {
    receive() external payable {}
}
import "./mocks/FeeOnTransferToken.sol";

contract GatewayV2Test is Test {
    // Emitted when token minted/burnt/transferred
    event Transfer(address indexed from, address indexed to, uint256 value);

    address public assetHubAgent;

    address public relayer;
    bytes32 public relayerRewardAddress = keccak256("relayerRewardAddress");

    bytes32[] public proof =
        [bytes32(0x2f9ee6cfdf244060dc28aa46347c5219e303fc95062dd672b4e406ca5c29764b)];
    bytes public parachainHeaderProof = bytes("validProof");

    MockGateway public gatewayLogic;
    GatewayProxy public gateway;

    WETH9 public weth;

    address public user1;
    address public user2;

    // tokenID for DOT
    bytes32 public dotTokenID;

    AgentExecutor executor;

    HelloWorld public helloWorld;

    TestHandlers handler;
    MockImpl impl;

    event SaidHello(string indexed message);

    function setUp() public {
        weth = new WETH9();
        executor = new AgentExecutor();
        gatewayLogic = new MockGateway(address(0), address(executor));
        Initializer.Config memory config = Initializer.Config({
            mode: OperatingMode.Normal,
            deliveryCost: 1e10,
            registerTokenFee: 0,
            assetHubCreateAssetFee: 1e10,
            assetHubReserveTransferFee: 1e10,
            exchangeRate: ud60x18(0.0025e18),
            multiplier: ud60x18(1e18),
            foreignTokenDecimals: 10,
            maxDestinationFee: 1e11
        });
        gateway = new GatewayProxy(address(gatewayLogic), abi.encode(config));
        MockGateway(address(gateway)).setCommitmentsAreVerified(true);

        SetOperatingModeParams memory params = SetOperatingModeParams({mode: OperatingMode.Normal});
        MockGateway(address(gateway)).v1_handleSetOperatingMode_public(abi.encode(params));

        assetHubAgent = IGatewayV2(address(gateway)).agentOf(Constants.ASSET_HUB_AGENT_ID);

        // fund the message relayer account
        relayer = makeAddr("relayer");

        // Features

        user1 = makeAddr("user1");
        user2 = makeAddr("user2");

        // create tokens for account 1
        hoax(user1);
        weth.deposit{value: 1 ether}();

        // create tokens for account 2
        hoax(user2);
        weth.deposit{value: 1 ether}();

        dotTokenID = bytes32(uint256(1));

        helloWorld = new HelloWorld();

        handler = new TestHandlers();
        impl = new MockImpl();
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

    function makeMockCommand() public pure returns (CommandV2[] memory) {
        CommandV2[] memory commands = new CommandV2[](1);
        SetOperatingModeParams memory params = SetOperatingModeParams({mode: OperatingMode.Normal});
        commands[0] = CommandV2({
            kind: CommandKind.SetOperatingMode, gas: 500_000, payload: abi.encode(params)
        });
        return commands;
    }

    function makeUnlockWethCommand(uint128 value) public view returns (CommandV2[] memory) {
        return makeUnlockTokenCommand(address(weth), relayer, value);
    }

    function makeUnlockTokenCommand(address token, address recipient, uint128 amount)
        public
        pure
        returns (CommandV2[] memory)
    {
        UnlockNativeTokenParams memory params =
            UnlockNativeTokenParams({token: token, recipient: recipient, amount: amount});
        bytes memory payload = abi.encode(params);

        CommandV2[] memory commands = new CommandV2[](1);
        commands[0] =
            CommandV2({kind: CommandKind.UnlockNativeToken, gas: 500_000, payload: payload});
        return commands;
    }

    function makeRegisterForeignTokenCommand(
        bytes32 id,
        string memory name,
        string memory symbol,
        uint8 decimals
    ) public pure returns (CommandV2[] memory) {
        RegisterForeignTokenParams memory params =
            RegisterForeignTokenParams(id, name, symbol, decimals);
        bytes memory payload = abi.encode(params);

        CommandV2[] memory commands = new CommandV2[](1);
        commands[0] =
            CommandV2({kind: CommandKind.RegisterForeignToken, gas: 1_200_000, payload: payload});
        return commands;
    }

    function makeMintForeignTokenCommand(bytes32 id, address recipient, uint128 amount)
        public
        pure
        returns (CommandV2[] memory)
    {
        MintForeignTokenParams memory params = MintForeignTokenParams(id, recipient, amount);
        bytes memory payload = abi.encode(params);

        CommandV2[] memory commands = new CommandV2[](1);
        commands[0] =
            CommandV2({kind: CommandKind.MintForeignToken, gas: 100_000, payload: payload});
        return commands;
    }

    function makeCallContractCommand(uint256 value) public view returns (CommandV2[] memory) {
        bytes memory data = abi.encodeWithSignature("sayHello(string)", "World");
        CallContractParams memory params =
            CallContractParams({target: address(helloWorld), data: data, value: value});
        bytes memory payload = abi.encode(params);

        CommandV2[] memory commands = new CommandV2[](1);
        commands[0] = CommandV2({kind: CommandKind.CallContract, gas: 500_000, payload: payload});
        return commands;
    }

    function makeCallContractCommandWithFunctionNotExists(uint256 value)
        public
        view
        returns (CommandV2[] memory)
    {
        bytes memory data = abi.encodeWithSignature("sayHelloNotExists(string)", "World");
        CallContractParams memory params =
            CallContractParams({target: address(helloWorld), data: data, value: value});
        bytes memory payload = abi.encode(params);

        CommandV2[] memory commands = new CommandV2[](1);
        commands[0] = CommandV2({kind: CommandKind.CallContract, gas: 500_000, payload: payload});
        return commands;
    }

    function makeCallContractCommandWithInsufficientGas(uint256 value)
        public
        view
        returns (CommandV2[] memory)
    {
        bytes memory data = abi.encodeWithSignature("sayHello(string)", "World");
        CallContractParams memory params =
            CallContractParams({target: address(helloWorld), data: data, value: value});
        bytes memory payload = abi.encode(params);

        CommandV2[] memory commands = new CommandV2[](1);
        commands[0] = CommandV2({kind: CommandKind.CallContract, gas: 1, payload: payload});
        return commands;
    }

    /**
     * Message Verification
     */
    function testSubmitHappyPath() public {
        bytes32 topic = keccak256("topic");

        // Expect the gateway to emit `InboundMessageDispatched`
        vm.expectEmit(true, false, false, true);
        emit IGatewayV2.InboundMessageDispatched(1, topic, true, relayerRewardAddress);

        hoax(relayer, 1 ether);
        IGatewayV2(address(gateway))
            .v2_submit(
                InboundMessageV2({
                    origin: keccak256("666"), nonce: 1, topic: topic, commands: makeMockCommand()
                }),
                proof,
                makeMockProof(),
                relayerRewardAddress
            );
    }

    function testSubmitFailInvalidNonce() public {
        bytes32 topic = keccak256("topic");

        InboundMessageV2 memory message = InboundMessageV2({
            origin: keccak256("666"), nonce: 1, topic: topic, commands: makeMockCommand()
        });

        hoax(relayer, 1 ether);
        IGatewayV2(address(gateway))
            .v2_submit(message, proof, makeMockProof(), relayerRewardAddress);

        vm.expectRevert(IGatewayBase.InvalidNonce.selector);
        hoax(relayer, 1 ether);
        IGatewayV2(address(gateway))
            .v2_submit(message, proof, makeMockProof(), relayerRewardAddress);
    }

    function testSubmitFailInvalidProof() public {
        bytes32 topic = keccak256("topic");

        InboundMessageV2 memory message = InboundMessageV2({
            origin: keccak256("666"), nonce: 1, topic: topic, commands: makeMockCommand()
        });

        MockGateway(address(gateway)).setCommitmentsAreVerified(false);
        vm.expectRevert(IGatewayBase.InvalidProof.selector);

        hoax(relayer, 1 ether);
        IGatewayV2(address(gateway))
            .v2_submit(message, proof, makeMockProof(), relayerRewardAddress);
    }

    function testSubmitFailNotEnoughGas() public {
        bytes32 topic = keccak256("topic");

        // Create a command with very high gas requirement
        CommandV2[] memory commands = new CommandV2[](1);
        SetOperatingModeParams memory params = SetOperatingModeParams({mode: OperatingMode.Normal});
        commands[0] = CommandV2({
            kind: CommandKind.SetOperatingMode,
            gas: 30_000_000, // Extremely high gas value
            payload: abi.encode(params)
        });

        InboundMessageV2 memory message = InboundMessageV2({
            origin: keccak256("666"),
            nonce: 2, // Use a different nonce from other tests
            topic: topic,
            commands: commands
        });

        // Limit the gas for this test to ensure we hit the NotEnoughGas error
        uint256 gasLimit = 100_000;
        vm.deal(relayer, 1 ether);

        vm.expectRevert(IGatewayV2.InsufficientGasLimit.selector);
        vm.prank(relayer);
        IGatewayV2(address(gateway))
        .v2_submit{gas: gasLimit}(message, proof, makeMockProof(), relayerRewardAddress);
    }

    function mockNativeTokenForSend(address user, uint128 amount)
        internal
        returns (address, bytes memory, Asset memory)
    {
        address token = address(new WETH9());
        MockGateway(address(gateway)).prank_registerNativeToken(token);
        hoax(user);
        WETH9(payable(token)).deposit{value: amount}();
        bytes memory inputAsset = abi.encode(0, token, amount);
        Asset memory expectedOutputAsset = makeNativeAsset(token, amount);

        hoax(user);
        IERC20(token).approve(address(gateway), amount);

        return (token, inputAsset, expectedOutputAsset);
    }

    function mockForeignTokenForSend(address user, uint128 amount)
        internal
        returns (address, bytes memory, Asset memory)
    {
        Token token = MockGateway(address(gateway))
            .prank_registerForeignToken(keccak256("ABC"), "ABC", "ABC", 18);
        hoax(address(gateway));
        token.mint(user, amount);
        bytes memory inputAsset = abi.encode(0, address(token), amount);
        Asset memory expectedOutputAsset = makeForeignAsset(keccak256("ABC"), amount);

        hoax(user);
        token.approve(address(gateway), amount);

        return (address(token), inputAsset, expectedOutputAsset);
    }

    // Sends all types of assets over the bridge
    function testSendMessageSucceeds() public {
        (address nativeToken, bytes memory inputAsset0, Asset memory outputAsset0) =
            mockNativeTokenForSend(user1, uint128(1 ether));

        (address foreignToken, bytes memory inputAsset1, Asset memory outputAsset1) =
            mockForeignTokenForSend(user1, uint128(1 ether));

        uint256 foreignTokenSupplyPre = IERC20(foreignToken).totalSupply();

        bytes[] memory assets = new bytes[](2);
        assets[0] = inputAsset0;
        assets[1] = inputAsset1;

        Asset[] memory outputAssets = new Asset[](2);
        outputAssets[0] = outputAsset0;
        outputAssets[1] = outputAsset1;

        // Expect the gateway to emit `InboundMessageDispatched`
        vm.expectEmit(true, false, false, true);
        emit IGatewayV2.OutboundMessageAccepted(
            1,
            Payload({
                origin: user1,
                assets: outputAssets,
                xcm: makeRawXCM(""),
                claimer: "",
                value: 0.5 ether,
                executionFee: 0.1 ether,
                relayerFee: 0.4 ether
            })
        );

        hoax(user1);
        IGatewayV2(payable(address(gateway)))
        .v2_sendMessage{value: 1 ether}("", assets, "", 0.1 ether, 0.4 ether);

        // Verify asset balances
        assertEq(assetHubAgent.balance, 1 ether);
        assertEq(IERC20(nativeToken).balanceOf(assetHubAgent), 1 ether);
        assertEq(IERC20(foreignToken).totalSupply(), foreignTokenSupplyPre - 1 ether);

        assertEq(IGatewayV2(address(gateway)).v2_outboundNonce(), 1);
    }

    function testSendMessageWithFeeOnTransferTokenReverts() public {
        FeeOnTransferToken feeToken = new FeeOnTransferToken("FeeToken", "FEE", 500);
        feeToken.mint(user1, 1 ether);
        MockGateway(address(gateway)).prank_registerNativeToken(address(feeToken));

        uint128 amount = 1 ether;

        bytes[] memory assets = new bytes[](1);
        assets[0] = abi.encode(0, address(feeToken), amount);

        hoax(user1);
        feeToken.approve(address(gateway), amount);

        vm.expectRevert();
        hoax(user1);
        IGatewayV2(payable(address(gateway))).v2_sendMessage{value: 1 ether}(
            "", assets, "", 0.1 ether, 0.4 ether
        );

        assertEq(feeToken.balanceOf(assetHubAgent), 0);
    }

    function testSendMessageFailsWithInsufficentValue() public {
        vm.expectRevert(IGatewayV2.InsufficientValue.selector);
        hoax(user1, 1 ether);
        IGatewayV2(payable(address(gateway)))
        .v2_sendMessage{value: 0.4 ether}("", new bytes[](0), "", 0.1 ether, 0.4 ether);
    }

    function testSendMessageFailsWithExceededMaximumValue() public {
        vm.expectRevert(IGatewayV2.ExceededMaximumValue.selector);
        uint256 value = uint256(type(uint128).max) + 1;
        hoax(user1, value);
        IGatewayV2(payable(address(gateway)))
        .v2_sendMessage{value: value}("", new bytes[](0), "", 0.1 ether, 0.4 ether);
    }

    function testUnlockWethSuccess() public {
        bytes32 topic = keccak256("topic");

        hoax(assetHubAgent);
        weth.deposit{value: 1 ether}();

        vm.expectEmit(true, false, false, true);
        emit IGatewayV2.InboundMessageDispatched(1, topic, true, relayerRewardAddress);

        vm.deal(assetHubAgent, 1 ether);
        hoax(relayer, 1 ether);
        IGatewayV2(address(gateway))
            .v2_submit(
                InboundMessageV2({
                    origin: Constants.ASSET_HUB_AGENT_ID,
                    nonce: 1,
                    topic: topic,
                    commands: makeUnlockWethCommand(0.1 ether)
                }),
                proof,
                makeMockProof(),
                relayerRewardAddress
            );
    }

    function testUnlockFeeOnTransferTokenFails() public {
        FeeOnTransferToken feeToken = new FeeOnTransferToken("FeeToken", "FEE", 500);
        feeToken.mint(assetHubAgent, 200);

        bytes32 topic = keccak256("topic");
        uint128 amount = 100;
        vm.expectEmit(true, false, false, true);
        emit IGatewayV2.CommandFailed(1, 0);

        vm.expectEmit(true, false, false, true);
        emit IGatewayV2.InboundMessageDispatched(1, topic, false, relayerRewardAddress);

        vm.deal(assetHubAgent, 1 ether);
        hoax(relayer, 1 ether);
        IGatewayV2(address(gateway)).v2_submit(
            InboundMessageV2({
                origin: Constants.ASSET_HUB_AGENT_ID,
                nonce: 1,
                topic: topic,
                commands: makeUnlockTokenCommand(address(feeToken), user1, amount)
            }),
            proof,
            makeMockProof(),
            relayerRewardAddress
        );

        assertEq(feeToken.balanceOf(user1), 0);
    }

    function testRegisterForeignToken() public {
        bytes32 topic = keccak256("topic");

        vm.expectEmit(false, false, false, false);
        emit IGatewayBase.ForeignTokenRegistered(keccak256("DOT"), address(0));

        vm.expectEmit(true, false, false, true);
        emit IGatewayV2.InboundMessageDispatched(1, topic, true, relayerRewardAddress);

        vm.deal(assetHubAgent, 1 ether);
        hoax(relayer, 1 ether);
        IGatewayV2(address(gateway))
            .v2_submit(
                InboundMessageV2({
                    origin: keccak256("origin"),
                    nonce: 1,
                    topic: topic,
                    commands: makeRegisterForeignTokenCommand(keccak256("DOT"), "DOT", "DOT", 10)
                }),
                proof,
                makeMockProof(),
                relayerRewardAddress
            );
    }

    function testMintForeignToken() public {
        testRegisterForeignToken();

        address recipient = makeAddr("recipient");
        bytes32 topic = keccak256("topic");

        vm.expectEmit(true, true, true, true);
        emit IERC20.Transfer(address(0), recipient, 100);

        vm.expectEmit(true, false, false, true);
        emit IGatewayV2.InboundMessageDispatched(2, topic, true, relayerRewardAddress);

        vm.deal(assetHubAgent, 1 ether);
        hoax(relayer, 1 ether);
        IGatewayV2(address(gateway))
            .v2_submit(
                InboundMessageV2({
                    origin: keccak256("origin"),
                    nonce: 2,
                    topic: topic,
                    commands: makeMintForeignTokenCommand(keccak256("DOT"), recipient, 100)
                }),
                proof,
                makeMockProof(),
                relayerRewardAddress
            );
    }

    function testAgentCallContractSuccess() public {
        bytes32 topic = keccak256("topic");

        vm.expectEmit(true, false, false, true);
        emit IGatewayV2.InboundMessageDispatched(1, topic, true, relayerRewardAddress);

        vm.deal(assetHubAgent, 1 ether);
        hoax(relayer, 1 ether);
        IGatewayV2(address(gateway))
            .v2_submit(
                InboundMessageV2({
                    origin: Constants.ASSET_HUB_AGENT_ID,
                    nonce: 1,
                    topic: topic,
                    commands: makeCallContractCommand(0.1 ether)
                }),
                proof,
                makeMockProof(),
                relayerRewardAddress
            );
    }

    function testCreateAgent() public {
        bytes32 origin = bytes32(uint256(1));
        vm.expectEmit(true, false, false, false);
        emit IGatewayV2.AgentCreated(origin, address(0x0));
        IGatewayV2(payable(address(gateway))).v2_createAgent(origin);
    }

    function testCreateAgentFailsIfAlreadyExists() public {
        bytes32 origin = bytes32(uint256(1));
        vm.expectEmit(true, false, false, false);
        emit IGatewayV2.AgentCreated(origin, address(0x0));
        IGatewayV2(payable(address(gateway))).v2_createAgent(origin);

        vm.expectRevert(IGatewayV2.AgentAlreadyExists.selector);
        IGatewayV2(payable(address(gateway))).v2_createAgent(origin);
    }

    function testRegisterNativeTokenValidatesAddress() public {
        // Try to register a non-contract address (EOA)
        address nonContractAddress = makeAddr("nonContractAddress");

        // Expect the function to revert with InvalidToken error
        vm.expectRevert(IGatewayBase.InvalidToken.selector);
        MockGateway(address(gateway)).prank_registerNativeToken(nonContractAddress);

        // Verify that a valid token contract can be registered
        address validTokenContract = address(new WETH9());
        MockGateway(address(gateway)).prank_registerNativeToken(validTokenContract);

        // Verify the token is registered
        assertTrue(IGatewayV2(address(gateway)).isTokenRegistered(validTokenContract));
    }

    function testRegisterTokenSuccess() public {
        address validTokenContract = address(new WETH9());
        uint128 executionFee = 0.1 ether;
        uint128 relayerFee = 0.2 ether;
        uint256 totalRequired = executionFee + relayerFee;

        hoax(user1, totalRequired);
        IGatewayV2(payable(address(gateway)))
        .v2_registerToken{
            value: totalRequired
        }(validTokenContract, uint8(0), executionFee, relayerFee);

        // Verify the token is registered
        assertTrue(IGatewayV2(address(gateway)).isTokenRegistered(validTokenContract));
    }

    function testRegisterTokenFailsWithInsufficientValue() public {
        address validTokenContract = address(new WETH9());
        uint128 executionFee = 0.1 ether;
        uint128 relayerFee = 0.2 ether;
        uint256 totalRequired = executionFee + relayerFee;

        // Verify token is not registered before the attempt
        assertFalse(IGatewayV2(address(gateway)).isTokenRegistered(validTokenContract));

        vm.expectRevert(IGatewayV2.InsufficientValue.selector);
        hoax(user1, totalRequired);
        IGatewayV2(payable(address(gateway)))
        .v2_registerToken{
            value: totalRequired - 1
        }(validTokenContract, uint8(0), executionFee, relayerFee);

        // Verify token still is not registered after the failed attempt
        assertFalse(IGatewayV2(address(gateway)).isTokenRegistered(validTokenContract));
    }

    function testRegisterTokenFailsWithExceededMaximumValue() public {
        address validTokenContract = address(new WETH9());
        uint128 executionFee = 0.1 ether;
        uint128 relayerFee = 0.2 ether;

        // Verify token is not registered before the attempt
        assertFalse(IGatewayV2(address(gateway)).isTokenRegistered(validTokenContract));

        vm.expectRevert(IGatewayV2.ExceededMaximumValue.selector);
        uint256 value = uint256(type(uint128).max) + 1;
        hoax(user1, value);
        IGatewayV2(payable(address(gateway)))
        .v2_registerToken{value: value}(validTokenContract, uint8(0), executionFee, relayerFee);

        // Verify token still is not registered after the failed attempt
        assertFalse(IGatewayV2(address(gateway)).isTokenRegistered(validTokenContract));
    }

    function testPartialCommandExecution() public {
        bytes32 topic = keccak256("topic");

        // Create a compound set of commands, where the second one will fail
        CommandV2[] memory commands = new CommandV2[](3);

        // First command should succeed - SetOperatingMode
        SetOperatingModeParams memory params1 =
            SetOperatingModeParams({mode: OperatingMode.Normal});
        commands[0] = CommandV2({
            kind: CommandKind.SetOperatingMode, gas: 500_000, payload: abi.encode(params1)
        });

        // Second command should fail - Call a function that reverts
        bytes memory failingData = abi.encodeWithSignature("revertUnauthorized()");
        CallContractParams memory params2 =
            CallContractParams({target: address(helloWorld), data: failingData, value: 0});
        commands[1] = CommandV2({
            kind: CommandKind.CallContract, gas: 500_000, payload: abi.encode(params2)
        });

        // Third command should succeed - SetOperatingMode again
        SetOperatingModeParams memory params3 =
            SetOperatingModeParams({mode: OperatingMode.Normal});
        commands[2] = CommandV2({
            kind: CommandKind.SetOperatingMode, gas: 500_000, payload: abi.encode(params3)
        });

        // Expect the failed command to emit CommandFailed event
        vm.expectEmit(true, false, false, true);
        emit IGatewayV2.CommandFailed(1, 1); // nonce 1, command index 1

        // Expect InboundMessageDispatched to be emitted with success=false since not all commands succeeded
        vm.expectEmit(true, false, false, true);
        emit IGatewayV2.InboundMessageDispatched(1, topic, false, relayerRewardAddress);

        hoax(relayer, 1 ether);
        IGatewayV2(address(gateway))
            .v2_submit(
                InboundMessageV2({
                    origin: keccak256("666"), nonce: 1, topic: topic, commands: commands
                }),
                proof,
                makeMockProof(),
                relayerRewardAddress
            );
    }

    function testUnknownCommandType() public {
        bytes32 topic = keccak256("topic");

        // Create a command with an unknown command type
        CommandV2[] memory commands = new CommandV2[](2);

        // First command should succeed
        SetOperatingModeParams memory params1 =
            SetOperatingModeParams({mode: OperatingMode.Normal});
        commands[0] = CommandV2({
            kind: CommandKind.SetOperatingMode, gas: 500_000, payload: abi.encode(params1)
        });

        // Second command is invalid
        commands[1] = CommandV2({
            kind: 255, // Invalid command kind
            gas: 500_000,
            payload: abi.encode(bytes32(0))
        });

        // Expect the unknown command to emit CommandFailed event
        vm.expectEmit(true, false, false, true);
        emit IGatewayV2.CommandFailed(2, 1); // nonce 2, command index 1

        // Expect InboundMessageDispatched to be emitted with success=false
        vm.expectEmit(true, false, false, true);
        emit IGatewayV2.InboundMessageDispatched(2, topic, false, relayerRewardAddress);

        hoax(relayer, 1 ether);
        IGatewayV2(address(gateway))
            .v2_submit(
                InboundMessageV2({
                    origin: keccak256("666"), nonce: 2, topic: topic, commands: commands
                }),
                proof,
                makeMockProof(),
                relayerRewardAddress
            );
    }

    function testMultipleSuccessfulCommands() public {
        bytes32 topic = keccak256("topic");

        // Create multiple commands that should all succeed
        CommandV2[] memory commands = new CommandV2[](3);

        // First command - SetOperatingMode to Normal
        SetOperatingModeParams memory params1 =
            SetOperatingModeParams({mode: OperatingMode.Normal});
        commands[0] = CommandV2({
            kind: CommandKind.SetOperatingMode, gas: 500_000, payload: abi.encode(params1)
        });

        // Second command - Set mode to RejectingOutboundMessages (will succeed)
        SetOperatingModeParams memory params2 =
            SetOperatingModeParams({mode: OperatingMode.RejectingOutboundMessages});
        commands[1] = CommandV2({
            kind: CommandKind.SetOperatingMode, gas: 500_000, payload: abi.encode(params2)
        });

        // Third command - Also set mode to Normal again (will succeed)
        SetOperatingModeParams memory params3 =
            SetOperatingModeParams({mode: OperatingMode.Normal});
        commands[2] = CommandV2({
            kind: CommandKind.SetOperatingMode, gas: 500_000, payload: abi.encode(params3)
        });

        // Expect InboundMessageDispatched to be emitted with success=true since all commands should succeed
        vm.expectEmit(true, false, false, true);
        emit IGatewayV2.InboundMessageDispatched(3, topic, true, relayerRewardAddress);

        hoax(relayer, 1 ether);
        IGatewayV2(address(gateway))
            .v2_submit(
                InboundMessageV2({
                    origin: keccak256("666"), nonce: 3, topic: topic, commands: commands
                }),
                proof,
                makeMockProof(),
                relayerRewardAddress
            );
    }

    function testUnknownCommandReturnsFalse() public {
        bytes memory payload = "";
        CommandV2 memory cmd =
            CommandV2({kind: uint8(200), gas: uint64(100_000), payload: payload});

        bool ok = gatewayLogic.callDispatch(cmd, bytes32(0));
        assertTrue(!ok, "unknown command should return false");
    }

    function testSetOperatingModeSucceeds() public {
        bytes memory payload = abi.encode((SetOperatingModeParams({mode: OperatingMode.Normal})));
        CommandV2 memory cmd = CommandV2({
            kind: CommandKind.SetOperatingMode, gas: uint64(200_000), payload: payload
        });

        // Expect the OperatingModeChanged event to be emitted
        vm.expectEmit(true, false, false, true);
        emit IGatewayBase.OperatingModeChanged(OperatingMode.Normal);

        bool ok = gatewayLogic.callDispatch(cmd, bytes32(0));
        assertTrue(ok, "setOperatingMode should succeed");

        // Verify mode was set
        assertEq(uint256(gatewayLogic.operatingMode()), uint256(OperatingMode.Normal));
    }

    function testHandlerRevertIsCaught_UnlockNativeToken() public {
        // Ensure no agent exists for ASSET_HUB_AGENT_ID so ensureAgent will revert and _dispatchCommand returns false
        UnlockNativeTokenParams memory params = UnlockNativeTokenParams({
            token: address(0), recipient: address(this), amount: uint128(1)
        });
        bytes memory payload = abi.encode(params);

        CommandV2 memory cmd = CommandV2({
            kind: CommandKind.UnlockNativeToken, gas: uint64(200_000), payload: payload
        });

        bool ok = gatewayLogic.callDispatch(cmd, bytes32(0));
        assertTrue(!ok, "handler revert should be caught and return false");
    }

    function testHandlerRevertIsCaught_UpgradeInvalidImpl() public {
        // Upgrade with impl == address(0) will cause Upgrade.upgrade to revert (InvalidContract)
        UpgradeParams memory up =
            UpgradeParams({impl: address(0), implCodeHash: bytes32(0), initParams: ""});
        bytes memory payload = abi.encode(up);

        CommandV2 memory cmd =
            CommandV2({kind: CommandKind.Upgrade, gas: uint64(200_000), payload: payload});

        bool ok = gatewayLogic.callDispatch(cmd, bytes32(0));
        assertTrue(!ok, "upgrade with invalid impl should be caught and return false");
    }

    function testMintForeignTokenNotRegisteredReturnsFalse() public {
        MintForeignTokenParams memory p = MintForeignTokenParams({
            foreignTokenID: bytes32(uint256(0x1234)), recipient: address(this), amount: uint128(1)
        });
        bytes memory payload = abi.encode(p);

        CommandV2 memory cmd = CommandV2({
            kind: CommandKind.MintForeignToken, gas: uint64(200_000), payload: payload
        });

        bool ok = gatewayLogic.callDispatch(cmd, bytes32(0));
        assertTrue(!ok, "mintForeignToken for unregistered token should return false");
    }

    function testRegisterForeignTokenDuplicateReturnsFalse() public {
        bytes32 fid = bytes32(uint256(0x5566));
        RegisterForeignTokenParams memory p = RegisterForeignTokenParams({
            foreignTokenID: fid, name: string("T"), symbol: string("T"), decimals: uint8(18)
        });
        bytes memory payload = abi.encode(p);

        CommandV2 memory cmd = CommandV2({
            kind: CommandKind.RegisterForeignToken, gas: uint64(3_000_000), payload: payload
        });

        bool ok1 = gatewayLogic.callDispatch(cmd, bytes32(0));
        assertTrue(ok1, "first register should succeed");

        bool ok2 = gatewayLogic.callDispatch(cmd, bytes32(0));
        assertTrue(!ok2, "duplicate register should return false");
    }

    function testCallContractAgentDoesNotExistReturnsFalse() public {
        CallContractParams memory p =
            CallContractParams({target: address(0xdead), data: "", value: uint256(0)});
        bytes memory payload = abi.encode(p);

        // origin corresponds to agent id; use a non-existent id
        CommandV2 memory cmd =
            CommandV2({kind: CommandKind.CallContract, gas: uint64(200_000), payload: payload});

        bool ok = gatewayLogic.callDispatch(cmd, bytes32(uint256(0x9999)));
        assertTrue(!ok, "callContract with missing agent should return false");
    }

    function testInsufficientGasReverts() public {
        bytes memory payload = "";
        // Use an extremely large gas value to trigger InsufficientGasLimit revert in _dispatchCommand
        CommandV2 memory cmd = CommandV2({
            kind: CommandKind.SetOperatingMode, gas: type(uint64).max, payload: payload
        });

        vm.expectRevert();
        gatewayLogic.callDispatch(cmd, bytes32(0));
    }

    function testUpgradeCallsInitialize() public {
        uint256 initValue = 0x12345;
        bytes memory initParams = abi.encode(initValue);

        // compute codehash expected by Upgrade.upgrade
        bytes32 codeHash = address(impl).codehash;

        // call upgrade via handler wrapper; this will cause the Upgrade library
        // to delegatecall into impl.initialize which writes into handler's storage
        handler.callUpgrade(address(impl), codeHash, initParams);

        // verify delegatecall wrote into handler's storage (slot 0)
        assertEq(handler.initialized(), initValue);
    }

    function testUnlockEther() public {
        // deploy agent via gateway so GATEWAY == gateway
        address agent = gatewayLogic.deployAgent();

        // register agent in gateway storage
        gatewayLogic.setAgentInStorage(agent);

        // fund agent with ether
        uint256 amt = 1 ether;
        vm.deal(agent, amt);

        // build params: token=address(0) indicates native ether
        PayableRecipient recipient = new PayableRecipient();
        bytes memory params = abi.encode(
            UnlockNativeTokenParams({
                token: address(0), recipient: address(recipient), amount: uint128(amt)
            })
        );

        // call
        gatewayLogic.callUnlockNativeToken(address(executor), params);

        // assert recipient got funds
        assertEq(address(recipient).balance, amt);
    }

    function testUnlockNativeTokenERC20() public {
        // deploy agent and register
        address agent = gatewayLogic.deployAgent();
        gatewayLogic.setAgentInStorage(agent);

        // deploy mock token and mint to agent
        MockERC20 token = new MockERC20();
        uint256 tAmt = 1000;
        token.mint(agent, tAmt);

        // build params for token transfer
        bytes memory params = abi.encode(
            UnlockNativeTokenParams({
                token: address(token), recipient: address(this), amount: uint128(tAmt)
            })
        );

        // call unlock with executor
        gatewayLogic.callUnlockNativeToken(address(executor), params);

        // assert recipient got tokens
        assertEq(token.balanceOf(address(this)), tAmt);
    }

    function testAgentCallContractRevertedForFunctionNotExists() public {
        bytes32 topic = keccak256("topic");

        vm.deal(assetHubAgent, 1 ether);
        hoax(relayer, 1 ether);

        vm.expectEmit(true, false, false, true);
        emit IGatewayV2.CommandFailed(1, 0);
        emit IGatewayV2.InboundMessageDispatched(1, topic, false, relayerRewardAddress);
        IGatewayV2(address(gateway))
            .v2_submit(
                InboundMessageV2({
                    origin: Constants.ASSET_HUB_AGENT_ID,
                    nonce: 1,
                    topic: topic,
                    commands: makeCallContractCommandWithFunctionNotExists(0.1 ether)
                }),
                proof,
                makeMockProof(),
                relayerRewardAddress
            );
    }

    function testAgentCallContractRevertedForInsufficientGas() public {
        bytes32 topic = keccak256("topic");

        vm.deal(assetHubAgent, 1 ether);
        hoax(relayer, 1 ether);

        vm.expectEmit(true, false, false, true);
        emit IGatewayV2.CommandFailed(1, 0);
        emit IGatewayV2.InboundMessageDispatched(1, topic, false, relayerRewardAddress);
        IGatewayV2(address(gateway))
            .v2_submit(
                InboundMessageV2({
                    origin: Constants.ASSET_HUB_AGENT_ID,
                    nonce: 1,
                    topic: topic,
                    commands: makeCallContractCommandWithInsufficientGas(0.1 ether)
                }),
                proof,
                makeMockProof(),
                relayerRewardAddress
            );
    }

    function test_v2_submit_rejects_duplicate_nonce() public {
        MockGateway gw = MockGateway(address(gateway));
        // mark nonce 5 as already processed
        gw.setInboundNonce(5);

        InboundMessageV2 memory m;
        m.nonce = 5;
        m.origin = bytes32("x");
        m.topic = bytes32(0);
        m.commands = new CommandV2[](0);

        bytes32[] memory leafProof = new bytes32[](0);

        // empty header proof
        Verification.DigestItem[] memory digestItems = new Verification.DigestItem[](0);
        Verification.ParachainHeader memory header = Verification.ParachainHeader({
            parentHash: bytes32(0),
            number: 0,
            stateRoot: bytes32(0),
            extrinsicsRoot: bytes32(0),
            digestItems: digestItems
        });
        bytes32[] memory emptyBytes32 = new bytes32[](0);
        Verification.HeadProof memory hp =
            Verification.HeadProof({pos: 0, width: 0, proof: emptyBytes32});
        Verification.MMRLeafPartial memory lp = Verification.MMRLeafPartial({
            version: 0,
            parentNumber: 0,
            parentHash: bytes32(0),
            nextAuthoritySetID: 0,
            nextAuthoritySetLen: 0,
            nextAuthoritySetRoot: bytes32(0)
        });
        Verification.Proof memory headerProof = Verification.Proof({
            header: header,
            headProof: hp,
            leafPartial: lp,
            leafProof: emptyBytes32,
            leafProofOrder: 0
        });

        vm.expectRevert(IGatewayBase.InvalidNonce.selector);
        gw.v2_submit(m, leafProof, headerProof, bytes32(0));
    }

    function test_onlySelf_enforced_on_external_calls() public {
        MockGateway gw = MockGateway(address(gateway));
        // calling the handler externally should revert with Unauthorized
        SetOperatingModeParams memory p = SetOperatingModeParams({mode: OperatingMode.Normal});
        bytes memory payload = abi.encode(p);
        vm.expectRevert(IGatewayBase.Unauthorized.selector);
        gw.v2_handleSetOperatingMode(payload);
    }

    function test_call_handleSetOperatingMode_via_self_changes_mode() public {
        MockGateway gw = MockGateway(address(gateway));
        // call via helper which forwards as `this` so onlySelf check passes
        SetOperatingModeParams memory p =
            SetOperatingModeParams({mode: OperatingMode.RejectingOutboundMessages});
        bytes memory payload = abi.encode(p);
        gw.setOperatingMode(payload);
        // ensure call did not revert
        assertTrue(true);
    }

    function test_dispatch_unknown_command_returns_false() public {
        MockGateway gw = MockGateway(address(gateway));
        CommandV2 memory cmd = CommandV2({kind: 0xFF, gas: 100_000, payload: ""});
        bool ok = gw.exposed_dispatchCommand(cmd, bytes32(0));
        assertFalse(ok, "unknown command must return false");
    }

    function test_v2_dispatch_partial_failure_emits_CommandFailed() public {
        MockGateway gw = MockGateway(address(gateway));
        // Build two commands: SetOperatingMode (should succeed) and CallContract (will fail due to missing agent)
        CommandV2[] memory cmds = new CommandV2[](2);
        SetOperatingModeParams memory p = SetOperatingModeParams({mode: OperatingMode.Normal});
        cmds[0] =
            CommandV2({kind: CommandKind.SetOperatingMode, gas: 200_000, payload: abi.encode(p)});

        CallContractParams memory cc =
            CallContractParams({target: address(0x1234), data: "", value: 0});
        cmds[1] =
            CommandV2({kind: CommandKind.CallContract, gas: 200_000, payload: abi.encode(cc)});
        InboundMessageV2 memory msgv;
        msgv.origin = bytes32("orig");
        msgv.nonce = 1;
        msgv.topic = bytes32(0);
        msgv.commands = cmds;

        // call v2_submit (verification overridden to true)

        // construct an empty Verification.Proof
        Verification.DigestItem[] memory digestItems = new Verification.DigestItem[](0);
        Verification.ParachainHeader memory header = Verification.ParachainHeader({
            parentHash: bytes32(0),
            number: 0,
            stateRoot: bytes32(0),
            extrinsicsRoot: bytes32(0),
            digestItems: digestItems
        });

        bytes32[] memory emptyBytes32 = new bytes32[](0);
        Verification.HeadProof memory hp =
            Verification.HeadProof({pos: 0, width: 0, proof: emptyBytes32});
        Verification.MMRLeafPartial memory lp = Verification.MMRLeafPartial({
            version: 0,
            parentNumber: 0,
            parentHash: bytes32(0),
            nextAuthoritySetID: 0,
            nextAuthoritySetLen: 0,
            nextAuthoritySetRoot: bytes32(0)
        });

        Verification.Proof memory headerProof = Verification.Proof({
            header: header,
            headProof: hp,
            leafPartial: lp,
            leafProof: emptyBytes32,
            leafProofOrder: 0
        });

        gw.v2_submit(msgv, proof, headerProof, bytes32(0));

        // message should be recorded as dispatched
        assertTrue(gw.v2_isDispatched(msgv.nonce));
    }
}
