// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.25;

import {console2} from "forge-std/console2.sol";
import {Script} from "forge-std/Script.sol";
import {stdJson} from "forge-std/StdJson.sol";

import {GatewayTanssi202506} from "../src/upgrades/GatewayTanssi202506.sol";
import {HelperConfig} from "./HelperConfig.sol";
import {UpgradeParams} from "../src/Params.sol";

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

        GatewayTanssi202506 gatewayLogic = new GatewayTanssi202506(
            address(gatewayConfig.beefyClient),
            address(gatewayConfig.agentExecutor),
            gatewayConfig.bridgeHubParaID,
            gatewayConfig.bridgeHubAgentID,
            gatewayConfig.foreignTokenDecimals,
            gatewayConfig.maxDestinationFee
        );

        console2.log("Gateway logic impl: ", address(gatewayLogic));
        console2.log("Gateway logic codehash: ");
        console2.logBytes32(address(gatewayLogic).codehash);
        UpgradeParams memory params = UpgradeParams({
            impl: address(gatewayLogic),
            implCodeHash: address(gatewayLogic).codehash,
            initParams: bytes("")
        });

        console2.logBytes(abi.encode(params));
        vm.stopBroadcast();
    }
}
