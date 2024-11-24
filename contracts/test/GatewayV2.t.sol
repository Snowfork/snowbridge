// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.25;

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

import {AgentExecutor} from "../src/AgentExecutor.sol";
import {Agent} from "../src/Agent.sol";
import {Verification} from "../src/Verification.sol";
import {SubstrateTypes} from "./../src/SubstrateTypes.sol";
import {
    MultiAddress,
    multiAddressFromBytes32,
    multiAddressFromBytes20
} from "../src/MultiAddress.sol";
import {
    OperatingMode,
    ParaID,
    CommandV2,
    CommandKind,
    InboundMessageV2
} from "../src/Types.sol";

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
    CallContractParams
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

    WETH9 public token;

    address public user1;
    address public user2;

    // tokenID for DOT
    bytes32 public dotTokenID;

    HelloWorld public helloWorld;

    event SaidHello(string indexed message);

    function setUp() public {
        token = new WETH9();
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
            maxDestinationFee: 1e11,
            weth: address(token)
        });
        gateway = new GatewayProxy(address(gatewayLogic), abi.encode(config));
        MockGateway(address(gateway)).setCommitmentsAreVerified(true);

        SetOperatingModeParams memory params =
            SetOperatingModeParams({mode: OperatingMode.Normal});
        MockGateway(address(gateway)).v1_handleSetOperatingMode_public(
            abi.encode(params)
        );

        assetHubAgent =
            IGatewayV2(address(gateway)).agentOf(Constants.ASSET_HUB_AGENT_ID);

        // fund the message relayer account
        relayer = makeAddr("relayer");

        // Features

        user1 = makeAddr("user1");
        user2 = makeAddr("user2");

        // create tokens for account 1
        hoax(user1);
        token.deposit{value: 500}();

        // create tokens for account 2
        hoax(user2);
        token.deposit{value: 500}();

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
        commands[0] =
            CommandV2({kind: CommandKind.CreateAgent, gas: 500_000, payload: ""});
        return commands;
    }

    function makeCallContractCommand(uint256 value) public view returns (CommandV2[] memory) {
        bytes memory data = abi.encodeWithSignature("sayHello(string)", "World");
        CallContractParams memory params = CallContractParams({
            target: address(helloWorld),
            data: data,
            value: value
        });
        bytes memory payload = abi.encode(params);

        CommandV2[] memory commands = new CommandV2[](1);
        commands[0] =
            CommandV2({kind: CommandKind.CallContract, gas: 500_000, payload: payload});
        return commands;
    }

    /**
     * Message Verification
     */
    function testSubmitHappyPath() public {
        // Expect the gateway to emit `InboundMessageDispatched`
        vm.expectEmit(true, false, false, true);
        emit IGatewayV2.InboundMessageDispatched(1, true, relayerRewardAddress);

        hoax(relayer, 1 ether);
        IGatewayV2(address(gateway)).v2_submit(
            InboundMessageV2({
                origin: keccak256("666"),
                nonce: 1,
                commands: makeMockCommand()
            }),
            proof,
            makeMockProof(),
            relayerRewardAddress
        );
    }

    function testSubmitFailInvalidNonce() public {
        InboundMessageV2 memory message = InboundMessageV2({
            origin: keccak256("666"),
            nonce: 1,
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
        InboundMessageV2 memory message = InboundMessageV2({
            origin: keccak256("666"),
            nonce: 1,
            commands: makeMockCommand()
        });

        MockGateway(address(gateway)).setCommitmentsAreVerified(false);
        vm.expectRevert(IGatewayBase.InvalidProof.selector);

        hoax(relayer, 1 ether);
        IGatewayV2(address(gateway)).v2_submit(
            message, proof, makeMockProof(), relayerRewardAddress
        );
    }

    function testSendEther() public {
        bytes[] memory assets = new bytes[](1);
        assets[0] = abi.encode(0, 0.5 ether);

        hoax(user1, 1 ether);
        IGatewayV2(payable(address(gateway))).v2_sendMessage{value: 1 ether}(
            "", assets, ""
        );

        // Agent balance should be 0.5 + 0.5
        assertEq(token.balanceOf(assetHubAgent), 1 ether);
    }

    function testEncodeDecodeMessageV2() public {
        UnlockNativeTokenParams memory params = UnlockNativeTokenParams({
            token: 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2,
            recipient: 0xEDa338E4dC46038493b885327842fD3E301CaB39,
            amount: 1_000_000
        });
        bytes memory encoded = abi.encode(params);
        CommandV2[] memory commands = new CommandV2[](1);
        commands[0] = CommandV2({
            kind: CommandKind.UnlockNativeToken,
            gas: 100_000,
            payload: encoded
        });
        InboundMessageV2 memory message = InboundMessageV2({
            origin: bytes32(uint256(1000)),
            nonce: 1,
            commands: commands
        });
        bytes memory rawBytes = abi.encode(message);

        //From OutboundQueueV2
        bytes memory data = abi.encodePacked(
            hex"000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000003e80000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000186a000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000060000000000000000000000000c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2000000000000000000000000eda338e4dc46038493b885327842fd3e301cab3900000000000000000000000000000000000000000000000000000000000f4240"
        );
        assertEq(data, rawBytes);
        InboundMessageV2 memory result = abi.decode(data, (InboundMessageV2));
        assertEq(result.nonce, 1);
        assertEq(result.commands.length, 1);
    }

    function testSubmitRegisterPNA() public {
        //From Relayer V2
        bytes memory data = abi.encodePacked(
            hex"000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000002c000000000000000000000000000000000000000000000000000000000000002e0d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d0000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000030000000000000000000000000000000000000000000000000000000000124f800000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000012000000000000000000000000000000000000000000000000000000000000000209441dceeeffa7e032eedaccf9b7632e60e86711551a82ffbbb0dda8afd9e4ef7000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000c0000000000000000000000000000000000000000000000000000000000000000c0000000000000000000000000000000000000000000000000000000000000020776e6401010101010101010101010101010101010101010101010101010101010000000000000000000000000000000000000000000000000000000000000020776e640101010101010101010101010101010101010101010101010101010101000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001400000000000000000000000000000000000000000000000000000000000000560000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000357fd8184830ac484da672bb9741f2b506f8503a1ac7e1cc48a47e1e0658ff382000000000000000000000000000000000000000000000000000000000000000030000000000000000000000000000000000000000000000000000000000000004bb47b15c3d292b996aa030ac4daa69f9ba4d07bba2846ad786fa9203cd7bba0e00000000000000000000000000000000000000000000000000000000000006200000000000000000000000000000000000000000000000000000000000000435620015748e29b5ddf1606eaa103b741d1b42cb5b6494e00e723415d50a8737f9000000000000000000000000000000000000000000000000000000000000001de0ab0b550176ac143add9e5dad46bf60f3ff09e6910ddcd8cbd0bd95d9af99c458e9af973605f8977d29a97331322ae8b7df3ee387660f4f78c391a8910b06c400000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000012000000000000000000000000000000000000000000000000000000000000001e000000000000000000000000000000000000000000000000000000000000002a00000000000000000000000000000000000000000000000000000000000000006617572610000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000008b1cb3111000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004525053520000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000021f2fa13b3689581ab90e8be6189a6a182bc5f18e67b0cd4d634780d57eda5e7abc800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000002100f97fc21a954ee5ce488329357e198fe1842be336d835186429d0d63b50ae6a43000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000005617572610000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000040deba90c498fc585fab36e1c5e916453b5bf9b1db0053740f456c8bb3cd319818bae31266b4091b569265cc091096c3759f0a37ab3c3ff47c49db45179454588c0000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000002c50ec1a82811ddd645332a2ba5b83d54e4b139cb6cccd8c243fe70ff46ee99107c8edcedf83c0365d02f0f0e81db16f5d26af633b1649447ae6227eb25f2d4d1000000000000000000000000000000000000000000000000000000000000000b4f279cd0d31616f60fbca046ca14d27d8c54183783356e2fde772b3af1ff640dec888d3366c7531cd23f04f924269472d18864602b96b8df7256bd5e13860bc455731c01ace46fb597458f801b25e372e783f539d5b1c4a2e0c1a821fe5338e3ca7022028de5b7152ca06f8f2c9665f08f9352e39863a626c356f01ffa8a56d0a468414d1d507347b97e913fa7ea12775f986069d3fb6a67536e17558485623d17e94f9962a75ef8b0653f508f4374e3ac17c983014d1155c1940a1f0b6facfd7c7665562546908af6e378cae444aa7cc88768e03607261892617aae0bbffe3409335c9382edbf4177711abc4d7918354a70a8c7fba91e8048b303ec2d2d3f23350f3eed8dc9af2607a0642499f364aedebfc250fb5658250702f131c5e866a02e21d45bc6f83c95b520aa8cee188cc3680179d112cc78227897faa8264f8b758bc8264207401f3a9fbc051fcf4f9083b3591c95de65c00175b88eefcc96723f"
        );
        InboundMessageV2 memory message = abi.decode(data, (InboundMessageV2));
        assertEq(message.nonce, 0);
        assertEq(message.commands.length, 1);
        hoax(relayer, 1 ether);
        vm.expectEmit(true, false, false, false);
        emit IGatewayBase.ForeignTokenRegistered(
            bytes32(0x9441dceeeffa7e032eedaccf9b7632e60e86711551a82ffbbb0dda8afd9e4ef7),
            address(0x7ff9C67c93D9f7318219faacB5c619a773AFeF6A)
        );
        IGatewayV2(address(gateway)).v2_submit(
            message, proof, makeMockProof(), relayerRewardAddress
        );
    }

    function testAgentCallContractSuccess() public {
        vm.expectEmit(true, false, false, true);
        emit IGatewayV2.InboundMessageDispatched(1, true, relayerRewardAddress);

        vm.deal(assetHubAgent, 1 ether);
        hoax(relayer, 1 ether);
        IGatewayV2(address(gateway)).v2_submit(
            InboundMessageV2({
                origin: Constants.ASSET_HUB_AGENT_ID,
                nonce: 1,
                commands: makeCallContractCommand(0.1 ether)
            }),
            proof,
            makeMockProof(),
            relayerRewardAddress
        );
    }
}
