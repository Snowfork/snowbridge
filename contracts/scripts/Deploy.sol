// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.28;

import {Script} from "forge-std/Script.sol";
import {GatewayProxy} from "../src/GatewayProxy.sol";
import {Shell} from "../src/Shell.sol";

contract Stage1 is Script {
    function setUp() public {}

    function run() public {
        vm.startBroadcast();

        address operator = vm.envAddress("OPERATOR_ADDRESS");
        Shell shell = new Shell(operator);
        new GatewayProxy(address(shell), bytes(""));

        vm.stopBroadcast();
    }
}
