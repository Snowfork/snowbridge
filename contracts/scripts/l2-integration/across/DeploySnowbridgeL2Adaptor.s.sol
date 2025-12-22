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
    BASE_WETH9,
    TIME_BUFFER
} from "./Constants.sol";
import {SnowbridgeL2Adaptor} from "./SnowbridgeL2Adaptor.sol";

contract DeploySnowbridgeL2Adaptor is Script {
    SnowbridgeL2Adaptor public snowbridgeL2Adaptor;

    uint256 internal deployerPrivateKey = vm.envUint("DEPLOYER_KEY");
    address deployerAddr = vm.addr(deployerPrivateKey);

    function setUp() public {}

    function run() public {
        vm.startBroadcast(deployerPrivateKey);

        snowbridgeL2Adaptor = new SnowbridgeL2Adaptor(
            BASE_SPOKE_POOL, MULTI_CALL_HANDLER, GatewayV2, WETH9, BASE_WETH9, TIME_BUFFER
        );
        console.log("Snowbridge L2 Adaptor deployed at:", address(snowbridgeL2Adaptor));
        vm.stopBroadcast();
        return;
    }
}
