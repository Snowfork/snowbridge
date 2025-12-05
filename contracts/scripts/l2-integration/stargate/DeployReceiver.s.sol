// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import {Script, console} from "forge-std/Script.sol";

import {StargateReceiver} from "./StargateReceiver.sol";
import {OPT_LZ_ENDPOINT, OPT_STARGATE} from "./Constants.sol";

// deploy on op-sepolia
contract DeployReceiver is Script {
    StargateReceiver public receiver;

    uint256 internal deployerPrivateKey = vm.envUint("DEPLOYER_KEY");
    address deployerAddr = vm.addr(deployerPrivateKey);

    function setUp() public {}

    function run() public {
        vm.startBroadcast(deployerPrivateKey);

        console.log("deployer balance", deployerAddr.balance, deployerAddr);

        // deploy receiver on arbitrum
        receiver = new StargateReceiver(OPT_LZ_ENDPOINT, OPT_STARGATE);

        vm.stopBroadcast();
    }
}
