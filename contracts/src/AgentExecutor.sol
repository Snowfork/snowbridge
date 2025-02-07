// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.28;

import {AgentExecuteCommand, ParaID} from "./Types.sol";
import {SubstrateTypes} from "./SubstrateTypes.sol";

import {IERC20} from "./interfaces/IERC20.sol";
import {IGateway} from "./interfaces/IGateway.sol";
import {SafeTokenTransfer, SafeNativeTransfer} from "./utils/SafeTransfer.sol";

/// @title Code which will run within an `Agent` using `delegatecall`.
/// @dev This is a singleton contract, meaning that all agents will execute the same code.
contract AgentExecutor {
    using SafeTokenTransfer for IERC20;
    using SafeNativeTransfer for address payable;

    /// @dev Transfer ether to `recipient`.
    function transferEther(address payable recipient, uint256 amount) external {
        recipient.safeNativeTransfer(amount);
    }

    /// @dev Transfer ERC20 to `recipient`.
    function transferToken(address token, address recipient, uint128 amount) external {
        IERC20(token).safeTransfer(recipient, amount);
    }
}
