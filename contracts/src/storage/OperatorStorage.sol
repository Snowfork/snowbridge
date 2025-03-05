// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.28;

library OperatorStorage {
    struct Layout {
        address operator;
    }

    bytes32 internal constant SLOT = keccak256("org.snowbridge.storage.operator");

    function layout() internal pure returns (Layout storage $) {
        bytes32 slot = SLOT;
        assembly {
            $.slot := slot
        }
    }
}
