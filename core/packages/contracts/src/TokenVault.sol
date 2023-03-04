// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";

/// @title ERC20 Vault
/// @dev Holds ERC20 Tokens on behalf of ERC20App.
contract TokenVault is Ownable {
    using SafeERC20 for IERC20;

    /// @dev Emitted when funds are deposited.
    event Deposit(address sender, address token, uint256 amount);

    /// @dev Emitted when funds are withdrawn.
    event Withdraw(address recipient, address token, uint256 amount);

    /// @dev Not enough funds to transfer.
    error InsufficientBalance();

    /// @dev stores the total balance of each token locked in the vault.
    mapping(address => uint128) public balance;

    function deposit(address sender, address token, uint128 amount) external onlyOwner {
        balance[token] += amount;
        IERC20(token).safeTransferFrom(sender, address(this), amount);
        emit Deposit(sender, token, amount);
    }

    function withdraw(address recipient, address token, uint128 amount) external onlyOwner {
        if (amount > balance[token]) {
            revert InsufficientBalance();
        }

        balance[token] -= amount;
        IERC20(token).safeTransfer(recipient, amount);

        emit Withdraw(recipient, token, amount);
    }
}
