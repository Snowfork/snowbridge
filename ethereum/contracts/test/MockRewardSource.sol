// SPDX-License-Identifier: Apache-2.0
pragma solidity >=0.7.6;
pragma experimental ABIEncoderV2;

import "../RewardSource.sol";

contract MockRewardSource is RewardSource {
    function reward(address payable, uint256 _amount) pure external override {
        // Simulate the case where there are no funds to reward the relayer
        require(_amount != 1024, "No funds available");
    }
}
