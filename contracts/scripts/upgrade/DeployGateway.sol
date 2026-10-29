// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.34;

import {AgentExecutor} from "../../src/AgentExecutor.sol";
import {Gateway202610} from "../../src/upgrade/Gateway202610.sol";
import {GatewaySepolia202603} from "../../src/upgrade/Gateway202603.sepolia.sol";
import {Gateway} from "../../src/Gateway.sol";
import {Script} from "forge-std/Script.sol";
import {console} from "forge-std/console.sol";
import {stdJson} from "forge-std/StdJson.sol";

contract DeployGateway is Script {
    using stdJson for string;

    function run() public {
        vm.startBroadcast();

        AgentExecutor executor = new AgentExecutor();
        Gateway gatewayLogic;
        if (
            keccak256(abi.encodePacked(vm.envString("SNOWBRIDGE_DEPLOY_STAGE")))
                == keccak256(abi.encodePacked("polkadot_mainnet"))
        ) {
            address beefyClient = 0xbCBEbc20a0FDF9EfD53B5852a71869b0cF25283f;
            gatewayLogic = new Gateway202610(beefyClient, address(executor));
        } else if (
            keccak256(abi.encodePacked(vm.envString("SNOWBRIDGE_DEPLOY_STAGE")))
                == keccak256(abi.encodePacked("westend_sepolia"))
        ) {
            address beefyClient = 0xEBD1CFcF82BaA170b86BDe532f69A6A49c6c790D;
            gatewayLogic = new GatewaySepolia202603(address(beefyClient), address(executor));
        }

        console.log("Snowbridge deployment stage: %s", vm.envString("SNOWBRIDGE_DEPLOY_STAGE"));
        console.log("Gateway contract address: %s", address(gatewayLogic));
        console.log("Gateway contract codehash:");
        console.logBytes32(address(gatewayLogic).codehash);
    }
}
