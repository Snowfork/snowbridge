// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.28;

import {WETH9} from "canonical-weth/WETH9.sol";
import {Script} from "forge-std/Script.sol";
import {BeefyClient} from "../src/BeefyClient.sol";

import {IGateway} from "../src/interfaces/IGateway.sol";
import {GatewayProxy} from "../src/GatewayProxy.sol";
import {Gateway} from "../src/Gateway.sol";
import {Agent} from "../src/Agent.sol";
import {AgentExecutor} from "../src/AgentExecutor.sol";
import {ParaID} from "../src/Types.sol";
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

        IGateway(address(gatewayAddress)).depositEther{value: initialDeposit}();

        vm.stopBroadcast();
    }
}
