// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "@openzeppelin/contracts/access/Ownable.sol";

// Holds ETH on behalf of ETHApp
contract EtherVault is Ownable {
    event Deposit(address account, uint256 amount);
    event Withdraw(address account, address recipient, uint256 amount);

    // Accepts ETH from the caller.
    receive() external payable {
        emit Deposit(msg.sender, msg.value);
    }

    // Returns ETH to the caller.
    function withdraw(address payable _recipient, uint256 _amount)
        external
        onlyOwner
    {
        require(_amount > 0, "Must unlock a positive amount");
        (bool success, ) = _recipient.call{ value: _amount }("");
        require(success, "Unable to send Ether");
        emit Withdraw(msg.sender, _recipient, _amount);
    }
}
