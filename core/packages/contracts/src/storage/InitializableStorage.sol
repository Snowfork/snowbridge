// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity ^0.8.20;

library InitializableStorage {
    struct Layout {
        /*
        * @dev Indicates that the contract has been initialized.
        * @custom:oz-retyped-from bool
        */
        uint8 initialized;
        /*
        * @dev Indicates that the contract is in the process of being initialized.
        */
        bool initializing;
    }

    bytes32 internal constant SLOT = keccak256("openzeppelin.contracts.storage.Initializable");

    function layout() internal pure returns (Layout storage $) {
        bytes32 slot = SLOT;
        assembly {
            $.slot := slot
        }
    }
}
