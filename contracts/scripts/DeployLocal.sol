// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.25;

import {Script} from "forge-std/Script.sol";
import {stdJson} from "forge-std/StdJson.sol";
import {BeefyClient} from "../src/BeefyClient.sol";
import {console2} from "forge-std/console2.sol";
import {GatewayProxy} from "../src/GatewayProxy.sol";
import {Gateway} from "../src/Gateway.sol";
import {AgentExecutor} from "../src/AgentExecutor.sol";
import {OperatingMode} from "../src/Types.sol";
import {HelperConfig} from "./HelperConfig.sol";
import {WETH9} from "canonical-weth/WETH9.sol";
import {IGateway} from "../src/interfaces/IGateway.sol";

contract DeployLocal is Script {
    using stdJson for string;

    function setUp() public {}

    function run() public {
        HelperConfig helperConfig = new HelperConfig("");
        _deploy(helperConfig);
    }

    function run(string calldata testnet) public {
        HelperConfig helperConfig = new HelperConfig(testnet);
        _deploy(helperConfig);
    }

    function _deploy(HelperConfig helperConfig) public {
        HelperConfig.GatewayConfig memory gatewayConfig = helperConfig.getGatewayConfig();
        HelperConfig.GatewayInitConfig memory gatewayInitConfig = helperConfig.getGatewayInitConfig();
        HelperConfig.BeefyClientConfig memory beefyClientConfig = helperConfig.getBeefyClientConfig();
        vm.startBroadcast();

        BeefyClient beefyClient;
        // BeefyClient
        // Seems `fs_permissions` explicitly configured as absolute path does not work and only allowed from project root
        {
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

            beefyClient = new BeefyClient(
                beefyClientConfig.randaoCommitDelay,
                beefyClientConfig.randaoCommitExpiration,
                beefyClientConfig.minimumSignatures,
                startBlock,
                current,
                next
            );
        }

        AgentExecutor executor = new AgentExecutor();
        Gateway gatewayLogic = new Gateway(
            address(beefyClient),
            address(executor),
            gatewayConfig.bridgeHubParaID,
            gatewayConfig.bridgeHubAgentID,
            gatewayConfig.foreignTokenDecimals,
            gatewayConfig.maxDestinationFee
        );

        OperatingMode defaultOperatingMode;
        if (gatewayInitConfig.rejectOutboundMessages) {
            defaultOperatingMode = OperatingMode.RejectingOutboundMessages;
        } else {
            defaultOperatingMode = OperatingMode.Normal;
        }

        Gateway.Config memory config = Gateway.Config({
            mode: defaultOperatingMode,
            deliveryCost: gatewayInitConfig.deliveryCost,
            registerTokenFee: gatewayInitConfig.registerTokenFee,
            assetHubParaID: gatewayInitConfig.assetHubParaID,
            assetHubAgentID: gatewayInitConfig.assetHubAgentID,
            assetHubCreateAssetFee: gatewayInitConfig.assetHubCreateAssetFee,
            assetHubReserveTransferFee: gatewayInitConfig.assetHubReserveTransferFee,
            exchangeRate: gatewayInitConfig.exchangeRate,
            multiplier: gatewayInitConfig.multiplier,
            rescueOperator: address(0)
        });

        GatewayProxy gateway = new GatewayProxy(address(gatewayLogic), abi.encode(config));
        console2.log("BeefyClient: ", address(beefyClient));
        console2.log("Gateway impl: ", address(gatewayLogic));
        console2.log("gateway address: ", address(gateway));

        // Deploy WETH for testing
        new WETH9();

        // Fund the gateway proxy contract. Used to reward relayers.
        uint256 initialDeposit = vm.envUint("GATEWAY_PROXY_INITIAL_DEPOSIT");

        IGateway(address(gateway)).depositEther{value: initialDeposit}();

        vm.stopBroadcast();
    }
}
