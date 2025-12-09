// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;
import {Script, console} from "forge-std/Script.sol";
import {Greeter} from "./Greeter.sol";
import {USDC, BASE_USDC, BASE_CHAIN_ID} from "./Constants.sol";

contract SetRemote is Script {
    uint256 internal deployerPrivateKey = vm.envUint("DEPLOYER_KEY");
    address deployerAddr = vm.addr(deployerPrivateKey);

    function setUp() public {}

    function run() public {
        vm.startBroadcast(deployerPrivateKey);

        address l1Greeter = vm.envAddress("L1_GREETER_ADDRESS");

        address l2Greeter = vm.envAddress("L2_GREETER_ADDRESS");

        Greeter(l1Greeter).setRemoteEndpoint(l2Greeter);

        vm.stopBroadcast();
        return;
    }
}
