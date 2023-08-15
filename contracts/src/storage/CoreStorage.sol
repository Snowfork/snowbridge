// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.20;

import {AgentExecutor} from "../AgentExecutor.sol";
import {Agent} from "../Agent.sol";
import {Channel, OperatingMode, ParaID} from "../Types.sol";

library CoreStorage {
    struct Layout {
        // Operating mode:
        OperatingMode mode;
        // Message channels
        mapping(ParaID paraID => Channel) channels;
        // Agents
        mapping(bytes32 agentID => address) agents;
        // AgentIDs of token owners
        mapping(address token => bytes32) ownerAgentIDs;
        // The default fee charged to users for submitting outbound message to Polkadot
        uint256 defaultFee;
        // The default reward given to relayers for submitting inbound messages from Polkadot
        uint256 defaultReward;
    }

    bytes32 internal constant SLOT = keccak256("org.snowbridge.storage.core");

    function layout() internal pure returns (Layout storage $) {
        bytes32 slot = SLOT;
        assembly {
            $.slot := slot
        }
    }
}
