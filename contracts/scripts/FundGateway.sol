// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity ^0.8.33;

import {Script} from "forge-std/Script.sol";
import {IGatewayV1} from "../src/v1/IGateway.sol";
import {SafeNativeTransfer} from "../src/utils/SafeTransfer.sol";
import {stdJson} from "forge-std/StdJson.sol";

contract FundGateway is Script {
    using SafeNativeTransfer for address payable;
    using stdJson for string;

    function setUp() public {}

    function run() public {
        uint256 privateKey = vm.envUint("PRIVATE_KEY");
        address deployer = vm.rememberKey(privateKey);
        vm.startBroadcast(deployer);

        uint256 initialDeposit = vm.envUint("GATEWAY_PROXY_INITIAL_DEPOSIT");
        address gatewayAddress = vm.envAddress("GATEWAY_PROXY_CONTRACT");

        IGatewayV1(address(gatewayAddress)).depositEther{value: initialDeposit}();

        vm.stopBroadcast();
    }
}
