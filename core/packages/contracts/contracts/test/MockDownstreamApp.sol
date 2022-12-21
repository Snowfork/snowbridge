// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

/// @dev mock app for testing proxies.
contract MockDownstreamApp {
    /// @dev An event to record the sender observed.
    /// @param sender The msg sender.
    event RecordSender(address sender);

    /// @dev emits an event with msg sender.
    function recordMsgSender() external {
        emit RecordSender(msg.sender);
    }
}
