// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import {Script, console} from "forge-std/Script.sol";
import {SPOKE_POOL, BASE_MULTI_CALL_HANDLER, WETH9, BASE_WETH9} from "../constants/Sepolia.sol";
import {SnowbridgeL1Adaptor} from "../../../../src/l2-integration/SnowbridgeL1Adaptor.sol";

contract DeploySnowbridgeL1Adaptor is Script {
    SnowbridgeL1Adaptor public snowbridgeL1Adaptor;

    function setUp() public {}

    function run() public {
        vm.startBroadcast();

        snowbridgeL1Adaptor =
            new SnowbridgeL1Adaptor(SPOKE_POOL, BASE_MULTI_CALL_HANDLER, WETH9, BASE_WETH9);
        console.log("Snowbridge L1 Adaptor deployed at:", address(snowbridgeL1Adaptor));
        return;
    }
}
