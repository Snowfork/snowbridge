// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.25;

import {AgentExecuteCommand, ParaID} from "./Types.sol";
import {SubstrateTypes} from "./SubstrateTypes.sol";

import {IERC20} from "./interfaces/IERC20.sol";
import {SafeTokenTransfer, SafeNativeTransfer} from "./utils/SafeTransfer.sol";
import {Token} from "./Token.sol";

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

    /// @dev Mint PNA token and send to `recipient`. Only callable via `execute`.
    function mintForeignToken(address token, address recipient, uint256 amount) external {
        Token(token).mint(recipient, amount);
    }

    /// @dev Burn PNA token from `sender`. Only callable via `execute`.
    function burnForeignToken(address token, address sender, uint256 amount) external {
        Token(token).burn(sender, amount);
    }

    /// @dev Transfer ERC20 to `recipient`. Only callable via `execute`.
    function transferToken(address token, address recipient, uint128 amount) external {
        _transferToken(token, recipient, amount);
    }

    /// @dev Transfer ERC20 to `recipient`. Only callable via `execute`.
    function _transferToken(address token, address recipient, uint128 amount) internal {
        IERC20(token).safeTransfer(recipient, amount);
    }
}
