// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.20;

type ParaID is uint256;

using {ParaIDEq as ==, ParaIDNe as !=} for ParaID global;

function ParaIDEq(ParaID a, ParaID b) pure returns (bool) {
    return ParaID.unwrap(a) == ParaID.unwrap(b);
}

function ParaIDNe(ParaID a, ParaID b) pure returns (bool) {
    return !ParaIDEq(a, b);
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
    /// @dev The fee charged to users for submitting outbound messages
    uint256 fee;
    /// @dev The reward disbursed to message relayers for submitting inbound messages
    uint256 reward;
}

/// @dev Inbound message from a Polkadot parachain (via BridgeHub)
struct InboundMessage {
    /// @dev The parachain from which this message originated
    ParaID origin;
    /// @dev The channel nonce
    uint64 nonce;
    /// @dev The command to execute
    Command command;
    /// @dev The Parameters for the command
    bytes params;
    /// @dev The gas to cover the cost of a dispatch call
    uint256 dispatchGas;
}

enum OperatingMode {
    Normal,
    RejectingOutboundMessages
}

// Initial configuration for bridge
struct Config {
    /// @dev The default fee charged to users for submitting outbound messages.
    uint256 fee;
    /// @dev The default reward disbursed to message relayers for submitting inbound messages.
    uint256 reward;
    /// @dev The extra fee charged for registering tokens.
    uint256 registerNativeTokenFee;
    /// @dev The extra fee charged for sending tokens.
    uint256 sendNativeTokenFee;
}

/// @dev Messages from Polkadot take the form of these commands.
enum Command {
    AgentExecute,
    Upgrade,
    CreateAgent,
    CreateChannel,
    UpdateChannel,
    SetOperatingMode,
    TransferNativeFromAgent
}

enum AgentExecuteCommand {TransferToken}
