// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.28;

import {AgentExecutor} from "../../src/AgentExecutor.sol";
import {Gateway202601} from "./Gateway202601.sepolia.sol";
import {ParaID} from "../../src/Types.sol";
import {Script} from "forge-std/Script.sol";
import {stdJson} from "forge-std/StdJson.sol";
import {console} from "forge-std/console.sol";

contract DeployGateway202601 is Script {
    using stdJson for string;

    address beefyClient = 0xA04460B1D8bBef33F54edB2C3115e3E4D41237A6;

    function run() public {
        vm.startBroadcast();

        AgentExecutor executor = new AgentExecutor();
        Gateway202601 gatewayLogic = new Gateway202601(address(beefyClient), address(executor));

        console.log("Gateway contract address: %s", address(gatewayLogic));
        console.log("Gateway contract codehash:");
        console.logBytes32(address(gatewayLogic).codehash);
    }
}
