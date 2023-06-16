// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.20;

import "openzeppelin/access/AccessControl.sol";
import "openzeppelin/token/ERC20/IERC20.sol";
import "openzeppelin/token/ERC20/utils/SafeERC20.sol";

import {Auth} from "./Auth.sol";

/// @title ERC20 Vault
/// @dev Holds ERC20 Tokens on behalf of ERC20App.
contract TokenVault is Auth {
    using SafeERC20 for IERC20;

    /// @dev Emitted when funds are deposited.
    event Deposit(address sender, address token, uint256 amount);

    /// @dev Emitted when funds are withdrawn.
    event Withdraw(address recipient, address token, uint256 amount);

    /// @dev Not enough funds to transfer.
    error InsufficientBalance();

    bytes32 public constant WITHDRAW_ROLE = keccak256("WITHDRAW_ROLE");
    bytes32 public constant DEPOSIT_ROLE = keccak256("DEPOSIT_ROLE");

    /// @dev stores the total balance of each token locked in the vault.
    mapping(address token => uint128) public balance;

    constructor() {
        _setRoleAdmin(WITHDRAW_ROLE, ADMIN_ROLE);
        _setRoleAdmin(DEPOSIT_ROLE, ADMIN_ROLE);
    }

    function deposit(address sender, address token, uint128 amount) external onlyRole(DEPOSIT_ROLE) {
        balance[token] += amount;
        IERC20(token).safeTransferFrom(sender, address(this), amount);
        emit Deposit(sender, token, amount);
    }

    function withdraw(address recipient, address token, uint128 amount) external onlyRole(WITHDRAW_ROLE) {
        if (amount > balance[token]) {
            revert InsufficientBalance();
        }

        balance[token] -= amount;
        IERC20(token).safeTransfer(recipient, amount);

        emit Withdraw(recipient, token, amount);
    }
}
