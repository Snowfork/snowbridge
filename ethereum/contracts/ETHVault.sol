// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "@openzeppelin/contracts/access/Ownable.sol";

contract ETHVault is Ownable {
    function lock() 
        public
        payable
        onlyOwner
    {
        require(msg.value > 0, "Value of transaction must be positive");
    }

    function unlock(uint128 _amount)
        public
        onlyOwner
    {
        require(_amount > 0, "Must unlock a positive amount");
        (bool success, ) = msg.sender.call{ value: _amount }("");
        require(success, "Unable to send Ether");
    }
}
