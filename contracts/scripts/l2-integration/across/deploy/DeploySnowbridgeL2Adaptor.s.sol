// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import {Script, console} from "forge-std/Script.sol";

import {
    SPOKE_POOL as SEPOLIA_SPOKE_POOL,
    MULTI_CALL_HANDLER as SEPOLIA_MULTI_CALL_HANDLER,
    BASE_SPOKE_POOL as SEPOLIA_BASE_SPOKE_POOL,
    BASE_MULTI_CALL_HANDLER as SEPOLIA_BASE_MULTI_CALL_HANDLER,
    WETH9 as SEPOLIA_WETH9,
    BASE_WETH9 as SEPOLIA_BASE_WETH9,
    GATEWAY as SEPOLIA_GATEWAY,
    UNISWAP_ROUTER as SEPOLIA_UNISWAP_ROUTER
} from "../constants/Sepolia.sol";
import {
    SPOKE_POOL as MAINNET_SPOKE_POOL,
    MULTI_CALL_HANDLER as MAINNET_MULTI_CALL_HANDLER,
    BASE_SPOKE_POOL as MAINNET_BASE_SPOKE_POOL,
    BASE_MULTI_CALL_HANDLER as MAINNET_BASE_MULTI_CALL_HANDLER,
    WETH9 as MAINNET_WETH9,
    BASE_WETH9 as MAINNET_BASE_WETH9,
    GATEWAY as MAINNET_GATEWAY,
    UNISWAP_ROUTER as MAINNET_UNISWAP_ROUTER
} from "../constants/Mainnet.sol";
import {SnowbridgeL2Adaptor} from "../../../../src/l2-integration/SnowbridgeL2Adaptor.sol";

contract DeploySnowbridgeL2Adaptor is Script {
    SnowbridgeL2Adaptor public snowbridgeL2Adaptor;

    function setUp() public {}

    function run() public {
        vm.startBroadcast();

        address BASE_SPOKE_POOL_ADDRESS;
        address MULTI_CALL_HANDLER_ADDRESS;
        address GATEWAY_V2_ADDRESS;
        address WETH9_ADDRESS;
        address BASE_WETH9_ADDRESS;
        address UNISWAP_ROUTER_ADDRESS;

        if (keccak256(bytes(vm.envString("L1_NETWORK"))) == keccak256(bytes("mainnet"))) {
            BASE_SPOKE_POOL_ADDRESS = MAINNET_BASE_SPOKE_POOL;
            MULTI_CALL_HANDLER_ADDRESS = MAINNET_MULTI_CALL_HANDLER;
            GATEWAY_V2_ADDRESS = MAINNET_GATEWAY;
            WETH9_ADDRESS = MAINNET_WETH9;
            BASE_WETH9_ADDRESS = MAINNET_BASE_WETH9;
            UNISWAP_ROUTER_ADDRESS = MAINNET_UNISWAP_ROUTER;
        } else if (keccak256(bytes(vm.envString("L1_NETWORK"))) == keccak256(bytes("sepolia"))) {
            BASE_SPOKE_POOL_ADDRESS = SEPOLIA_BASE_SPOKE_POOL;
            MULTI_CALL_HANDLER_ADDRESS = SEPOLIA_MULTI_CALL_HANDLER;
            GATEWAY_V2_ADDRESS = SEPOLIA_GATEWAY;
            WETH9_ADDRESS = SEPOLIA_WETH9;
            BASE_WETH9_ADDRESS = SEPOLIA_BASE_WETH9;
            UNISWAP_ROUTER_ADDRESS = SEPOLIA_UNISWAP_ROUTER;
        } else {
            revert("Unsupported L1 network");
        }

        snowbridgeL2Adaptor = new SnowbridgeL2Adaptor(
            BASE_SPOKE_POOL_ADDRESS,
            MULTI_CALL_HANDLER_ADDRESS,
            GATEWAY_V2_ADDRESS,
            WETH9_ADDRESS,
            BASE_WETH9_ADDRESS,
            UNISWAP_ROUTER_ADDRESS
        );
        console.log("Snowbridge L2 Adaptor deployed at:", address(snowbridgeL2Adaptor));
        return;
    }
}
