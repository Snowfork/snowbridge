// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.25;

import {MultiAddress, TokenInfo, OperatingMode} from "../types/Common.sol";
import {UD60x18} from "prb/math/src/UD60x18.sol";

type ParaID is uint32;

using {ParaIDEq as ==, ParaIDNe as !=, into} for ParaID global;

function ParaIDEq(ParaID a, ParaID b) pure returns (bool) {
    return ParaID.unwrap(a) == ParaID.unwrap(b);
}

function ParaIDNe(ParaID a, ParaID b) pure returns (bool) {
    return !ParaIDEq(a, b);
}

function into(ParaID paraID) pure returns (ChannelID) {
    return ChannelID.wrap(keccak256(abi.encodePacked("para", ParaID.unwrap(paraID))));
}

type ChannelID is bytes32;

using {ChannelIDEq as ==, ChannelIDNe as !=} for ChannelID global;

function ChannelIDEq(ChannelID a, ChannelID b) pure returns (bool) {
    return ChannelID.unwrap(a) == ChannelID.unwrap(b);
}

function ChannelIDNe(ChannelID a, ChannelID b) pure returns (bool) {
    return !ChannelIDEq(a, b);
}

/// @dev A messaging channel for a Polkadot parachain
struct Channel {
    /// @dev The operating mode for this channel. Can be used to
    /// disable messaging on a per-channel basis.
    OperatingMode mode;
    /// @dev The current nonce for the inbound lane
    uint64 inboundNonce;
    /// @dev The current node for the outbound lane
    uint64 outboundNonce;
    /// @dev The address of the agent of the parachain owning this channel
    address agent;
}

/// @dev Inbound message from a Polkadot parachain (via BridgeHub)
struct InboundMessage {
    /// @dev The parachain from which this message originated
    ChannelID channelID;
    /// @dev The channel nonce
    uint64 nonce;
    /// @dev The command to execute
    Command command;
    /// @dev The Parameters for the command
    bytes params;
    /// @dev The maximum gas allowed for message dispatch
    uint64 maxDispatchGas;
    /// @dev The maximum fee per gas
    uint256 maxFeePerGas;
    /// @dev The reward for message submission
    uint256 reward;
    /// @dev ID for this message
    bytes32 id;
}

/// @dev Messages from Polkadot take the form of these commands.
enum Command {
    AgentExecute,
    Upgrade,
    CreateAgent,
    CreateChannel,
    UpdateChannel,
    SetOperatingMode,
    TransferNativeFromAgent,
    SetTokenTransferFees,
    SetPricingParameters,
    UnlockNativeToken,
    RegisterForeignToken,
    MintForeignToken
}

enum AgentExecuteCommand {
    TransferToken
}

/// @dev Application-level costs for a message
struct Costs {
    /// @dev Costs in foreign currency
    uint256 foreign;
    /// @dev Costs in native currency
    uint256 native;
}

struct Ticket {
    ParaID dest;
    Costs costs;
    bytes payload;
}

// Payload for AgentExecute
struct AgentExecuteParams {
    bytes32 agentID;
    bytes payload;
}

// Payload for CreateAgent
struct CreateAgentParams {
    /// @dev The agent ID of the consensus system
    bytes32 agentID;
}

// Payload for CreateChannel
struct CreateChannelParams {
    /// @dev The channel ID
    ChannelID channelID;
    /// @dev The agent ID
    bytes32 agentID;
    /// @dev Initial operating mode
    OperatingMode mode;
}

// Payload for UpdateChannel
struct UpdateChannelParams {
    /// @dev The parachain used to identify the channel to update
    ChannelID channelID;
    /// @dev The new operating mode
    OperatingMode mode;
}

// Payload for Upgrade
struct UpgradeParams {
    /// @dev The address of the implementation contract
    address impl;
    /// @dev the codehash of the new implementation contract.
    /// Used to ensure the implementation isn't updated while
    /// the upgrade is in flight
    bytes32 implCodeHash;
    /// @dev parameters used to upgrade storage of the gateway
    bytes initParams;
}

// Payload for SetOperatingMode
struct SetOperatingModeParams {
    /// @dev The new operating mode
    OperatingMode mode;
}

// Payload for TransferNativeFromAgent
struct TransferNativeFromAgentParams {
    /// @dev The ID of the agent to transfer funds from
    bytes32 agentID;
    /// @dev The recipient of the funds
    address recipient;
    /// @dev The amount to transfer
    uint256 amount;
}

// Payload for SetTokenTransferFees
struct SetTokenTransferFeesParams {
    /// @dev The remote fee (DOT) for registering a token on AssetHub
    uint128 assetHubCreateAssetFee;
    /// @dev The remote fee (DOT) for send tokens to AssetHub
    uint128 assetHubReserveTransferFee;
    /// @dev extra fee to register an asset and discourage spamming (Ether)
    uint256 registerTokenFee;
}

// Payload for SetPricingParameters
struct SetPricingParametersParams {
    /// @dev The ETH/DOT exchange rate
    UD60x18 exchangeRate;
    /// @dev The cost of delivering messages to BridgeHub in DOT
    uint128 deliveryCost;
    /// @dev Fee multiplier
    UD60x18 multiplier;
}

// Payload for TransferToken
struct UnlockNativeTokenParams {
    /// @dev The agent ID of the consensus system
    bytes32 agentID;
    /// @dev The token address
    address token;
    /// @dev The address of the recipient
    address recipient;
    /// @dev The amount to mint with
    uint128 amount;
}

// Payload for RegisterForeignToken
struct RegisterForeignTokenParams {
    /// @dev The token ID (hash of stable location id of token)
    bytes32 foreignTokenID;
    /// @dev The name of the token
    string name;
    /// @dev The symbol of the token
    string symbol;
    /// @dev The decimal of the token
    uint8 decimals;
}

// Payload for MintForeignToken
struct MintForeignTokenParams {
    /// @dev The token ID
    bytes32 foreignTokenID;
    /// @dev The address of the recipient
    address recipient;
    /// @dev The amount to mint with
    uint128 amount;
}
