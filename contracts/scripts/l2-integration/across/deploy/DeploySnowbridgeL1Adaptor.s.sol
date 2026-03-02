// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.33;

import {Script, console} from "forge-std/Script.sol";
import {SPOKE_POOL as SEPOLIA_SPOKE_POOL, WETH9 as SEPOLIA_WETH9, GATEWAY as SEPOLIA_GATEWAY} from "../constants/Sepolia.sol";
import {SPOKE_POOL as MAINNET_SPOKE_POOL, WETH9 as MAINNET_WETH9, GATEWAY as MAINNET_GATEWAY} from "../constants/Mainnet.sol";
import {SnowbridgeL1Adaptor} from "../../../../src/l2-integration/SnowbridgeL1Adaptor.sol";

contract DeploySnowbridgeL1Adaptor is Script {
    SnowbridgeL1Adaptor public snowbridgeL1Adaptor;

    function setUp() public {}

    function run() public {
        vm.startBroadcast();
        address SPOKE_POOL_ADDRESS;
        address BASE_MULTI_CALL_HANDLER_ADDRESS;
        address WETH9_ADDRESS;
        address GATEWAY_ADDRESS;

        if (keccak256(bytes(vm.envString("L1_NETWORK"))) == keccak256(bytes("mainnet"))) {
            SPOKE_POOL_ADDRESS = MAINNET_SPOKE_POOL;
            WETH9_ADDRESS = MAINNET_WETH9;
            GATEWAY_ADDRESS = MAINNET_GATEWAY;
        } else if (keccak256(bytes(vm.envString("L1_NETWORK"))) == keccak256(bytes("sepolia"))) {
            SPOKE_POOL_ADDRESS = SEPOLIA_SPOKE_POOL;
            WETH9_ADDRESS = SEPOLIA_WETH9;
            GATEWAY_ADDRESS = SEPOLIA_GATEWAY;
        } else {
            revert("Unsupported L1 network");
        }

        snowbridgeL1Adaptor = new SnowbridgeL1Adaptor(SPOKE_POOL_ADDRESS, WETH9_ADDRESS, GATEWAY_ADDRESS);
        console.log("Snowbridge L1 Adaptor deployed at:", address(snowbridgeL1Adaptor));
        return;
    }
}
