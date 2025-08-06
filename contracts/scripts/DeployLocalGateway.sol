// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.25;

import {console2} from "forge-std/console2.sol";
import {Script} from "forge-std/Script.sol";
import {stdJson} from "forge-std/StdJson.sol";

import {GatewayProxy} from "../src/GatewayProxy.sol";
import {Gateway} from "../src/Gateway.sol";
import {OperatingMode} from "../src/Types.sol";
import {HelperConfig} from "./HelperConfig.sol";

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

    function _deploy(HelperConfig helperConfig) private {
        vm.startBroadcast();
        HelperConfig.GatewayConfig memory gatewayConfig = helperConfig.getGatewayConfig();
        HelperConfig.GatewayInitConfig memory gatewayInitConfig = helperConfig.getGatewayInitConfig();

        Gateway gatewayLogic = new Gateway(
            address(gatewayConfig.beefyClient),
            address(gatewayConfig.agentExecutor),
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
        console2.log("Gateway impl: ", address(gatewayLogic));
        console2.log("gateway address: ", address(gateway));
        vm.stopBroadcast();
    }
}
