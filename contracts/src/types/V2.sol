// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.25;

import {ParaID} from "./Common.sol";

struct Origin {
    // Origin network
    uint8 network;
    // Origin chain
    ParaID paraID;
    // Stable hash of full origin location;
    bytes32 locationID;
}

// Inbound message from a Polkadot parachain (via BridgeHub)
struct InboundMessage {
    // Stable ID of origin consensus system
    Origin origin;
    // @dev Non-consensus ID for this message
    bytes32 id;
    // @dev The message nonce
    uint64 nonce;
    // @dev The command to run
    bytes command;
    // The maximum gas allowed for message dispatch
    uint64 maxDispatchGas;
    // The maximum fee per gas
    uint256 maxFeePerGas;
    // The reward for message submission
    uint256 reward;
}

library Command {
    uint8 constant Upgrade = 0;
    uint8 constant SetOperatingMode = 1;
    uint8 constant SetPricingParameters = 2;
    uint8 constant SetTokenTransferFees = 3;

    uint8 constant CreateAgent = 20;
    uint8 constant TransferFromAgent = 21;
    uint8 constant TransferNativeFromAgent = 22;
}
