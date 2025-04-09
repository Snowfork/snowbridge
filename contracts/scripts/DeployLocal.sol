// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.28;

import {WETH9} from "canonical-weth/WETH9.sol";
import {Script} from "forge-std/Script.sol";
import {BeefyClient} from "../src/BeefyClient.sol";
import {IGatewayV1} from "../src/v1/IGateway.sol";
import {GatewayProxy} from "../src/GatewayProxy.sol";
import {Gateway} from "../src/Gateway.sol";
import {MockGatewayV2} from "../test/mocks/MockGatewayV2.sol";
import {AgentExecutor} from "../src/AgentExecutor.sol";
import {OperatingMode} from "../src/Types.sol";
import {Initializer} from "../src/Initializer.sol";
import {SafeNativeTransfer} from "../src/utils/SafeTransfer.sol";
import {stdJson} from "forge-std/StdJson.sol";
import {UD60x18, ud60x18} from "prb/math/src/UD60x18.sol";
import {HelloWorld} from "../test/mocks/HelloWorld.sol";

contract DeployLocal is Script {
    using SafeNativeTransfer for address payable;
    using stdJson for string;

    function setUp() public {}

    function run() public {
        uint256 privateKey = vm.envUint("PRIVATE_KEY");
        address deployer = vm.rememberKey(privateKey);
        vm.startBroadcast(deployer);

        // BeefyClient
        // Seems `fs_permissions` explicitly configured as absolute path does not work and only allowed from project root
        string memory root = vm.projectRoot();
        string memory beefyCheckpointFile = string.concat(root, "/beefy-state.json");
        string memory beefyCheckpointRaw = vm.readFile(beefyCheckpointFile);
        uint64 startBlock = uint64(beefyCheckpointRaw.readUint(".startBlock"));

        BeefyClient.ValidatorSet memory current = BeefyClient.ValidatorSet(
            uint128(beefyCheckpointRaw.readUint(".current.id")),
            uint128(beefyCheckpointRaw.readUint(".current.length")),
            beefyCheckpointRaw.readBytes32(".current.root")
        );
        BeefyClient.ValidatorSet memory next = BeefyClient.ValidatorSet(
            uint128(beefyCheckpointRaw.readUint(".next.id")),
            uint128(beefyCheckpointRaw.readUint(".next.length")),
            beefyCheckpointRaw.readBytes32(".next.root")
        );

        uint256 randaoCommitDelay = vm.envUint("RANDAO_COMMIT_DELAY");
        uint256 randaoCommitExpiration = vm.envUint("RANDAO_COMMIT_EXP");
        uint256 minimumSignatures = vm.envUint("MINIMUM_REQUIRED_SIGNATURES");
        BeefyClient beefyClient = new BeefyClient(
            randaoCommitDelay, randaoCommitExpiration, minimumSignatures, startBlock, current, next
        );

        uint8 foreignTokenDecimals = uint8(vm.envUint("FOREIGN_TOKEN_DECIMALS"));
        uint128 maxDestinationFee = uint128(vm.envUint("RESERVE_TRANSFER_MAX_DESTINATION_FEE"));

        AgentExecutor executor = new AgentExecutor();
        Gateway gatewayLogic = new Gateway(address(beefyClient), address(executor));

        bool rejectOutboundMessages = vm.envBool("REJECT_OUTBOUND_MESSAGES");
        OperatingMode defaultOperatingMode;
        if (rejectOutboundMessages) {
            defaultOperatingMode = OperatingMode.RejectingOutboundMessages;
        } else {
            defaultOperatingMode = OperatingMode.Normal;
        }

        Initializer.Config memory config = Initializer.Config({
            mode: defaultOperatingMode,
            deliveryCost: uint128(vm.envUint("DELIVERY_COST")),
            registerTokenFee: uint128(vm.envUint("REGISTER_TOKEN_FEE")),
            assetHubCreateAssetFee: uint128(vm.envUint("CREATE_ASSET_FEE")),
            assetHubReserveTransferFee: uint128(vm.envUint("RESERVE_TRANSFER_FEE")),
            exchangeRate: ud60x18(vm.envUint("EXCHANGE_RATE")),
            multiplier: ud60x18(vm.envUint("FEE_MULTIPLIER")),
            foreignTokenDecimals: foreignTokenDecimals,
            maxDestinationFee: maxDestinationFee
        });

        GatewayProxy gateway = new GatewayProxy(address(gatewayLogic), abi.encode(config));

        // Deploy WETH for testing
        new WETH9();

        // For testing call contract
        new HelloWorld();

        // Fund the gateway proxy contract. Used to reward relayers
        uint256 initialDeposit = vm.envUint("GATEWAY_PROXY_INITIAL_DEPOSIT");
        IGatewayV1(address(gateway)).depositEther{value: initialDeposit}();

        // Deploy MockGatewayV2 for testing
        new MockGatewayV2();

        vm.stopBroadcast();
    }
}
