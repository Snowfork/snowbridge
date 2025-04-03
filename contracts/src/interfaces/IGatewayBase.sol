// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.28;

import {OperatingMode} from "../types/Common.sol";

/// Base interface for Gateway
interface IGatewayBase {
    error InvalidToken();
    error InvalidAmount();
    error InvalidDestination();
    error TokenNotRegistered();
    error Unsupported();
    error InvalidDestinationFee();
    error AgentDoesNotExist();
    error TokenAlreadyRegistered();
    error TokenMintFailed();
    error TokenTransferFailed();
    error InvalidProof();
    error InvalidNonce();
    error NotEnoughGas();
    error InsufficientEther();
    error Unauthorized();
    error Disabled();
    error AgentExecutionFailed(bytes returndata);
    error InvalidAgentExecutionPayload();
    error InvalidConstructorParams();
    error AlreadyInitialized();

    // Emitted when the operating mode is changed
    event OperatingModeChanged(OperatingMode mode);

    // Emitted when foreign token from polkadot registed
    event ForeignTokenRegistered(bytes32 indexed tokenID, address token);

    /// @dev Emitted when a command is sent to register a new wrapped token on AssetHub
    event TokenRegistrationSent(address token);
}
