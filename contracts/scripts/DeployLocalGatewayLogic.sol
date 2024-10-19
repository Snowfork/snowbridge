// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.25;

import {AgentExecutor} from "../src/AgentExecutor.sol";
import {Gateway} from "../src//Gateway.sol";
import {ParaID} from "../src//Types.sol";
import {Script} from "forge-std/Script.sol";
import {stdJson} from "forge-std/StdJson.sol";

contract DeployLocalGatewayLogic is Script {
    using stdJson for string;

    function setUp() public {}

    function run() public {
        uint256 privateKey = vm.envUint("PRIVATE_KEY");
        address deployer = vm.rememberKey(privateKey);
        vm.startBroadcast(deployer);

        address beefyClient = vm.envAddress("BEEFY_CLIENT_CONTRACT_ADDRESS");

        AgentExecutor executor = new AgentExecutor();

        new Gateway(address(beefyClient), address(executor));

        vm.stopBroadcast();
    }
}
