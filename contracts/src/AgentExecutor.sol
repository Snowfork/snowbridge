// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.28;

import {WETH9} from "canonical-weth/WETH9.sol";
import {ParaID} from "./Types.sol";
import {SubstrateTypes} from "./SubstrateTypes.sol";

import {IERC20} from "./interfaces/IERC20.sol";
import {SafeTokenTransfer, SafeNativeTransfer} from "./utils/SafeTransfer.sol";
import {Call} from "./utils/Call.sol";
import {Gateway} from "./Gateway.sol";

/// @title Code which will run within an `Agent` using `delegatecall`.
/// @dev This is a singleton contract, meaning that all agents will execute the same code.
contract AgentExecutor {
    using SafeTokenTransfer for IERC20;
    using SafeNativeTransfer for address payable;

    /// @dev Transfer ether to `recipient`. Unlike `_transferToken` This logic is not nested within `execute`,
    /// as the gateway needs to control an agent's ether balance directly.
    ///
    function transferNative(address payable recipient, uint256 amount) external {
        recipient.safeNativeTransfer(amount);
    }

    /// @dev Transfer ERC20 to `recipient`. Only callable via `execute`.
    function transferToken(address token, address recipient, uint128 amount) external {
        _transferToken(token, recipient, amount);
    }

    /// @dev Call contract with Ether value. Only callable via `execute`.
    function callContract(address target, bytes memory data) external {
        bool success = Call.safeCall(target, data);
        if (!success) {
            revert();
        }
    }

    /// @dev Transfer WETH to `recipient`. Only callable via `execute`.
    function transferWeth(address weth, address recipient, uint128 amount) external {
        WETH9(payable(weth)).withdraw(amount);
        payable(recipient).safeNativeTransfer(amount);
    }

    function deposit() external payable {}

    function _transferToken(address token, address recipient, uint128 amount) internal {
        IERC20(token).safeTransfer(recipient, amount);
    }
}
