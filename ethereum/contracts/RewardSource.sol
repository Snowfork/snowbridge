// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;
pragma experimental ABIEncoderV2;

// Something that can reward a relayer
interface RewardSource {
    function reward(address payable feePayer, uint128 _amount) external;
}
