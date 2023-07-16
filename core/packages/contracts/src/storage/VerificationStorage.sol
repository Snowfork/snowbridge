// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.20;

import {BeefyClient} from "../BeefyClient.sol";

library VerificationStorage {
    struct Layout {
        address beefyClient;
        uint32 parachainID;
        bytes4 encodedParachainID;
    }

    bytes32 constant SLOT = keccak256("org.snowbridge.storage.verification");

    function layout() internal pure returns (Layout storage $) {
        bytes32 slot = SLOT;
        assembly {
            $.slot := slot
        }
    }
}
