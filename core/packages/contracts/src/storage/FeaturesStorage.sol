// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.20;

import {Agent} from "../Agent.sol";
import {ParaID} from "../Types.sol";

library FeaturesStorage {
    struct Layout {
        ParaID assetHubParaID;
        address assetHubAgent;
        uint256 createTokenFee;
        bytes2 createTokenCallId;
    }

    bytes32 constant SLOT = keccak256("org.snowbridge.storage.features");

    function layout() internal pure returns (Layout storage $) {
        bytes32 slot = SLOT;
        assembly {
            $.slot := slot
        }
    }
}
