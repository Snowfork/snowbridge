// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import { IRecipient } from "../IRecipient.sol";

contract RecipientMock is IRecipient {
    uint256 foo;
    function handle(bytes calldata, bytes calldata) external {
        while (true) {
            foo++;
        }
    }

}
