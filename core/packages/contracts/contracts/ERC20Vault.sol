// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";

/// @title ERC20 Vault
/// @dev Holds ERC20 Tokens on behalf of ERC20App.
contract ERC20Vault is Ownable {
    using SafeERC20 for IERC20;

    /// @dev Emitted when funds are deposited.
    /// @param account The address of the ERC20App contract.
    /// @param sender The address of the sender.
    /// @param token The address of the ERC20 token.
    /// @param amount The amount being deposited.
    event Deposit(address account, address sender, address token, uint256 amount);

    /// @dev Emitted when funds are withdrawn.
    /// @param account The address of the ERC20App contract.
    /// @param recipient The address of the sender.
    /// @param token The address of the ERC20 token.
    /// @param amount The amount being withdrawn.
    event Withdraw(address account, address recipient, address token, uint256 amount);

    /// @dev Not enough funds to transfer.
    error InsufficientBalance();

    /// @dev stores the total balance of each token locked in the vault.
    mapping(address => uint256) public balances;

    /// @dev Accepts an amount of a token from the caller.
    /// @param sender The address of the sender.
    /// @param token The address of the Token.
    /// @param amount The amount being deposited.
    function deposit(address sender, address token, uint256 amount) external onlyOwner {
        balances[token] = balances[token] + amount;
        IERC20(token).safeTransferFrom(sender, address(this), amount);
        emit Deposit(msg.sender, sender, token, amount);
    }

    /// @dev Returns an amount of a token to the caller.
    /// @param recipient The address that will receive funds.
    /// @param token The address of the Token.
    /// @param amount The amount being deposited.
    function withdraw(address recipient, address token, uint256 amount) external onlyOwner {
        if (amount > balances[token]) {
            revert InsufficientBalance();
        }

        balances[token] = balances[token] - amount;
        IERC20(token).safeTransfer(recipient, amount);
        emit Withdraw(msg.sender, recipient, token, amount);
    }
}
