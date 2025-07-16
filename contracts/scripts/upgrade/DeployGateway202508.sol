// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.28;

import {AgentExecutor} from "../../src/AgentExecutor.sol";
import {Gateway202508} from "../../src/upgrade/Gateway202508.sol";
import {ParaID} from "../../src/Types.sol";
import {Script} from "forge-std/Script.sol";
import {stdJson} from "forge-std/StdJson.sol";
import {console} from "forge-std/console.sol";

contract DeployGatewayWithFeeInitializer is Script {
    using stdJson for string;

    struct Config {
        uint8 foreignTokenDecimals;
        uint128 maxDestinationFee;
    }

    function readConfig() internal view returns (Config memory config) {
        uint8 foreignTokenDecimals = uint8(vm.envUint("FOREIGN_TOKEN_DECIMALS"));
        uint128 maxDestinationFee = uint128(vm.envUint("RESERVE_TRANSFER_MAX_DESTINATION_FEE"));
        config = Config({
            foreignTokenDecimals: foreignTokenDecimals,
            maxDestinationFee: maxDestinationFee
        });
    }

    function run() public {
        uint256 privateKey = vm.envUint("PRIVATE_KEY");
        address deployer = vm.rememberKey(privateKey);
        vm.startBroadcast(deployer);

        Config memory config = readConfig();

        address beefyClient = vm.envAddress("BEEFY_CLIENT_CONTRACT_ADDRESS");

        AgentExecutor executor = new AgentExecutor();

        Gateway202508 gatewayLogic = new Gateway202508(address(beefyClient), address(executor));

        console.log("Gateway contract address: %s", address(gatewayLogic));
        console.log("Gateway contract codehash:");
        console.logBytes32(address(gatewayLogic).codehash);

        console.log("Gateway initialize parameters:");
        console.logBytes(abi.encode(config));

        vm.stopBroadcast();
    }
}
