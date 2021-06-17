// SPDX-License-Identifier: MIT
pragma solidity >=0.7.6;
pragma experimental ABIEncoderV2;

contract MaliciousApp {
    function unlock(
        address _token,
        bytes32 _sender,
        address _recipient,
        uint256 _amount
    ) external pure {
        while (true) {}
    }
}
