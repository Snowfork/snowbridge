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
} from "../constants/Sepolia.sol";
import {SnowbridgeL2Adaptor} from "../../../../src/l2-integration/SnowbridgeL2Adaptor.sol";

contract DeploySnowbridgeL2Adaptor is Script {
    SnowbridgeL2Adaptor public snowbridgeL2Adaptor;

    function setUp() public {}

    function run() public {
        vm.startBroadcast();

        snowbridgeL2Adaptor = new SnowbridgeL2Adaptor(
            BASE_SPOKE_POOL, MULTI_CALL_HANDLER, GatewayV2, WETH9, BASE_WETH9
        );
        console.log("Snowbridge L2 Adaptor deployed at:", address(snowbridgeL2Adaptor));
        return;
    }
}
