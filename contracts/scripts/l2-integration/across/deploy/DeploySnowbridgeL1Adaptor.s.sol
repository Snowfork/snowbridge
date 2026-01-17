// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import {Script, console} from "forge-std/Script.sol";
import {
    SPOKE_POOL as SEPOLIA_SPOKE_POOL,
    BASE_MULTI_CALL_HANDLER as SEPOLIA_BASE_MULTI_CALL_HANDLER,
    WETH9 as SEPOLIA_WETH9,
    BASE_WETH9 as SEPOLIA_BASE_WETH9
} from "../constants/Sepolia.sol";
import {
    SPOKE_POOL as MAINNET_SPOKE_POOL,
    BASE_MULTI_CALL_HANDLER as MAINNET_BASE_MULTI_CALL_HANDLER,
    WETH9 as MAINNET_WETH9,
    BASE_WETH9 as MAINNET_BASE_WETH9
} from "../constants/Mainnet.sol";
import {SnowbridgeL1Adaptor} from "../../../../src/l2-integration/SnowbridgeL1Adaptor.sol";

contract DeploySnowbridgeL1Adaptor is Script {
    SnowbridgeL1Adaptor public snowbridgeL1Adaptor;

    function setUp() public {}

    function run() public {
        vm.startBroadcast();
        address SPOKE_POOL_ADDRESS;
        address BASE_MULTI_CALL_HANDLER_ADDRESS;
        address WETH9_ADDRESS;
        address BASE_WETH9_ADDRESS;

        if (keccak256(bytes(vm.envString("L1_NETWORK"))) == keccak256(bytes("mainnet"))) {
            SPOKE_POOL_ADDRESS = MAINNET_SPOKE_POOL;
            BASE_MULTI_CALL_HANDLER_ADDRESS = MAINNET_BASE_MULTI_CALL_HANDLER;
            WETH9_ADDRESS = MAINNET_WETH9;
            BASE_WETH9_ADDRESS = MAINNET_BASE_WETH9;
        } else if (keccak256(bytes(vm.envString("L1_NETWORK"))) == keccak256(bytes("sepolia"))) {
            SPOKE_POOL_ADDRESS = SEPOLIA_SPOKE_POOL;
            BASE_MULTI_CALL_HANDLER_ADDRESS = SEPOLIA_BASE_MULTI_CALL_HANDLER;
            WETH9_ADDRESS = SEPOLIA_WETH9;
            BASE_WETH9_ADDRESS = SEPOLIA_BASE_WETH9;
        } else {
            revert("Unsupported L1 network");
        }

        snowbridgeL1Adaptor = new SnowbridgeL1Adaptor(
            SPOKE_POOL_ADDRESS, BASE_MULTI_CALL_HANDLER_ADDRESS, WETH9_ADDRESS, BASE_WETH9_ADDRESS
        );
        console.log("Snowbridge L1 Adaptor deployed at:", address(snowbridgeL1Adaptor));
        return;
    }
}
