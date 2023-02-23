// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/token/ERC20/extensions/IERC20Metadata.sol";

import "./ERC20Vault.sol";
import "./OutboundChannel.sol";

/// @title Native Tokens
/// @dev Manages locking, unlocking ERC20 tokens in the vault. Initializes ethereum native tokens on the substrate side via create.
contract NativeTokens is Ownable {
    /// @dev Describes the type of message.
    enum Action {
        /// @dev The default value for messages created.
        Uninitialized,
        /// @dev A message which unlocks funds for native tokens.
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
    /// @param origin The address which initiated the lock.
    /// @param recipient The substrate address that will receive the funds.
    /// @param token The token locked.
    /// @param amount The amount locked.
    event Locked(address origin, bytes32 recipient, address token, uint128 amount);

    /// @dev Emitted once the funds are unlocked.
    /// @param origin The substrate address which initiated the unlock.
    /// @param recipient The ethereyn address that will receive the funds.
    /// @param token The token unlocked.
    /// @param amount The amount unlocked.
    event Unlocked(bytes32 origin, address recipient, address token, uint128 amount);

    /// @dev Emitted after enqueueing a a create token message to substrate.
    /// @param token The address of the token created.
    // TODO: Remove name, symbol and decimals.
    event Created(address token, string name, string symbol, uint8 decimals);

    /// @dev Zero amount.
    error ZeroAmount();

    /// @dev Zero addressed token.
    error ZeroAddressToken();

    /// @dev Zero addressed recipient.
    error ZeroAddressRecipient();

    /// @dev the origin provided is unauthorized.
    error UnauthorizedOrigin();

    /// @dev the message action is unsupported.
    error UnsupportedMessageAction();

    /// @dev the origin that
    bytes32 public immutable allowedOrigin;

    /// @dev The vault where ERC20 tokens are locked.
    ERC20Vault public immutable vault;

    /// @dev The channel used to enqueue messages for lock and create functions.
    OutboundChannel public immutable outboundChannel;

    /// @dev Initializes the NativeTokens contract with a vault and channels.
    /// @param _vault The vault to use to `lock`/`unlock` tokens.
    /// @param _outboundChannel The channel used to queue lock and create messages.
    /// @param _allowedOrigin The origin allowed to call handle.
    constructor(ERC20Vault _vault, OutboundChannel _outboundChannel, bytes32 _allowedOrigin) {
        vault = _vault;
        outboundChannel = _outboundChannel;
        allowedOrigin = _allowedOrigin;
    }

    /// @dev Locks an amount of ERC20 Tokens in the vault and enqueues a mint message. Requires the allowance to be set on the ERC20 token where the spender is the vault.
    /// @param token The token to lock.
    /// @param recipient The recipient on the substrate side.
    /// @param amount The amount to lock.
    function lock(address token, bytes32 recipient, uint128 amount) public {
        if (amount == 0) {
            revert ZeroAmount();
        }
        if (token == address(0)) {
            revert ZeroAddressToken();
        }
        if (recipient == bytes32(0)) {
            revert ZeroAddressRecipient();
        }

        // TODO: Implement a max locked amount.
        vault.deposit(msg.sender, token, amount);

        // TODO: Encode a call
        bytes memory call;
        // TODO: Get weight
        uint64 weight = 1_000_000;

        emit Locked(msg.sender, recipient, token, amount);
        outboundChannel.submit(msg.sender, call, weight);
    }

    /// @dev Enqueues a create native token message to substrate.
    /// @param token The ERC20 token address.
    function create(address token) public {
        if (token == address(0)) {
            revert ZeroAddressToken();
        }

        IERC20Metadata metadata = IERC20Metadata(token);
        // TODO: Use metadata to encode the call.
        string memory name = metadata.name();
        string memory symbol = metadata.symbol();
        uint8 decimals = metadata.decimals();

        // TODO: Encode a call
        bytes memory call;
        // TODO: Get weight
        uint64 weight = 1_000_000;

        emit Created(token, name, symbol, decimals);
        outboundChannel.submit(msg.sender, call, weight);
    }

    /// @dev Processes messages from inbound channel.
    /// @param origin The hashed substrate sovereign account.
    /// @param message The message enqueued from substrate.
    function handle(bytes32 origin, bytes calldata message) external onlyOwner {
        if (origin != allowedOrigin) {
            revert UnauthorizedOrigin();
        }

        Message memory decoded = abi.decode(message, (Message));
        if (decoded.action == Action.Unlock) {
            doUnlock(origin, abi.decode(decoded.payload, (UnlockPayload)));
        } else {
            revert UnsupportedMessageAction();
        }
    }

    function doUnlock(bytes32 origin, UnlockPayload memory payload) private {
        emit Unlocked(origin, payload.recipient, payload.token, payload.amount);
        if (payload.amount > 0) {
            vault.withdraw(payload.recipient, payload.token, payload.amount);
        }
    }
}
