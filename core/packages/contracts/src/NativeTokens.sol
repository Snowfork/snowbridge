// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

import "openzeppelin/access/Ownable.sol";
import "openzeppelin/access/AccessControl.sol";
import "openzeppelin/token/ERC20/extensions/IERC20Metadata.sol";

import "./TokenVault.sol";
import "./SubstrateTypes.sol";
import "./NativeTokensTypes.sol";
import "./OutboundChannel.sol";

/// @title Native Tokens
/// @dev Manages locking, unlocking ERC20 tokens in the vault. Initializes ethereum native
/// tokens on the substrate side via create.
contract NativeTokens is AccessControl {
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
    event OutboundChannelUpdated(address newOutBoundChannel);

    /* State */

    bytes32 public constant ADMIN_ROLE = keccak256("ADMIN_ROLE");
    bytes32 public constant SENDER_ROLE = keccak256("SENDER_ROLE");

    bytes32 public immutable peerID;
    bytes public peer;

    TokenVault public immutable vault;
    IOutboundChannel public outboundChannel;

    uint256 public createTokenFee;

    /* Errors */

    error InvalidAmount();
    error Unauthorized();
    error NoFundsforCreateToken();

    constructor(TokenVault _vault, IOutboundChannel _outboundChannel, bytes memory _peer, uint256 _createTokenFee) {
        _grantRole(ADMIN_ROLE, msg.sender);
        _setRoleAdmin(SENDER_ROLE, ADMIN_ROLE);
        vault = _vault;
        outboundChannel = _outboundChannel;
        peer = _peer;
        peerID = keccak256(_peer);
        createTokenFee = _createTokenFee;
    }

    /// @dev Locks an amount of ERC20 Tokens in the vault and enqueues a mint message.
    /// Requires the allowance to be set on the ERC20 token where the spender is the vault.
    /// @param token The token to lock.
    /// @param recipient The recipient on the substrate side.
    /// @param amount The amount to lock.
    function lock(address token, bytes calldata recipient, uint128 amount) external payable {
        if (amount == 0) {
            revert InvalidAmount();
        }

        vault.deposit(msg.sender, token, amount);

        bytes memory payload = NativeTokensTypes.Mint(peer, token, recipient, amount);
        outboundChannel.submit{value: msg.value}(peer, payload);

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

        bytes memory payload = NativeTokensTypes.Create(peer, token, name, symbol, decimals);
        outboundChannel.submit{value: msg.value}(peer, payload);

        emit Created(token);
    }

    /// @dev Processes messages from inbound channel.
    /// @param origin The multilocation of the source parachain
    /// @param message The message enqueued from substrate.
    function handle(bytes calldata origin, bytes calldata message) external onlyRole(SENDER_ROLE) {
        if (peerID != keccak256(origin)) {
            revert Unauthorized();
        }

        Message memory decoded = abi.decode(message, (Message));
        if (decoded.action == Action.Unlock) {
            UnlockPayload memory payload = abi.decode(decoded.payload, (UnlockPayload));
            vault.withdraw(payload.recipient, payload.token, payload.amount);
            emit Unlocked(payload.recipient, payload.token, payload.amount);
        }
    }

    function setOutboundChannel(IOutboundChannel _outboundChannel) external onlyRole(ADMIN_ROLE) {
        outboundChannel = _outboundChannel;
        emit OutboundChannelUpdated(address(_outboundChannel));
    }
}
