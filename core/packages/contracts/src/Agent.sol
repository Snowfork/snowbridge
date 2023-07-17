// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.20;

import {SafeNativeTransfer} from "./utils/SafeTransfer.sol";

contract Agent {
    using SafeNativeTransfer for address payable;

    error Unauthorized();
    error InvokeFailed();

    // The unique ID for this agent, derived from the MultiLocation of the corresponding consensus system on Polkadot
    bytes32 public immutable agentID;

    // The gateway contract owning this agent
    address public immutable gateway;

    constructor(bytes32 _agentID) {
        agentID = _agentID;
        gateway = msg.sender;
    }

    receive() external payable {}

    function withdrawTo(address payable recipient, uint256 amount) external {
        if (msg.sender != gateway) {
            revert Unauthorized();
        }
        recipient.safeNativeTransfer(amount);
    }

    function invoke(address delegate, bytes calldata data) external returns (bytes memory) {
        if (msg.sender != gateway) {
            revert Unauthorized();
        }
        (bool success, bytes memory result) = delegate.delegatecall(data);
        if (!success) {
            revert InvokeFailed();
        }

        return result;
    }
}
