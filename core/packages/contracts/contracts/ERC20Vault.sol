// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";

/// @title ERC20 Vault
/// @notice Holds ERC20 Tokens on behalf of ERC20App.
contract ERC20Vault is Ownable {
    using SafeERC20 for IERC20;

    /// @dev Emitted when funds are deposited.
    /// @param account The address of the ERC20App contract.
    /// @param sender The address of the sender.
    /// @param token The address of the ERC20 token.
    /// @param amount The amount being deposited.
    event Deposit(address account, address sender, address token, uint128 amount);

    /// @dev Emitted when funds are withdrawn.
    /// @param account The address of the ERC20App contract.
    /// @param recipient The address of the sender.
    /// @param token The address of the ERC20 token.
    /// @param amount The amount being withdrawn.
    event Withdraw(address account, address recipient, address token, uint128 amount);

    /// @dev Token Transfer failed.
    error TokenTransferFailed();

    /// @dev Not enough funds to transfer.
    error InsufficientBalance();

    /* State */
    mapping(address => uint256) public balances;

    /// @dev Accepts a ERC20 Token from the caller.
    /// @param _sender The address of the sender.
    /// @param _token The address of the Token.
    /// @param _amount The amount being deposited.
    function deposit(address _sender, address _token, uint128 _amount) external onlyOwner {
        balances[_token] = balances[_token] + _amount;
        // TODO: Transfer ERC20 tokens safely. https://linear.app/snowfork/issue/SNO-366
        if (!IERC20(_token).transferFrom(_sender, address(this), _amount)) {
            revert TokenTransferFailed();
        }
        emit Deposit(msg.sender, _sender, _token, _amount);
    }

    /// @dev Returns ETH to the caller.
    /// @param _recipient The address that will receive funds.
    /// @param _token The address of the Token.
    /// @param _amount The amount being deposited.
    function withdraw(address _recipient, address _token, uint128 _amount) external onlyOwner {
        if (_amount > balances[_token]) {
            revert InsufficientBalance();
        }

        balances[_token] = balances[_token] - _amount;
        IERC20(_token).safeTransfer(_recipient, _amount);
        emit Withdraw(msg.sender, _recipient, _token, _amount);
    }
}
