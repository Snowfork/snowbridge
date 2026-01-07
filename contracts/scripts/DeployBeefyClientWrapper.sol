// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.28;

import {Script} from "forge-std/Script.sol";
import {console} from "forge-std/console.sol";
import {BeefyClientWrapper} from "../src/BeefyClientWrapper.sol";
import {BeefyClientWrapperProxy} from "../src/BeefyClientWrapperProxy.sol";

contract DeployBeefyClientWrapper is Script {
    struct Config {
        address beefyClient;
        address owner;
        uint256 maxGasPrice;
        uint256 gracePeriodBlocks;
        uint256 minBlockIncrement;
    }

    function readConfig() internal returns (Config memory config) {
        config = Config({
            beefyClient: vm.envAddress("BEEFY_CLIENT_ADDRESS"),
            owner: vm.envAddress("REFUND_PROXY_OWNER"),
            maxGasPrice: vm.envOr("MAX_GAS_PRICE", uint256(100 gwei)),
            gracePeriodBlocks: vm.envOr("GRACE_PERIOD_BLOCKS", uint256(10)),
            minBlockIncrement: vm.envOr("MIN_BLOCK_INCREMENT", uint256(100))
        });
    }

    function run() public {
        vm.startBroadcast();

        Config memory config = readConfig();

        BeefyClientWrapper implementation = new BeefyClientWrapper();

        bytes memory initParams = abi.encode(
            config.beefyClient,
            config.owner,
            config.maxGasPrice,
            config.gracePeriodBlocks,
            config.minBlockIncrement
        );

        BeefyClientWrapperProxy proxy =
            new BeefyClientWrapperProxy(address(implementation), initParams);

        console.log("Implementation:", address(implementation));
        console.log("Proxy:", address(proxy));

        vm.stopBroadcast();
    }

    function runWithRelayers(address[] calldata relayers) public {
        vm.startBroadcast();

        Config memory config = readConfig();

        BeefyClientWrapper implementation = new BeefyClientWrapper();

        bytes memory initParams = abi.encode(
            config.beefyClient,
            config.owner,
            config.maxGasPrice,
            config.gracePeriodBlocks,
            config.minBlockIncrement
        );

        BeefyClientWrapperProxy proxy =
            new BeefyClientWrapperProxy(address(implementation), initParams);

        BeefyClientWrapper refund = BeefyClientWrapper(payable(address(proxy)));

        for (uint256 i = 0; i < relayers.length; i++) {
            refund.addRelayer(relayers[i]);
        }

        console.log("Implementation:", address(implementation));
        console.log("Proxy:", address(proxy));
        console.log("Relayers added:", relayers.length);

        vm.stopBroadcast();
    }
}
