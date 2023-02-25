// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/token/ERC20/extensions/IERC20Metadata.sol";

import "./ERC20Vault.sol";
import "./SubstrateTypes.sol";
import "./NativeTokensTypes.sol";
import "./OutboundChannel.sol";

/// @title Native Tokens
/// @dev Manages locking, unlocking ERC20 tokens in the vault. Initializes ethereum native
/// tokens on the substrate side via create.
contract NativeTokens is Ownable {
    /// @dev Describes the type of message.
    enum Action {
        Unlock
    }

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
    event Locked(address origin, bytes32 recipient, address token, uint128 amount);

    /// @dev Emitted once the funds are unlocked.
    event Unlocked(bytes32 origin, address recipient, address token, uint128 amount);

    /// @dev Emitted after enqueueing a a create token message to substrate.
    event Created(address token);

    /* State */

    bytes32 public immutable peerID;
    bytes public immutable peer;

    ERC20Vault public immutable vault;
    OutboundChannel public immutable outboundChannel;

    /* Errors */

    error InvalidAmount();
    error InvalidMessage();
    error Unauthorized();

    constructor(ERC20Vault _vault, OutboundChannel _outboundChannel, bytes32 _peer) {
        vault = _vault;
        outboundChannel = _outboundChannel;
        peer = _peer;
        peerID = keccak256(_peer);
    }

    /// @dev Locks an amount of ERC20 Tokens in the vault and enqueues a mint message.
    /// Requires the allowance to be set on the ERC20 token where the spender is the vault.
    /// @param token The token to lock.
    /// @param recipient The recipient on the substrate side.
    /// @param amount The amount to lock.
    function lock(address token, bytes recipient, uint128 amount) public {
        if (amount == 0) {
            revert InvalidAmount();
        }

        vault.deposit(msg.sender, token, amount);

        bytes memory payload = NativeTokensTypes.Mint(peer, token, recipient, amount);
        outboundChannel.submit(peerID, payload);

        emit Locked(recipient, token, amount);
    }

    /// @dev Enqueues a create native token message to substrate.
    /// @param token The ERC20 token address.
    function create(address token) external {
        IERC20Metadata metadata = IERC20Metadata(token);

        bytes memory name = bytes(metadata.name());
        if (name.length > 32) {
            name = hex"";
        }
        string memory symbol = bytes(metadata.symbol());
        if (symbol.length > 32) {
            symbol = hex"";
        }
        uint8 decimals = metadata.decimals();

        bytes memory payload = NativeTokensTypes.Create(peer, token, name, symbol, decimals);
        outboundChannel.submit(peerID, payload);

        emit Created(token);
    }

    /// @dev Processes messages from inbound channel.
    /// @param origin The hashed multilocation of the source parachain
    /// @param message The message enqueued from substrate.
    function handle(bytes32 origin, bytes calldata message) external onlyOwner {
        if (origin != peerID) {
            revert Unauthorized();
        }

        Message memory decoded = abi.decode(message, (Message));
        if (decoded.action == Action.Unlock) {
            doUnlock(origin, abi.decode(decoded.payload, (UnlockPayload)));
        } else {
            revert InvalidMessage();
        }
    }

    function doUnlock(bytes32 origin, UnlockPayload memory payload) private {
        emit Unlocked(origin, payload.recipient, payload.token, payload.amount);
        if (payload.amount > 0) {
            vault.withdraw(payload.recipient, payload.token, payload.amount);
        }
    }
}
