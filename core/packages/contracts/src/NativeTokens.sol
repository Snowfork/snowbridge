// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.20;

import {Ownable} from "openzeppelin/access/Ownable.sol";
import {AccessControl} from "openzeppelin/access/AccessControl.sol";
import {IERC20Metadata} from "openzeppelin/token/ERC20/extensions/IERC20Metadata.sol";

import {TokenVault} from "./TokenVault.sol";
import {SubstrateTypes} from "./SubstrateTypes.sol";
import {NativeTokensTypes} from "./NativeTokensTypes.sol";
import {IOutboundQueue} from "./OutboundQueue.sol";
import {ParaID} from "./Types.sol";
import {Gateway} from "./Gateway.sol";
import {Registry} from "./Registry.sol";

/// @title Native Tokens
/// @dev Manages locking, unlocking ERC20 tokens in the vault. Initializes ethereum native
/// tokens on the substrate side via create.
contract NativeTokens is Gateway {
    /// @dev Describes the type of message.
    enum Action {Unlock}

    /// @dev Message format.
    struct Message {
        /// @dev The action type.
        Action action;
        /// @dev The message payload.
        bytes payload;
    }

    /// @dev Unlock payload format.
    struct UnlockPayload {
        /// @dev The ERC20 token to unlock.
        address token;
        /// @dev The destination address that will receive unlocked funds.
        address recipient;
        /// @dev The amount to unlock.
        uint128 amount;
    }

    /// @dev Emitted once the funds are locked and a message is successfully queued.
    event Locked(bytes recipient, address token, uint128 amount);

    /// @dev Emitted once the funds are unlocked.
    event Unlocked(address recipient, address token, uint128 amount);

    /// @dev Emitted after enqueueing a a create token message to substrate.
    event Created(address token);

    /// @dev Set a new outbound channel.
    event OutboundQueueUpdated(address newOutboundQueue);

    /* State */

    // Parachain ID of AssetHub (aka Statemint)
    ParaID public immutable assetHubParaID;

    TokenVault public immutable vault;

    uint256 public createTokenFee;

    /* Constants */

    // Call index for ForeignAssets::create dispatchable on AssetHub parachain
    bytes2 public immutable createCallId;

    // Call index for ForeignAssets::set_metata dispatchable AssetHub parachain
    bytes2 public immutable setMetadataCallId;

    /* Errors */

    error InvalidAmount();
    error NoFundsforCreateToken();

    constructor(
        Registry registry,
        TokenVault _vault,
        ParaID _assetHubParaID,
        uint256 _createTokenFee,
        bytes2 _createCallId,
        bytes2 _setMetadataCallId
    ) Gateway(registry) {
        vault = _vault;
        assetHubParaID = _assetHubParaID;
        createTokenFee = _createTokenFee;
        createCallId = _createCallId;
        setMetadataCallId = _setMetadataCallId;
    }

    /// @dev Locks an amount of ERC20 Tokens in the vault and enqueues a mint message.
    /// Requires the allowance to be set on the ERC20 token where the spender is the vault.
    /// @param token The token to lock.
    /// @param recipient The recipient on the substrate side.
    /// @param amount The amount to lock.
    function lock(address token, ParaID dest, bytes calldata recipient, uint128 amount) external payable {
        if (amount == 0) {
            revert InvalidAmount();
        }

        vault.deposit(msg.sender, token, amount);

        bytes memory payload = NativeTokensTypes.Mint(address(registry), token, dest, recipient, amount);
        outboundQueue().submit{value: msg.value}(assetHubParaID, payload);

        emit Locked(recipient, token, amount);
    }

    /// @dev Enqueues a create native token message to substrate.
    /// @param token The ERC20 token address.
    function create(address token) external payable {
        // to avoid spam, charge a fee for creating a new token
        if (msg.value < createTokenFee) {
            revert NoFundsforCreateToken();
        }

        IERC20Metadata metadata = IERC20Metadata(token);
        bytes memory name = bytes(metadata.name());
        if (name.length > 32) {
            name = hex"";
        }
        bytes memory symbol = bytes(metadata.symbol());
        if (symbol.length > 32) {
            symbol = hex"";
        }
        uint8 decimals = metadata.decimals();

        bytes memory payload =
            NativeTokensTypes.Create(address(registry), token, name, symbol, decimals, createCallId, setMetadataCallId);
        outboundQueue().submit{value: msg.value}(assetHubParaID, payload);

        emit Created(token);
    }

    /// @dev Processes messages from inbound channel.
    /// @param origin The multilocation of the source parachain
    /// @param message The message enqueued from substrate.
    function handle(ParaID origin, bytes calldata message) external override onlyRole(SENDER_ROLE) {
        ensureOrigin(origin, assetHubParaID);

        Message memory decoded = abi.decode(message, (Message));
        if (decoded.action == Action.Unlock) {
            UnlockPayload memory payload = abi.decode(decoded.payload, (UnlockPayload));
            vault.withdraw(payload.recipient, payload.token, payload.amount);
            emit Unlocked(payload.recipient, payload.token, payload.amount);
        }
    }
}
