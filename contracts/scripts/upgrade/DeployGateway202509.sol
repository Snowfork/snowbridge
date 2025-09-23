// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.28;

import {AgentExecutor} from "../../src/AgentExecutor.sol";
import {Gateway202509} from "../../src/upgrade/Gateway202509.sol";
import {ParaID} from "../../src/Types.sol";
import {Script} from "forge-std/Script.sol";
import {stdJson} from "forge-std/StdJson.sol";
import {console} from "forge-std/console.sol";

contract DeployGateway202509 is Script {
    using stdJson for string;

    address beefyClient = 0x1817874feAb3ce053d0F40AbC23870DB35C2AFfc;

    function run() public {
        vm.startBroadcast();

        AgentExecutor executor = new AgentExecutor();
        Gateway202509 gatewayLogic = new Gateway202509(address(beefyClient), address(executor));

        console.log("Gateway contract address: %s", address(gatewayLogic));
        console.log("Gateway contract codehash:");
        console.logBytes32(address(gatewayLogic).codehash);
    }
}
