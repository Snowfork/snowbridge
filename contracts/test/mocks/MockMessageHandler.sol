// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.33;

import {IMessageHandler} from "../../src/l2-integration/interfaces/ISpokePool.sol";

contract MockMessageHandler is IMessageHandler {
    function handleV3AcrossMessage(address, uint256, address, bytes memory) external override {}
}
