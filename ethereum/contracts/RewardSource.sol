// SPDX-License-Identifier: Apache-2.0
pragma solidity >=0.7.6;
pragma experimental ABIEncoderV2;

// Something that can reward a relayer
interface RewardSource {
    function reward(address payable feePayer, uint256 _amount) external;
}
