// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

library MessageProtocol {
    /// @notice Describes the type of message.
    enum Action {
        Unlock
    }

    /// @notice Message format.
    struct Message {
        /// @notice The action type.
        Action action;
        /// @notice The message payload.
        bytes payload;
    }
}
