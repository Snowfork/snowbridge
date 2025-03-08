// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.28;

import {stdJson} from "forge-std/StdJson.sol";
import {console} from "forge-std/console.sol";

import {WETH9} from "canonical-weth/WETH9.sol";
import {Script} from "forge-std/Script.sol";
import {BeefyClient} from "../../../src/BeefyClient.sol";
import {AgentExecutor} from "../../../src/AgentExecutor.sol";
import {ParaID} from "../../../src/Types.sol";
import {Gateway202503} from "../../../src/upgrades/Gateway202503.sol";
import {SafeNativeTransfer} from "../../../src/utils/SafeTransfer.sol";
import {UD60x18, ud60x18} from "prb/math/src/UD60x18.sol";


contract DeployGateway202502 is Script {
    address public constant BEEFY_CLIENT = 0x6eD05bAa904df3DE117EcFa638d4CB84e1B8A00C;

    function run() public {
        vm.startBroadcast();

        AgentExecutor executor = new AgentExecutor();
        new Gateway202503(BEEFY_CLIENT, address(executor));

        console.log("Gateway contract address: %s", address(gatewayLogic));
        console.log("Gateway contract codehash:");
        console.logBytes32(address(gatewayLogic).codehash);

        vm.stopBroadcast();
    }
}
