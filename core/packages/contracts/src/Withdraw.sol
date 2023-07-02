// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.20;

contract Withdraw {
    error InsufficientBalance();
    error CannotSendFunds();

    function withdraw(address payable recipient, uint256 amount) external {
        if (address(this).balance < amount) {
            revert InsufficientBalance();
        }

        (bool success,) = recipient.call{value: amount}("");
        if (!success) {
            revert CannotSendFunds();
        }
    }
}
