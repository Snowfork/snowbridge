// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.33;

import {OperatingMode} from "./../types/Common.sol";

// Inbound message from a Polkadot parachain (via BridgeHub)
struct InboundMessage {
    // origin
    bytes32 origin;
    // Message nonce
    uint64 nonce;
    // XCM Topic
    bytes32 topic;
    // Commands
    Command[] commands;
}

struct Command {
    uint8 kind;
    uint64 gas;
    bytes payload;
}

// Command IDs
library CommandKind {
    // Upgrade the Gateway implementation contract
    uint8 constant Upgrade = 0;
    // Set the operating mode for outbound messaging in the Gateway
    uint8 constant SetOperatingMode = 1;
    // Unlock native ERC20 and transfer to recipient
    uint8 constant UnlockNativeToken = 2;
    // Register a foreign token
    uint8 constant RegisterForeignToken = 3;
    // Mint foreign tokens
    uint8 constant MintForeignToken = 4;
    // Call an arbitrary solidity contract
    uint8 constant CallContract = 5;
    // Delegatecall into an arbitrary solidity contract
    uint8 constant DelegateCallContract = 6;
}

// Payload for outbound messages destined for Polkadot
struct Payload {
    // Sender of the message
    address origin;
    // Asset transfer instructions
    Asset[] assets;
    // XCM Message
    Xcm xcm;
    // SCALE-encoded location of claimer who can claim trapped assets on AssetHub
    bytes claimer;
    // Ether value not reserved for fees
    uint128 value;
    // Ether value reserved for execution fees on AH
    uint128 executionFee;
    // Ether value reserved for relayer incentives
    uint128 relayerFee;
}

struct Xcm {
    // Variant ID
    uint8 kind;
    // ABI-encoded xcm variant
    bytes data;
}

library XcmKind {
    // SCALE-encoded raw bytes for `VersionedXcm`
    uint8 constant Raw = 0;
    // Create a new asset in the ForeignAssets pallet of AH
    uint8 constant CreateAsset = 1;
}

// Format of ABI-encoded Xcm.data when Xcm.kind == XcmKind.CreateAsset
struct AsCreateAsset {
    // Token address
    address token;
    // Polkadot network to create the asset in
    uint8 network;
}

function makeRawXCM(bytes memory xcm) pure returns (Xcm memory) {
    return Xcm({kind: XcmKind.Raw, data: xcm});
}

function makeCreateAssetXCM(address token, Network network) pure returns (Xcm memory) {
    return Xcm({
        kind: XcmKind.CreateAsset,
        data: abi.encode(AsCreateAsset({token: token, network: uint8(network)}))
    });
}

struct Asset {
    uint8 kind;
    bytes data;
}

library AssetKind {
    uint8 constant NativeTokenERC20 = 0;
    uint8 constant ForeignTokenERC20 = 1;
}

// Format of Asset.data when Asset.kind == AssetKind.NativeTokenERC20
struct AsNativeTokenERC20 {
    address token;
    uint128 amount;
}

// Format of Asset.data when Asset.kind == AssetKind.ForeignTokenERC20
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

// Payload for DelegateCallContractParams instruction
struct DelegateCallContractParams {
    // target contract
    address target;
    // Delegatecall data
    bytes data;
}

enum Network {
    Polkadot
}
