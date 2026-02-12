// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.33;

import {AgentExecutor} from "../../src/AgentExecutor.sol";
import {Gateway202509} from "../../src/upgrade/Gateway202509.sol";
import {GatewaySepolia202602} from "../../src/upgrade/Gateway202602.sepolia.sol";
import {Gateway} from "../../src/Gateway.sol";
import {ParaID} from "../../src/Types.sol";
import {Script} from "forge-std/Script.sol";
import {stdJson} from "forge-std/StdJson.sol";
import {console} from "forge-std/console.sol";

contract DeployGateway is Script {
    using stdJson for string;

    function run() public {
        vm.startBroadcast();

        AgentExecutor executor = new AgentExecutor();
        Gateway gatewayLogic;
        if (
            keccak256(abi.encodePacked(vm.envString("NODE_ENV")))
                == keccak256(abi.encodePacked("polkadot_mainnet"))
        ) {
            address beefyClient = 0x1817874feAb3ce053d0F40AbC23870DB35C2AFfc;
            gatewayLogic = new Gateway202509(address(beefyClient), address(executor));
        } else if (
            keccak256(abi.encodePacked(vm.envString("NODE_ENV")))
                == keccak256(abi.encodePacked("westend_sepolia"))
        ) {
            address beefyClient = 0xA04460B1D8bBef33F54edB2C3115e3E4D41237A6;
            gatewayLogic = new GatewaySepolia202602(address(beefyClient), address(executor));
        }

        console.log("Gateway contract address: %s", address(gatewayLogic));
        console.log("Gateway contract codehash:");
        console.logBytes32(address(gatewayLogic).codehash);
    }
}
