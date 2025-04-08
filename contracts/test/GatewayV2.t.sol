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

contract GatewayV2Test is Test {
    // Emitted when token minted/burnt/transfered
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

    HelloWorld public helloWorld;

    event SaidHello(string indexed message);

    function setUp() public {
        weth = new WETH9();
        AgentExecutor executor = new AgentExecutor();
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
            kind: CommandKind.SetOperatingMode,
            gas: 500_000,
            payload: abi.encode(params)
        });
        return commands;
    }

    function makeUnlockWethCommand(uint128 value) public view returns (CommandV2[] memory) {
        UnlockNativeTokenParams memory params =
            UnlockNativeTokenParams({token: address(weth), recipient: relayer, amount: value});
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

    /**
     * Message Verification
     */
    function testSubmitHappyPath() public {
        bytes32 topic = keccak256("topic");

        // Expect the gateway to emit `InboundMessageDispatched`
        vm.expectEmit(true, false, false, true);
        emit IGatewayV2.InboundMessageDispatched(1, topic, true, relayerRewardAddress);

        hoax(relayer, 1 ether);
        IGatewayV2(address(gateway)).v2_submit(
            InboundMessageV2({
                origin: keccak256("666"),
                nonce: 1,
                topic: topic,
                commands: makeMockCommand()
            }),
            proof,
            makeMockProof(),
            relayerRewardAddress
        );
    }

    function testSubmitFailInvalidNonce() public {
        bytes32 topic = keccak256("topic");

        InboundMessageV2 memory message = InboundMessageV2({
            origin: keccak256("666"),
            nonce: 1,
            topic: topic,
            commands: makeMockCommand()
        });

        hoax(relayer, 1 ether);
        IGatewayV2(address(gateway)).v2_submit(
            message, proof, makeMockProof(), relayerRewardAddress
        );

        vm.expectRevert(IGatewayBase.InvalidNonce.selector);
        hoax(relayer, 1 ether);
        IGatewayV2(address(gateway)).v2_submit(
            message, proof, makeMockProof(), relayerRewardAddress
        );
    }

    function testSubmitFailInvalidProof() public {
        bytes32 topic = keccak256("topic");

        InboundMessageV2 memory message = InboundMessageV2({
            origin: keccak256("666"),
            nonce: 1,
            topic: topic,
            commands: makeMockCommand()
        });

        MockGateway(address(gateway)).setCommitmentsAreVerified(false);
        vm.expectRevert(IGatewayBase.InvalidProof.selector);

        hoax(relayer, 1 ether);
        IGatewayV2(address(gateway)).v2_submit(
            message, proof, makeMockProof(), relayerRewardAddress
        );
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
        Token token = MockGateway(address(gateway)).prank_registerForeignToken(
            keccak256("ABC"), "ABC", "ABC", 18
        );
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
        IGatewayV2(payable(address(gateway))).v2_sendMessage{value: 1 ether}(
            "", assets, "", 0.1 ether, 0.4 ether
        );

        // Verify asset balances
        assertEq(assetHubAgent.balance, 1 ether);
        assertEq(IERC20(nativeToken).balanceOf(assetHubAgent), 1 ether);
        assertEq(IERC20(foreignToken).totalSupply(), foreignTokenSupplyPre - 1 ether);
    }

    function testSendMessageFailsWithInsufficentValue() public {
        vm.expectRevert(IGatewayV2.InsufficientValue.selector);
        hoax(user1, 1 ether);
        IGatewayV2(payable(address(gateway))).v2_sendMessage{value: 0.4 ether}(
            "", new bytes[](0), "", 0.1 ether, 0.4 ether
        );
    }

    function testSendMessageFailsWithExceededMaximumValue() public {
        vm.expectRevert(IGatewayV2.ExceededMaximumValue.selector);
        uint256 value = uint256(type(uint128).max) + 1;
        hoax(user1, value);
        IGatewayV2(payable(address(gateway))).v2_sendMessage{value: value}(
            "", new bytes[](0), "", 0.1 ether, 0.4 ether
        );
    }

    function testUnlockWethSuccess() public {
        bytes32 topic = keccak256("topic");

        hoax(assetHubAgent);
        weth.deposit{value: 1 ether}();

        vm.expectEmit(true, false, false, true);
        emit IGatewayV2.InboundMessageDispatched(1, topic, true, relayerRewardAddress);

        vm.deal(assetHubAgent, 1 ether);
        hoax(relayer, 1 ether);
        IGatewayV2(address(gateway)).v2_submit(
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

    function testRegisterForeignToken() public {
        bytes32 topic = keccak256("topic");

        vm.expectEmit(false, false, false, false);
        emit IGatewayBase.ForeignTokenRegistered(keccak256("DOT"), address(0));

        vm.expectEmit(true, false, false, true);
        emit IGatewayV2.InboundMessageDispatched(1, topic, true, relayerRewardAddress);

        vm.deal(assetHubAgent, 1 ether);
        hoax(relayer, 1 ether);
        IGatewayV2(address(gateway)).v2_submit(
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
        IGatewayV2(address(gateway)).v2_submit(
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
        IGatewayV2(address(gateway)).v2_submit(
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

    function testCommandBatchAtomicity() public {
        testRegisterForeignToken();

        bytes32 topic = keccak256("topic");
        bytes32 dotTokenID = keccak256("DOT");
        address recipient = makeAddr("recipient");

        // Get the token address for DOT that was registered in testRegisterForeignToken
        address dotTokenAddress = IGatewayV2(address(gateway)).tokenAddressOf(dotTokenID);
        Token dotToken = Token(dotTokenAddress);

        // Check initial token supply and recipient balance
        uint256 initialTotalSupply = dotToken.totalSupply();
        uint256 initialRecipientBalance = dotToken.balanceOf(recipient);
        assertEq(initialRecipientBalance, 0, "Initial recipient balance should be 0");

        // Create a batch of commands where:
        // 1. First command mints DOT tokens to the recipient (should succeed)
        // 2. Second command contains an unknown command kind (should fail)
        CommandV2[] memory commands = new CommandV2[](2);

        // First command: Mint tokens (valid)
        MintForeignTokenParams memory mintParams = MintForeignTokenParams(dotTokenID, recipient, 100);
        commands[0] = CommandV2({
            kind: CommandKind.MintForeignToken,
            gas: 100_000,
            payload: abi.encode(mintParams)
        });

        // Second command: Invalid command kind
        commands[1] = CommandV2({
            kind: CommandKind(255), // Using 255 as an invalid command kind
            gas: 100_000,
            payload: bytes("invalid")
        });

        // Execute the batch and expect failure
        vm.expectEmit(true, false, false, true);
        emit IGatewayV2.InboundMessageDispatched(3, topic, false, relayerRewardAddress);

        hoax(relayer, 1 ether);
        IGatewayV2(address(gateway)).v2_submit(
            InboundMessageV2({
                origin: keccak256("origin"),
                nonce: 3, // Use a new nonce
                topic: topic,
                commands: commands
            }),
            proof,
            makeMockProof(),
            relayerRewardAddress
        );

        // Verify that no state change occurred (the mint operation should have been rolled back)
        uint256 finalTotalSupply = dotToken.totalSupply();
        uint256 finalRecipientBalance = dotToken.balanceOf(recipient);

        assertEq(finalTotalSupply, initialTotalSupply, "Total supply should not change");
        assertEq(finalRecipientBalance, initialRecipientBalance, "Recipient balance should not change");
    }

    // Test to verify that multiple commands in a batch are executed atomically
    function testCommandBatchAtomicSuccess() public {
        // Setup: Register a foreign token (DOT) that we'll use to test state changes
        testRegisterForeignToken();

        bytes32 topic = keccak256("topic");
        bytes32 dotTokenID = keccak256("DOT");
        address recipient1 = makeAddr("recipient1");
        address recipient2 = makeAddr("recipient2");

        // Get the token address for DOT that was registered in testRegisterForeignToken
        address dotTokenAddress = IGatewayV2(address(gateway)).tokenAddressOf(dotTokenID);
        Token dotToken = Token(dotTokenAddress);

        // Check initial balances
        uint256 initialTotalSupply = dotToken.totalSupply();
        assertEq(dotToken.balanceOf(recipient1), 0, "Initial recipient1 balance should be 0");
        assertEq(dotToken.balanceOf(recipient2), 0, "Initial recipient2 balance should be 0");

        // Create a batch with multiple valid commands
        CommandV2[] memory commands = new CommandV2[](2);

        // First command: Mint tokens to recipient1
        MintForeignTokenParams memory mintParams1 = MintForeignTokenParams(dotTokenID, recipient1, 100);
        commands[0] = CommandV2({
            kind: CommandKind.MintForeignToken,
            gas: 100_000,
            payload: abi.encode(mintParams1)
        });

        // Second command: Mint tokens to recipient2
        MintForeignTokenParams memory mintParams2 = MintForeignTokenParams(dotTokenID, recipient2, 200);
        commands[1] = CommandV2({
            kind: CommandKind.MintForeignToken,
            gas: 100_000,
            payload: abi.encode(mintParams2)
        });

        // Execute the batch and expect success
        vm.expectEmit(true, false, false, true);
        emit IGatewayV2.InboundMessageDispatched(3, topic, true, relayerRewardAddress);

        hoax(relayer, 1 ether);
        IGatewayV2(address(gateway)).v2_submit(
            InboundMessageV2({
                origin: keccak256("origin"),
                nonce: 3, // Use a new nonce
                topic: topic,
                commands: commands
            }),
            proof,
            makeMockProof(),
            relayerRewardAddress
        );

        // Verify that both state changes occurred
        uint256 finalTotalSupply = dotToken.totalSupply();

        assertEq(finalTotalSupply, initialTotalSupply + 300, "Total supply should increase by 300");
        assertEq(dotToken.balanceOf(recipient1), 100, "Recipient1 should have 100 tokens");
        assertEq(dotToken.balanceOf(recipient2), 200, "Recipient2 should have 200 tokens");
    }
}
