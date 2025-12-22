// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import {Script, console} from "forge-std/Script.sol";
import {
    SPOKE_POOL,
    MULTI_CALL_HANDLER,
    BASE_SPOKE_POOL,
    BASE_MULTI_CALL_HANDLER
} from "./Constants.sol";
import {Greeter} from "./Greeter.sol";

contract DeployGreeter is Script {
    Greeter public greeter;

    uint256 internal deployerPrivateKey = vm.envUint("DEPLOYER_KEY");
    address deployerAddr = vm.addr(deployerPrivateKey);

    function setUp() public {}

    function run() public {
        vm.startBroadcast(deployerPrivateKey);

        bool isL1 = vm.envBool("IS_L1");
        if (isL1) {
            greeter = new Greeter(SPOKE_POOL, BASE_MULTI_CALL_HANDLER);
        } else {
            greeter = new Greeter(BASE_SPOKE_POOL, MULTI_CALL_HANDLER);
        }
        vm.stopBroadcast();
        return;
    }
}
