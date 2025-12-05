// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import {Script, console} from "forge-std/Script.sol";

import {StargateComposer} from "./StargateComposer.sol";

// deploy on sepolia
contract DeployComposer is Script {
    StargateComposer public stargateComposer;

    uint256 internal deployerPrivateKey = vm.envUint("DEPLOYER_KEY");
    address deployerAddr = vm.addr(deployerPrivateKey);

    function setUp() public {}

    function run() public {
        console.log("deployer balance", deployerAddr.balance);

        vm.startBroadcast(deployerPrivateKey);
        // deploy composer

        stargateComposer = new StargateComposer();
        vm.stopBroadcast();
    }
}
