// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "@openzeppelin/contracts/access/Ownable.sol";

// Holds ETH on behalf of ETHApp
contract EtherVault is Ownable {
    // Accepts ETH from the caller.
    receive() external payable {}

    // Returns ETH to the caller.
    function unlock(uint128 _amount)
        public
        onlyOwner
    {
        require(_amount > 0, "Must unlock a positive amount");
        (bool success, ) = msg.sender.call{ value: _amount }("");
        require(success, "Unable to send Ether");
    }
}
