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
import {
    MultiAddress,
    multiAddressFromBytes32,
    multiAddressFromBytes20
} from "../src/MultiAddress.sol";
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
            rescueOperator: 0x4B8a782D4F03ffcB7CE1e95C5cfe5BFCb2C8e967,
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

    function recipientAddress32() internal pure returns (MultiAddress memory) {
        return multiAddressFromBytes32(keccak256("recipient"));
    }

    function recipientAddress20() internal pure returns (MultiAddress memory) {
        return multiAddressFromBytes20(bytes20(keccak256("recipient")));
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

    function testEncodeDecodeMessageV2() public {
        bytes32 topic = bytes32(uint256(1));

        UnlockNativeTokenParams memory params = UnlockNativeTokenParams({
            token: 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2,
            recipient: 0xEDa338E4dC46038493b885327842fD3E301CaB39,
            amount: 1_000_000
        });
        bytes memory encoded = abi.encode(params);
        CommandV2[] memory commands = new CommandV2[](1);
        commands[0] =
            CommandV2({kind: CommandKind.UnlockNativeToken, gas: 100_000, payload: encoded});
        InboundMessageV2 memory message = InboundMessageV2({
            origin: bytes32(uint256(1000)),
            nonce: 1,
            topic: topic,
            commands: commands
        });
        bytes memory rawBytes = abi.encode(message);

        //From OutboundQueueV2
        bytes memory data = abi.encodePacked(
            hex"000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000003e800000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000186a000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000060000000000000000000000000c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2000000000000000000000000eda338e4dc46038493b885327842fd3e301cab3900000000000000000000000000000000000000000000000000000000000f4240"
        );
        assertEq(data, rawBytes);
        InboundMessageV2 memory result = abi.decode(data, (InboundMessageV2));
        assertEq(result.nonce, 1);
        assertEq(result.commands.length, 1);
    }

    function testSubmitRegisterPNA() public {
        //From Relayer V2
        bytes memory data = abi.encodePacked(
            hex"000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000003e80000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000030000000000000000000000000000000000000000000000000000000000124f80000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000000c00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"
        );
        InboundMessageV2 memory message = abi.decode(data, (InboundMessageV2));
        assertEq(message.nonce, 1);
        assertEq(message.commands.length, 1);
        hoax(relayer, 1 ether);
        vm.expectEmit(true, false, false, false);
        emit IGatewayBase.ForeignTokenRegistered(
            bytes32(0x0000000000000000000000000000000000000000000000000000000000000001),
            address(0x7ff9C67c93D9f7318219faacB5c619a773AFeF6A)
        );
        IGatewayV2(address(gateway)).v2_submit(
            message, proof, makeMockProof(), relayerRewardAddress
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
        emit IGatewayBase.AgentCreated(origin, address(0x0));
        IGatewayV2(payable(address(gateway))).v2_createAgent(origin);
    }
}
