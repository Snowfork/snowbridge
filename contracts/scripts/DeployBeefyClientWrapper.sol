// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.33;

import {Script} from "forge-std/Script.sol";
import {console} from "forge-std/console.sol";
import {BeefyClientWrapper} from "../src/BeefyClientWrapper.sol";

contract DeployBeefyClientWrapper is Script {
    struct Config {
        address gateway;
        address owner;
        uint256 maxGasPrice;
        uint256 maxRefundAmount;
        uint256 refundTarget;
    }

    function readConfig() internal returns (Config memory config) {
        config = Config({
            gateway: vm.envAddress("GATEWAY_PROXY_ADDRESS"),
            owner: vm.envAddress("WRAPPER_OWNER"),
            maxGasPrice: vm.envOr("MAX_GAS_PRICE", uint256(100 gwei)),
            maxRefundAmount: vm.envOr("MAX_REFUND_AMOUNT", uint256(0.05 ether)),
            refundTarget: vm.envOr("REFUND_TARGET", uint256(350)) // ~35 min for 100% refund
        });
    }

    function run() public {
        vm.startBroadcast();

        Config memory config = readConfig();

        BeefyClientWrapper wrapper = new BeefyClientWrapper(
            config.gateway,
            config.owner,
            config.maxGasPrice,
            config.maxRefundAmount,
            config.refundTarget
        );

        console.log("BeefyClientWrapper:", address(wrapper));

        vm.stopBroadcast();
    }
}
