// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "@openzeppelin/contracts/access/Ownable.sol";

import "./utils/MessageProtocol.sol";
import "./ERC20Vault.sol";
import "./OutboundChannel.sol";

/// @title Native Tokens
/// @notice A contract for managing ethereum native tokens.
/// @dev Manages locking, unlocking ERC20 tokens in the vault. Initializes ethereum native tokens on the substrate side via create.
contract NativeTokens is Ownable {
    /// @notice Unlock payload format.
    struct UnlockPayload {
        /// @notice The ERC20 token to unlock.
        address token;
        /// @notice The destination address that will receive unlocked funds.
        address recipient;
        /// @notice The amount to unlock.
        uint256 amount;
    }

    /// @notice Funds where locked.
    /// @dev Emitted once the funds are locked and a message is successfully queued.
    /// @param origin The address which initiated the lock.
    /// @param recipient The substrate address that will receive the funds.
    /// @param token The token locked.
    /// @param amount The amount locked.
    event Locked(address origin, bytes32 recipient, address token, uint256 amount);

    /// @notice Funds where unlocked.
    /// @dev Emitted once the funds are unlocked.
    /// @param origin The substrate address which initiated the unlock.
    /// @param recipient The ethereyn address that will receive the funds.
    /// @param token The token unlocked.
    /// @param amount The amount unlocked.
    event Unlocked(bytes32 origin, address recipient, address token, uint256 amount);

    /// @notice a token was created in Statemint.
    /// @dev Emitted after enqueueing a a create token message to substrate.
    event Created(address token);

    /// @dev the origin that
    bytes32 public immutable allowedOrigin;

    /// @dev The vault where ERC20 tokens are locked.
    ERC20Vault public immutable vault;

    /// @dev The channel used to enqueue messages for lock and create functions.
    OutboundChannel public immutable outboundChannel;

    /// @notice Initializes the NativeTokens contract with a vault and channels.
    /// @param _vault The vault to use to `lock`/`unlock` tokens.
    /// @param _outboundChannel The channel used to queue lock and create messages.
    /// @param _allowedOrigin The origin allowed to call handle.
    constructor(ERC20Vault _vault, OutboundChannel _outboundChannel, bytes32 _allowedOrigin) {
        vault = _vault;
        outboundChannel = _outboundChannel;
        allowedOrigin = _allowedOrigin;
    }

    /// @notice Locks tokens to mint on substrate.
    /// @dev Locks an amount of ERC20 Tokens in the vault and enqueues a mint message. Requires the allowance to be set on the ERC20 token where the spender is the vault.
    /// @param token The token to lock.
    /// @param recipient The recipient on the substrate side.
    /// @param amount The amount to lock.
    function lock(address token, bytes32 recipient, uint256 amount) public {
        require(amount > 0, "NativeTokens: zero amount");
        require(token != address(0), "NativeTokens: zero address token");
        require(recipient != bytes32(0), "NativeTokens: zero address recipient");

        /// TODO: Implement a max locked amount.
        vault.deposit(msg.sender, token, amount);

        /// TODO: Encode a call
        bytes memory call;
        /// TODO: Get weight
        uint64 weight = 1_000_000;

        emit Locked(msg.sender, recipient, token, amount);
        outboundChannel.submit(msg.sender, call, weight);
    }

    /// @notice Creates a native token.
    /// @dev Enqueues a create native token message to substrate.
    /// @param token The ERC20 token address.
    /// @param name The name of the ERC20 token.
    /// @param symbol The symbol of the ERC20 token.
    /// @param decimals The decimals for the ERC20 token.
    function create(
        address token,
        string calldata name,
        string calldata symbol,
        uint8 decimals
    ) public {
        /// TODO: Encode a call
        bytes memory call;
        /// TODO: Get weight
        uint64 weight = 1_000_000;

        emit Created(token);
        outboundChannel.submit(msg.sender, call, weight);
    }

    /// @notice Handles messages coming in over the bridge.
    /// @dev Processes messages from inbound channel.
    /// @param origin The hashed substrate sovereign account.
    /// @param message The message enqueued from substrate.
    function handle(bytes32 origin, bytes calldata message) external onlyOwner {
        require(origin == allowedOrigin, "NativeTokens: unknown origin");
        MessageProtocol.Message memory decoded = abi.decode(message, (MessageProtocol.Message));
        if (decoded.action == MessageProtocol.Action.Unlock) {
            unlock(origin, abi.decode(decoded.payload, (UnlockPayload)));
        } else {
            revert("NativeTokens: unknown action");
        }
    }

    /// @notice Unlocks funds from the vault and sends it to recipient.
    /// @param origin The hashed substrate sovereign account.
    /// @param payload A decoded unlock payload.
    function unlock(bytes32 origin, UnlockPayload memory payload) private {
        emit Unlocked(origin, payload.recipient, payload.token, payload.amount);
        if (payload.amount > 0) {
            vault.withdraw(payload.recipient, payload.token, payload.amount);
        }
    }
}
