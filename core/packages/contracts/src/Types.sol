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

struct Channel {
    OperatingMode mode;
    uint64 inboundNonce;
    uint64 outboundNonce;
    address agent;
    uint256 fee;
    uint256 reward;
}

// Inbound message from a Polkadot parachain (via BridgeHub)
struct InboundMessage {
    ParaID origin;
    uint64 nonce;
    Command command;
    bytes params;
}

enum OperatingMode {
    Normal,
    RejectingOutboundMessages
}

// Initial configuration for bridge
struct Config {
    // default fee/reward parameters
    uint256 fee;
    uint256 reward;
    // Assets
    uint256 registerNativeTokenFee;
    uint256 sendNativeTokenFee;
}

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
