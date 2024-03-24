// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.23;

import {OperatingMode, InboundMessage, ParaID, ChannelID, MultiAddress} from "../Types.sol";
import {Verification} from "../Verification.sol";
import {UD60x18} from "prb/math/src/UD60x18.sol";
import {TokenInfo} from "../storage/AssetsStorage.sol";

interface IGateway {
    /**
     * Events
     */

    // Emitted when inbound message has been dispatched
    event InboundMessageDispatched(ChannelID indexed channelID, uint64 nonce, bytes32 indexed messageID, bool success);

    // Emitted when an agent has been created for a consensus system on Polkadot
    event AgentCreated(bytes32 agentID, address agent);

    // Emitted when a channel has been created
    event ChannelCreated(ChannelID indexed channelID);

    // Emitted when a channel has been updated
    event ChannelUpdated(ChannelID indexed channelID);

    // Emitted when the gateway is upgraded
    event Upgraded(address indexed implementation);

    // Emitted when the operating mode is changed
    event OperatingModeChanged(OperatingMode mode);

    // Emitted when pricing params updated
    event PricingParametersChanged();

    // Emitted when funds are withdrawn from an agent
    event AgentFundsWithdrawn(bytes32 indexed agentID, address indexed recipient, uint256 amount);

    // Emitted when foreign token registed
    event ForeignTokenRegistered(bytes32 indexed tokenID, bytes32 agentID, address token);

    /**
     * Getters
     */
    function operatingMode() external view returns (OperatingMode);
    function channelOperatingModeOf(ChannelID channelID) external view returns (OperatingMode);
    function channelNoncesOf(ChannelID channelID) external view returns (uint64, uint64);
    function agentOf(bytes32 agentID) external view returns (address);
    function pricingParameters() external view returns (UD60x18, uint128);
    function implementation() external view returns (address);

    /**
     * Messaging
     */

    // Submit a message from a Polkadot network
    function submitV1(
        InboundMessage calldata message,
        bytes32[] calldata leafProof,
        Verification.Proof calldata headerProof
    ) external;

    /**
     * Token Transfers
     */

    // @dev Emitted when the fees updated
    event TokenTransferFeesChanged();
}
