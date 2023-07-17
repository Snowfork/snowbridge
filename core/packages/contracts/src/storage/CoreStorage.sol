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
        // All agents
        mapping(bytes32 agentID => address) agents;
        // Executor logic for agents
        address agentExecutor;
        // The fee charged to users for submitting outbound message to Polkadot
        uint256 defaultFee;
        // The reward given to relayers for submitting inbound messages from Polkadot
        uint256 defaultReward;
        // Parachain ID of BridgeHub
        ParaID bridgeHubParaID;
    }

    bytes32 internal constant SLOT = keccak256("org.snowbridge.storage.core");

    function layout() internal pure returns (Layout storage $) {
        bytes32 slot = SLOT;
        assembly {
            $.slot := slot
        }
    }
}
