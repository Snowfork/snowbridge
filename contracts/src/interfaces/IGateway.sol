// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.22;

import {OperatingMode, InboundMessage, ParaID, ChannelID, MultiAddress} from "../Types.sol";
import {Verification} from "../Verification.sol";

interface IGateway {
    /**
     * Events
     */

    // Emitted when inbound message has been dispatched
    event InboundMessageDispatched(ChannelID indexed channelID, uint64 nonce, bytes32 indexed messageID, bool success);

    // Emitted when an outbound message has been accepted for delivery to a Polkadot parachain
    event OutboundMessageAccepted(ChannelID indexed channelID, uint64 nonce, bytes32 indexed messageID, bytes payload);

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

    // Emitted when funds are withdrawn from an agent
    event AgentFundsWithdrawn(bytes32 indexed agentID, address indexed recipient, uint256 amount);

    /**
     * Getters
     */

    function operatingMode() external view returns (OperatingMode);
    function channelOperatingModeOf(ChannelID channelID) external view returns (OperatingMode);
    function channelOutboundFeeOf(ChannelID channelID) external view returns (uint256);
    function channelNoncesOf(ChannelID channelID) external view returns (uint64, uint64);
    function agentOf(bytes32 agentID) external view returns (address);
    function implementation() external view returns (address);

    /**
     * Messaging
     */

    // Submit an inbound message from Polkadot
    function submitInbound(
        InboundMessage calldata message,
        bytes32[] calldata leafProof,
        Verification.Proof calldata headerProof
    ) external;

    /**
     * Token Transfers
     */

    // @dev Emitted when the fees updated
    event TokenTransferFeesChanged(uint256 register, uint256 send);

    /// @dev Emitted once the funds are locked and an outbound message is successfully queued.
    event TokenSent(
        address indexed token,
        address indexed sender,
        ParaID destinationChain,
        MultiAddress destinationAddress,
        uint128 amount
    );

    /// @dev Emitted when a command is sent to register a new wrapped token on AssetHub
    event TokenRegistrationSent(address token);

    // @dev Fees in Ether for registering and sending tokens respectively
    function tokenTransferFees() external view returns (uint256, uint256);

    /// @dev Send a message to the AssetHub parachain to register a new fungible asset
    ///      in the `ForeignAssets` pallet.
    function registerToken(address token) external payable;

    /// @dev Send ERC20 tokens to parachain `destinationChain` and deposit into account `destinationAddress`
    function sendToken(address token, ParaID destinationChain, MultiAddress calldata destinationAddress, uint128 amount)
        external
        payable;
}
