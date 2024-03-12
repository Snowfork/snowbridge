// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.23;

import {AgentExecutor} from "./AgentExecutor.sol";
import {Gateway} from "./Gateway.sol";
import {ParaID} from "./Types.sol";
import {Script} from "forge-std/Script.sol";
import {stdJson} from "forge-std/StdJson.sol";

contract DeployGatewayLogic is Script {
    using stdJson for string;

    function setUp() public {}

    function run() public {
        uint256 privateKey = vm.envUint("PRIVATE_KEY");
        address deployer = vm.rememberKey(privateKey);
        vm.startBroadcast(deployer);

        address beefyClient = vm.envAddress("BEEFY_CLIENT_CONTRACT_ADDRESS");

        ParaID bridgeHubParaID = ParaID.wrap(uint32(vm.envUint("BRIDGE_HUB_PARAID")));
        bytes32 bridgeHubAgentID = vm.envBytes32("BRIDGE_HUB_AGENT_ID");

        uint8 foreignTokenDecimals = uint8(vm.envUint("FOREIGN_TOKEN_DECIMALS"));

        AgentExecutor executor = new AgentExecutor();
        new Gateway(address(beefyClient), address(executor), bridgeHubParaID, bridgeHubAgentID, foreignTokenDecimals);

        vm.stopBroadcast();
    }
}
