// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import {Script} from "forge-std/Script.sol";
import {StargateAdaptor} from "./StargateAdaptor.sol";

contract DeployAdaptor is Script {
    StargateAdaptor public adaptor;

    uint256 internal deployerPrivateKey = vm.envUint("DEPLOYER_KEY");
    address deployerAddr = vm.addr(deployerPrivateKey);

    function setUp() public {}

    function run() public {
        vm.startBroadcast(deployerPrivateKey);
        // ! fill with the deployed composer and receiver addresses
        adaptor = new StargateAdaptor(
            vm.envAddress("COMPOSER_ADDRESS"), vm.envAddress("RECEIVER_ADDRESS")
        );

        vm.stopBroadcast();
    }
}
