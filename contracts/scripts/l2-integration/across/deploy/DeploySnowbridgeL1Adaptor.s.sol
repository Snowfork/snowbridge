// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import {Script, console} from "forge-std/Script.sol";
import {
    SPOKE_POOL,
    TIME_BUFFER,
    BASE_MULTI_CALL_HANDLER,
    WETH9,
    BASE_WETH9
} from "../constants/Sepolia.sol";
import {SnowbridgeL1Adaptor} from "../SnowbridgeL1Adaptor.sol";

contract DeploySnowbridgeL1Adaptor is Script {
    SnowbridgeL1Adaptor public snowbridgeL1Adaptor;

    uint256 internal deployerPrivateKey = vm.envUint("DEPLOYER_KEY");
    address deployerAddr = vm.addr(deployerPrivateKey);

    function setUp() public {}

    function run() public {
        vm.startBroadcast(deployerPrivateKey);

        snowbridgeL1Adaptor = new SnowbridgeL1Adaptor(
            SPOKE_POOL, BASE_MULTI_CALL_HANDLER, WETH9, BASE_WETH9, TIME_BUFFER
        );
        console.log("Snowbridge L1 Adaptor deployed at:", address(snowbridgeL1Adaptor));
        vm.stopBroadcast();
        return;
    }
}
