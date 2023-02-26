
// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import { IRecipient } from "../IRecipient.sol";

contract RecipientMock is IRecipient {

    function handle(bytes calldata origin, bytes calldata message) external {

    }


}
