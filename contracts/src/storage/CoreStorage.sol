// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity ^0.8.33;

import {Channel, OperatingMode, ChannelID} from "../v1/Types.sol";
import {SparseBitmap} from "../utils/SparseBitmap.sol";

library CoreStorage {
    struct Layout {
        // Operating mode:
        OperatingMode mode;
        // Message channels
        mapping(ChannelID channelID => Channel) channels;
        // Agents
        mapping(bytes32 agentID => address) agents;
        // Reserve slot to prevent state collision
        uint256 __gap;
        // V2
        SparseBitmap inboundNonce;
        uint64 outboundNonce;
    }

    bytes32 internal constant SLOT = keccak256("org.snowbridge.storage.core");

    function layout() internal pure returns (Layout storage $) {
        bytes32 slot = SLOT;
        assembly {
            $.slot := slot
        }
    }
}
