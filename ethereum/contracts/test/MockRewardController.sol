// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "../RewardController.sol";

contract MockRewardController is RewardController {
    function handleReward(address payable, uint128 _amount) pure external override {
        // Simulate the case where there are no funds to reward the relayer
        require(_amount != 1024, "No funds available");
    }
}
