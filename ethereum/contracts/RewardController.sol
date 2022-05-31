// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

// Something that can reward a relayer
interface RewardController {

    event Rewarded(address relayer, uint128 reward);

    // NOTE: should never revert or the bridge will be broken
    function handleReward(address payable relayer, uint128 reward) external;
}
