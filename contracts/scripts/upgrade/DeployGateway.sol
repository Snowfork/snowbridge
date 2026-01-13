// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.28;

import {AgentExecutor} from "../../src/AgentExecutor.sol";
import {Gateway202509} from "../../src/upgrade/Gateway202509.sol";
import {GatewaySepolia202601} from "../../src/upgrade/Gateway202601.sepolia.sol";
import {Gateway} from "../../src/Gateway.sol";
import {ParaID} from "../../src/Types.sol";
import {Script} from "forge-std/Script.sol";
import {stdJson} from "forge-std/StdJson.sol";
import {console} from "forge-std/console.sol";

contract DeployGateway is Script {
    using stdJson for string;

    address beefyClient = 0x1817874feAb3ce053d0F40AbC23870DB35C2AFfc;

    function run() public {
        vm.startBroadcast();

        AgentExecutor executor = new AgentExecutor();
        Gateway gatewayLogic;
        if (
            keccak256(abi.encodePacked(vm.envString("NODE_ENV")))
                == keccak256(abi.encodePacked("polkadot_mainnet"))
        ) {
            gatewayLogic = new Gateway202509(address(beefyClient), address(executor));
        } else if (
            keccak256(abi.encodePacked(vm.envString("NODE_ENV")))
                == keccak256(abi.encodePacked("westend_sepolia"))
        ) {
            gatewayLogic = new GatewaySepolia202601(address(beefyClient), address(executor));
        }

        console.log("Gateway contract address: %s", address(gatewayLogic));
        console.log("Gateway contract codehash:");
        console.logBytes32(address(gatewayLogic).codehash);
    }
}
