// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.25;

import {OperatingMode} from "./Common.sol";

// Inbound message from a Polkadot parachain (via BridgeHub)
struct InboundMessage {
    // origin
    bytes32 origin;
    // Message nonce
    uint64 nonce;
    // Commands
    Command[] commands;
}

struct Command {
    uint8 kind;
    uint256 gas;
    bytes payload;
}

library CommandKind {
    uint8 constant Upgrade = 0;
    uint8 constant SetOperatingMode = 1;
    uint8 constant UnlockNativeToken = 2;
    uint8 constant MintForeignToken = 3;
    uint8 constant CreateAgent = 4;
    uint8 constant CallContract = 5;
}

struct Ticket {
    bytes[] xfers;
    bytes xcm;
}

enum TransferKind {
    LocalReserve,
    DestinationReserve
}

// V2 Command Params

// Payload for Upgrade
struct UpgradeParams {
    // The address of the implementation contract
    address impl;
    // Codehash of the new implementation contract.
    bytes32 implCodeHash;
    // Parameters used to upgrade storage of the gateway
    bytes initParams;
}

// Payload for SetOperatingMode instruction
struct SetOperatingModeParams {
    /// The new operating mode
    OperatingMode mode;
}

// Payload for NativeTokenUnlock instruction
struct UnlockNativeTokenParams {
    // Token address
    address token;
    // Recipient address
    address recipient;
    // Amount to unlock
    uint128 amount;
}

// Payload for MintForeignTokenParams instruction
struct MintForeignTokenParams {
    // Foreign token ID
    bytes32 foreignTokenID;
    // Recipient address
    address recipient;
    // Amount to mint
    uint128 amount;
}

// Payload for CallContractParams instruction
struct CallContractParams {
    // target contract
    address target;
    // Call data
    bytes data;
}
