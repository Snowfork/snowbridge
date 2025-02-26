// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.28;

import {OperatingMode} from "./../types/Common.sol";

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
    uint64 gas;
    bytes payload;
}

library CommandKind {
    uint8 constant Upgrade = 0;
    uint8 constant SetOperatingMode = 1;
    uint8 constant UnlockNativeToken = 2;
    uint8 constant RegisterForeignToken = 3;
    uint8 constant MintForeignToken = 4;
    uint8 constant CallContract = 5;
}

struct Payload {
    // sender of the message
    address origin;
    Asset[] assets;
    bytes xcm;
    bytes claimer;
    // ether value
    uint128 value;
    // additional ether value for execution fees
    uint128 executionFee;
    // additional ether value for relayer fees
    uint128 relayerFee;
}

struct Asset {
    uint8 kind;
    bytes data;
}

library AssetKind {
    uint8 constant NativeTokenERC20 = 0;
    uint8 constant ForeignTokenERC20 = 1;
}

struct AsNativeTokenERC20 {
    address token;
    uint128 amount;
}

struct AsForeignTokenERC20 {
    bytes32 foreignID;
    uint128 amount;
}

function makeNativeAsset(address token, uint128 amount) pure returns (Asset memory) {
    return Asset({
        kind: AssetKind.NativeTokenERC20,
        data: abi.encode(AsNativeTokenERC20({token: token, amount: amount}))
    });
}

function makeForeignAsset(bytes32 foreignID, uint128 amount) pure returns (Asset memory) {
    return Asset({
        kind: AssetKind.ForeignTokenERC20,
        data: abi.encode(AsForeignTokenERC20({foreignID: foreignID, amount: amount}))
    });
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
    // Ether value
    uint256 value;
}

enum Network {
    Polkadot,
    Kusama
}
