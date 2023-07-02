// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.20;

contract Agent {
    error Unauthorized();
    error ExecutionFailed();

    address public immutable gateway;

    constructor() {
        gateway = msg.sender;
    }

    function invoke(address delegate, bytes calldata data) external {
        if (msg.sender != gateway) {
            revert Unauthorized();
        }
        (bool success,) = delegate.delegatecall(data);
        if (!success) {
            revert ExecutionFailed();
        }
    }
}
