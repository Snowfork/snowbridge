// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.25;

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

    address public account1;
    address public account2;

    // tokenID for DOT
    bytes32 public dotTokenID;

    function setUp() public {
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

        SetOperatingModeParams memory params =
            SetOperatingModeParams({mode: OperatingMode.Normal});
        MockGateway(address(gateway)).v1_handleSetOperatingMode_public(
            abi.encode(params)
        );

        assetHubAgent = IGateway(address(gateway)).agentOf(Constants.ASSET_HUB_AGENT_ID);

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

        dotTokenID = bytes32(uint256(1));
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

    /**
     * Message Verification
     */
    function testSubmitHappyPath() public {
        // Expect the gateway to emit `InboundMessageDispatched`
        vm.expectEmit(true, false, false, true);
        emit IGateway.InboundMessageDispatched(1, true, relayerRewardAddress);

        hoax(relayer, 1 ether);
        IGateway(address(gateway)).v2_submit(
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
        IGateway(address(gateway)).v2_submit(
            message, proof, makeMockProof(), relayerRewardAddress
        );

        vm.expectRevert(IGateway.InvalidNonce.selector);
        hoax(relayer, 1 ether);
        IGateway(address(gateway)).v2_submit(
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
        vm.expectRevert(IGateway.InvalidProof.selector);

        hoax(relayer, 1 ether);
        IGateway(address(gateway)).v2_submit(
            message, proof, makeMockProof(), relayerRewardAddress
        );
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
    }
}
