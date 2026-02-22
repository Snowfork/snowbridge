// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.33;

import {Script, console} from "forge-std/Script.sol";

import {
    SPOKE_POOL as SEPOLIA_SPOKE_POOL,
    MULTI_CALL_HANDLER as SEPOLIA_MULTI_CALL_HANDLER,
    WETH9 as SEPOLIA_WETH9,
    GATEWAY as SEPOLIA_GATEWAY,
    BASE_SPOKE_POOL as SEPOLIA_BASE_SPOKE_POOL,
    BASE_WETH9 as SEPOLIA_BASE_WETH9,
    ARBITRUM_SPOKE_POOL as SEPOLIA_ARBITRUM_SPOKE_POOL,
    ARBITRUM_WETH9 as SEPOLIA_ARBITRUM_WETH9
} from "../constants/Sepolia.sol";
import {
    SPOKE_POOL as MAINNET_SPOKE_POOL,
    MULTI_CALL_HANDLER as MAINNET_MULTI_CALL_HANDLER,
    WETH9 as MAINNET_WETH9,
    GATEWAY as MAINNET_GATEWAY,
    BASE_SPOKE_POOL as MAINNET_BASE_SPOKE_POOL,
    BASE_WETH9 as MAINNET_BASE_WETH9,
    ARBITRUM_SPOKE_POOL as MAINNET_ARBITRUM_SPOKE_POOL,
    ARBITRUM_WETH9 as MAINNET_ARBITRUM_WETH9,
    OPTIMISM_SPOKE_POOL as MAINNET_OPTIMISM_SPOKE_POOL,
    OPTIMISM_WETH9 as MAINNET_OPTIMISM_WETH9
} from "../constants/Mainnet.sol";
import {SnowbridgeL2Adaptor} from "../../../../src/l2-integration/SnowbridgeL2Adaptor.sol";

contract DeploySnowbridgeL2Adaptor is Script {
    SnowbridgeL2Adaptor public snowbridgeL2Adaptor;

    function setUp() public {}

    function run() public {
        vm.startBroadcast();

        address L2_SPOKE_POOL_ADDRESS;
        address MULTI_CALL_HANDLER_ADDRESS;
        address GATEWAY_V2_ADDRESS;
        address WETH9_ADDRESS;
        address L2_WETH9_ADDRESS;
        address UNISWAP_ROUTER_ADDRESS;

        if (
            keccak256(bytes(vm.envString("L1_NETWORK"))) == keccak256(bytes("mainnet"))
                && keccak256(bytes(vm.envString("L2_NETWORK"))) == keccak256(bytes("base"))
        ) {
            L2_SPOKE_POOL_ADDRESS = MAINNET_BASE_SPOKE_POOL;
            MULTI_CALL_HANDLER_ADDRESS = MAINNET_MULTI_CALL_HANDLER;
            GATEWAY_V2_ADDRESS = MAINNET_GATEWAY;
            WETH9_ADDRESS = MAINNET_WETH9;
            L2_WETH9_ADDRESS = MAINNET_BASE_WETH9;
        } else if (
            keccak256(bytes(vm.envString("L1_NETWORK"))) == keccak256(bytes("mainnet"))
                && keccak256(bytes(vm.envString("L2_NETWORK"))) == keccak256(bytes("arbitrum"))
        ) {
            L2_SPOKE_POOL_ADDRESS = MAINNET_ARBITRUM_SPOKE_POOL;
            MULTI_CALL_HANDLER_ADDRESS = MAINNET_MULTI_CALL_HANDLER;
            GATEWAY_V2_ADDRESS = MAINNET_GATEWAY;
            WETH9_ADDRESS = MAINNET_WETH9;
            L2_WETH9_ADDRESS = MAINNET_ARBITRUM_WETH9;
        } else if (
            keccak256(bytes(vm.envString("L1_NETWORK"))) == keccak256(bytes("mainnet"))
                && keccak256(bytes(vm.envString("L2_NETWORK"))) == keccak256(bytes("optimism"))
        ) {
            L2_SPOKE_POOL_ADDRESS = MAINNET_OPTIMISM_SPOKE_POOL;
            MULTI_CALL_HANDLER_ADDRESS = MAINNET_MULTI_CALL_HANDLER;
            GATEWAY_V2_ADDRESS = MAINNET_GATEWAY;
            WETH9_ADDRESS = MAINNET_WETH9;
            L2_WETH9_ADDRESS = MAINNET_OPTIMISM_WETH9;
        } else if (
            keccak256(bytes(vm.envString("L1_NETWORK"))) == keccak256(bytes("sepolia"))
                && keccak256(bytes(vm.envString("L2_NETWORK"))) == keccak256(bytes("base-sepolia"))
        ) {
            L2_SPOKE_POOL_ADDRESS = SEPOLIA_BASE_SPOKE_POOL;
            MULTI_CALL_HANDLER_ADDRESS = SEPOLIA_MULTI_CALL_HANDLER;
            GATEWAY_V2_ADDRESS = SEPOLIA_GATEWAY;
            WETH9_ADDRESS = SEPOLIA_WETH9;
            L2_WETH9_ADDRESS = SEPOLIA_BASE_WETH9;
        } else if (
            keccak256(bytes(vm.envString("L1_NETWORK"))) == keccak256(bytes("sepolia"))
                && keccak256(bytes(vm.envString("L2_NETWORK")))
                    == keccak256(bytes("arbitrum-sepolia"))
        ) {
            L2_SPOKE_POOL_ADDRESS = SEPOLIA_ARBITRUM_SPOKE_POOL; // Arbitrum Sepolia doesn't have a separate spoke pool deployment, so we can reuse the WETH9 address for testing purposes
            MULTI_CALL_HANDLER_ADDRESS = SEPOLIA_MULTI_CALL_HANDLER; // Arbitrum Sepolia doesn't have a separate multicall handler deployment, so we can reuse the base multicall handler address for testing purposes
            GATEWAY_V2_ADDRESS = SEPOLIA_GATEWAY;
            WETH9_ADDRESS = SEPOLIA_WETH9;
            L2_WETH9_ADDRESS = SEPOLIA_ARBITRUM_WETH9;
        } else {
            revert("Unsupported L2 network");
        }

        snowbridgeL2Adaptor = new SnowbridgeL2Adaptor(
            L2_SPOKE_POOL_ADDRESS,
            MULTI_CALL_HANDLER_ADDRESS,
            GATEWAY_V2_ADDRESS,
            WETH9_ADDRESS,
            L2_WETH9_ADDRESS
        );
        console.log("Snowbridge L2 Adaptor deployed at:", address(snowbridgeL2Adaptor));
        return;
    }
}
