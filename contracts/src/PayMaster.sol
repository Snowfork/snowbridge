// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.25;

import {SafeNativeTransfer} from "./utils/SafeTransfer.sol";

contract Paymaster {
    using SafeNativeTransfer for address payable;

    error Unauthorized();

    event Transfer(address recipient, uint256 amount);

    /// @dev The gateway contract controlling this agent
    address public immutable GATEWAY;

    constructor(bytes32 agentID) {
        GATEWAY = msg.sender;
    }

    receive() external payable {}

    function withdraw(address recipient, uint256 amount) external {
        if (msg.sender != GATEWAY) {
            revert Unauthorized();
        }
        payable(recipient).safeNativeTransfer(amount);
        emit Transfer(recipient, amount)
    }
}
