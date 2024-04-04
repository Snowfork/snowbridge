// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.23;

import {AgentExecuteCommand, ParaID} from "./Types.sol";
import {SubstrateTypes} from "./SubstrateTypes.sol";

import {IERC20} from "./interfaces/IERC20.sol";
import {SafeTokenTransfer, SafeNativeTransfer} from "./utils/SafeTransfer.sol";
import {ERC20} from "./ERC20.sol";
import {Gateway} from "./Gateway.sol";

/// @title Code which will run within an `Agent` using `delegatecall`.
/// @dev This is a singleton contract, meaning that all agents will execute the same code.
contract AgentExecutor {
    using SafeTokenTransfer for IERC20;
    using SafeNativeTransfer for address payable;

    // Emitted when token minted
    event TokenMinted(bytes32 indexed tokenID, address token, address recipient, uint256 amount);
    // Emitted when token burnt
    event TokenBurnt(bytes32 indexed tokenID, address token, address sender, uint256 amount);

    /// @dev Transfer ether to `recipient`. Unlike `_transferToken` This logic is not nested within `execute`,
    /// as the gateway needs to control an agent's ether balance directly.
    ///
    function transferNative(address payable recipient, uint256 amount) external {
        recipient.safeNativeTransfer(amount);
    }

    /// @dev Transfer ERC20 to `recipient`. Only callable via `execute`.
    function transferToken(address token, address recipient, uint128 amount) external {
        IERC20(token).safeTransfer(recipient, amount);
    }

    /// @dev Mint ERC20 token to `recipient`.
    function mintToken(bytes32 tokenID, address token, address recipient, uint256 amount) external {
        ERC20(token).mint(recipient, amount);
        emit TokenMinted(tokenID, token, recipient, amount);
    }

    function burnToken(bytes32 tokenID, address token, address sender, uint256 amount) external {
        ERC20(token).burn(sender, amount);
        emit TokenBurnt(tokenID, token, sender, amount);
    }
}
