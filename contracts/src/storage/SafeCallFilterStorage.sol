// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.23;

library SafeCallFilterStorage {
    struct Layout {
        mapping(address => mapping(bytes4 => bool)) safeCalls;
    }

    bytes32 internal constant SLOT = keccak256("org.snowbridge.storage.safeCallFilter");

    function layout() internal pure returns (Layout storage $) {
        bytes32 slot = SLOT;
        assembly {
            $.slot := slot
        }
    }
}
