// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.20;

contract Agent {
    error Unauthorized();

    // The unique ID for this agent, derived from the MultiLocation of the corresponding consensus system on Polkadot
    bytes32 public immutable AGENT_ID;

    // The gateway contract owning this agent
    address public immutable GATEWAY;

    constructor(bytes32 agentID) {
        AGENT_ID = agentID;
        GATEWAY = msg.sender;
    }

    receive() external payable {}

    function invoke(address executor, bytes calldata data) external returns (bool, bytes memory) {
        if (msg.sender != GATEWAY) {
            revert Unauthorized();
        }
        return executor.delegatecall(data);
    }
}
