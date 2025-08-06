//SPDX-License-Identifier: GPL-3.0-or-later

// Copyright (C) Moondance Labs Ltd.
// This file is part of Tanssi.
// Tanssi is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
// Tanssi is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
// You should have received a copy of the GNU General Public License
// along with Tanssi.  If not, see <http://www.gnu.org/licenses/>
pragma solidity ^0.8.0;

interface IMiddlewareBasic {
    /**
     * @notice Distribute rewards for a specific era contained in an epoch by providing a Merkle root, total points, total amount of tokens and the token address of the rewards.
     * @param epoch network epoch of the middleware
     * @param eraIndex era index of Starlight's rewards distribution
     * @param totalPointsToken total amount of points for the reward distribution
     * @param amount amount of tokens to distribute
     * @param root Merkle root of the reward distribution
     * @param tokenAddress The token address of the rewards
     * @dev This function is called by the gateway only
     * @dev Emit DistributeRewards event.
     */
    function distributeRewards(
        uint256 epoch,
        uint256 eraIndex,
        uint256 totalPointsToken,
        uint256 amount,
        bytes32 root,
        address tokenAddress
    ) external;

    /**
     * @notice Slashes an operator's stake
     * @dev Only the owner can call this function
     * @dev This function first updates the stake cache for the target epoch
     * @param epoch The epoch number
     * @param operatorKey The operator key to slash
     * @param percentage Percentage to slash, represented as parts per billion.
     */
    function slash(uint48 epoch, bytes32 operatorKey, uint256 percentage) external;
    /**
     * @notice Determines which epoch a timestamp belongs to
     * @param timestamp The timestamp to check
     * @return epoch The corresponding epoch number
     */
    function getEpochAtTs(uint48 timestamp) external view returns (uint48 epoch);
}
