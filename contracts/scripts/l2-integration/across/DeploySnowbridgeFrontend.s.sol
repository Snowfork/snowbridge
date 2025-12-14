// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import {Script, console} from "forge-std/Script.sol";
import {
    SPOKE_POOL,
    MULTI_CALL_HANDLER,
    BASE_SPOKE_POOL,
    BASE_MULTI_CALL_HANDLER,
    GatewayV2,
    WETH9,
    BASE_WETH9
} from "./Constants.sol";
import {SnowbridgeFrontend} from "./SnowbridgeFrontend.sol";

contract DeploySnowbridgeFrontend is Script {
    SnowbridgeFrontend public snowbridgeFrontend;

    uint256 internal deployerPrivateKey = vm.envUint("DEPLOYER_KEY");
    address deployerAddr = vm.addr(deployerPrivateKey);

    function setUp() public {}

    function run() public {
        vm.startBroadcast(deployerPrivateKey);

        snowbridgeFrontend = new SnowbridgeFrontend(
            BASE_SPOKE_POOL, MULTI_CALL_HANDLER, GatewayV2, WETH9, BASE_WETH9
        );
        vm.stopBroadcast();
        return;
    }
}
