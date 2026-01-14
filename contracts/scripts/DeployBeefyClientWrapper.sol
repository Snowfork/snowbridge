// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.28;

import {Script} from "forge-std/Script.sol";
import {console} from "forge-std/console.sol";
import {BeefyClientWrapper} from "../src/BeefyClientWrapper.sol";

contract DeployBeefyClientWrapper is Script {
    struct Config {
        address beefyClient;
        address owner;
        uint256 maxGasPrice;
        uint256 maxRefundAmount;
        uint256 refundTarget;
        uint256 rewardTarget;
    }

    function readConfig() internal returns (Config memory config) {
        config = Config({
            beefyClient: vm.envAddress("BEEFY_CLIENT_ADDRESS"),
            owner: vm.envAddress("REFUND_PROXY_OWNER"),
            maxGasPrice: vm.envOr("MAX_GAS_PRICE", uint256(100 gwei)),
            maxRefundAmount: vm.envOr("MAX_REFUND_AMOUNT", uint256(1 ether)),
            refundTarget: vm.envOr("REFUND_TARGET", uint256(300)), // ~30 min for 100% refund
            rewardTarget: vm.envOr("REWARD_TARGET", uint256(2400)) // ~4 hours for 100% reward
        });
    }

    function run() public {
        vm.startBroadcast();

        Config memory config = readConfig();

        BeefyClientWrapper wrapper = new BeefyClientWrapper(
            config.beefyClient,
            config.owner,
            config.maxGasPrice,
            config.maxRefundAmount,
            config.refundTarget,
            config.rewardTarget
        );

        console.log("BeefyClientWrapper:", address(wrapper));

        vm.stopBroadcast();
    }
}
