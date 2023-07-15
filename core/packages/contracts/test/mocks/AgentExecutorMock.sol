// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.20;

contract AgentExecutorMock {
    address immutable gateway;

    constructor() {
        gateway = msg.sender;
    }

    uint256 counter;

    // consume all available gas
    function execute(address, bytes memory) external {
        while (true) {
            counter += 1;
        }
    }
}
