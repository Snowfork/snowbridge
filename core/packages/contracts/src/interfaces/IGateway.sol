// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.20;

import {OperatingMode, InboundMessage, ParaID} from "../Types.sol";
import {Verification} from "../Verification.sol";

interface IGateway {
    /**
     * Events
     */

    // Emitted when inbound message has been dispatched
    event InboundMessageDispatched(ParaID indexed origin, uint64 nonce, bool success);

    // Emitted when an outbound message has been accepted for delivery to a Polkadot parachain
    event OutboundMessageAccepted(ParaID indexed destination, uint64 nonce, bytes payload);

    // Emitted when an agent has been created for a consensus system on Polkadot
    event AgentCreated(bytes32 agentID, address agent);

    // Emitted when a channel has been created
    event ChannelCreated(ParaID indexed paraID);

    // Emitted when a channel has been updated
    event ChannelUpdated(ParaID indexed paraID);

    // Emitted when the gateway is upgraded
    event Upgraded(address indexed implementation);

    // Emitted when the operating mode is changed
    event OperatingModeChanged(OperatingMode mode);

    // Emitted when funds are withdrawn from an agent
    event AgentFundsWithdrawn(bytes32 indexed agentID, address indexed recipient, uint256 amount);

    event TokenRegistered(address token);

    event TokenSent(address token, ParaID destinationChain, bytes destinationAddress, uint128 amount);

    /**
     * Getters
     */

    function operatingMode() external view returns (OperatingMode);
    function channelOperatingModeOf(ParaID paraID) external view returns (OperatingMode);
    function channelFeeRewardOf(ParaID paraID) external view returns (uint256, uint256);
    function channelNoncesOf(ParaID paraID) external view returns (uint64, uint64);
    function agentOf(bytes32 agentID) external view returns (address);
    function implementation() external view returns (address);

    /**
     * Messaging
     */

    // Submit an inbound message for dispatch
    function submitInbound(
        InboundMessage calldata message,
        bytes32[] calldata leafProof,
        Verification.Proof calldata headerProof
    ) external;

    /**
     * Token Transfers
     */

    /// @dev Send a message to the AssetHub parachain to register a new fungible asset
    /// in the `ForeignAssets` pallet.
    function registerToken(address token) external payable;

    /// @dev Send ERC20 tokens to Polkadot.
    function sendToken(address token, ParaID destinationChain, bytes32 destinationAddress, uint128 amount)
        external
        payable;

    function sendToken(address token, ParaID destinationChain, address destinationAddress, uint128 amount)
        external
        payable;
}
