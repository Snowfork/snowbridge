// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.28;

import {AgentExecutor} from "../../../src/AgentExecutor.sol";
import {Gateway20250311} from "../../../src/upgrades/westend/Gateway20250311.sol";
import {ParaID} from "../../../src/Types.sol";
import {Script} from "forge-std/Script.sol";
import {stdJson} from "forge-std/StdJson.sol";
import {console} from "forge-std/console.sol";

contract DeployGateway20250311 is Script {
    using stdJson for string;

    function setUp() public {}

    function run() public {
        uint256 privateKey = vm.envUint("PRIVATE_KEY");
        address deployer = vm.rememberKey(privateKey);
        vm.startBroadcast(deployer);

        address beefyClient = vm.envAddress("BEEFY_CLIENT_CONTRACT_ADDRESS");

        AgentExecutor executor = new AgentExecutor();

        Gateway20250311 gatewayLogic = new Gateway20250311(address(beefyClient), address(executor));

        console.log("Gateway contract address: %s", address(gatewayLogic));
        console.log("Gateway contract codehash:");
        console.logBytes32(address(gatewayLogic).codehash);

        vm.stopBroadcast();
    }
}
