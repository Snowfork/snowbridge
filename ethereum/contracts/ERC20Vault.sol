// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";

// Holds ETH on behalf of ETHApp
contract ERC20Vault is Ownable {
    using SafeERC20 for IERC20;

    event Deposit(address account, address sender, uint256 amount);
    event Withdraw(address account, address recipient, uint256 amount);

    mapping(address => uint256) public balances;

    receive() external payable {
        revert("Must use deposit function");
    }

    // Accepts ETH from the caller.
    function deposit(address _sender, address _token, uint256 _amount) 
        external
        onlyOwner
    {
        balances[_token] = balances[_token] + _amount;
        require(
            IERC20(_token).transferFrom(_sender, address(this), _amount),
            "Contract token allowances insufficient to complete this lock request"
        );
        emit Deposit(msg.sender, _sender, _amount);
    }

    // Returns ETH to the caller.
    function withdraw(address _recipient, address _token, uint256 _amount)
        external
        onlyOwner
    {
        require(_amount > 0, "Must unlock a positive amount");
        require(
            _amount <= balances[_token],
            "ERC20 token balances insufficient to fulfill the unlock request"
        );

        balances[_token] = balances[_token] - _amount;
        IERC20(_token).safeTransfer(_recipient, _amount);
        emit Withdraw(msg.sender, _recipient, _amount);
    }
}
