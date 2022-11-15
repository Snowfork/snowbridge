// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "@openzeppelin/contracts/access/Ownable.sol";

/// @title Ether Vault.
/// @notice Holds Ether on behalf of ETHApp
contract EtherVault is Ownable {

    /// @dev Emitted when funds are deposited.
    /// @param account The address of the ERC20App contract.
    /// @param sender The address of the sender.
    /// @param amount The amount being deposited.
    event Deposit(address account, address sender, uint256 amount);

    /// @dev Emitted when funds are withdrawn.
    /// @param account The address of the ERC20App contract.
    /// @param recipient The address of the sender.
    /// @param amount The amount being withdrawn.
    event Withdraw(address account, address recipient, uint256 amount);

    /// @dev Recipient cannot withdraw funds.
    error CannotWithdraw();

    /// @dev Revert calls which send funds directly.
    receive() external payable {
        revert("Must use deposit function");
    }

    /// @dev Accepts ETH from the caller.
    /// @param _sender The address of the sender.
    function deposit(address _sender) 
        external
        payable
        onlyOwner
    {
        emit Deposit(msg.sender, _sender, msg.value);
    }

    /// @dev Returns ETH to the caller.
    /// @param _recipient The address that will receive funds.
    /// @param _amount The amount of ether that will be received.
    function withdraw(address payable _recipient, uint256 _amount)
        external
        onlyOwner
    {
        require(_amount > 0, "Must unlock a positive amount");
        (bool success, ) = _recipient.call{ value: _amount }("");
        if(!success) {
            revert CannotWithdraw();
        }
        emit Withdraw(msg.sender, _recipient, _amount);
    }
}