// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "@openzeppelin/contracts/access/Ownable.sol";

import "./ERC20Vault.sol";
import "./OutboundChannel.sol";

/// @title Native Tokens
/// @notice A contract for managing ethereum native tokens.
/// @dev Manages locking, unlocking ERC20 tokens in the vault. Initializes ethereum native tokens on the substrate side via create.
contract NativeTokens is Ownable {
    enum Action {
        Unlock
    }

    struct Message {
        Action action;
        bytes payload;
    }

    struct UnlockPayload {
        address token;
        address recipient;
        uint256 amount;
    }

    /// @notice Funds where locked.
    /// @dev Emitted once the funds are locked and a message is successfully queued.
    /// @param sender The address which initiated the lock.
    /// @param recipient The substrate address that will receive the funds.
    /// @param token The token locked.
    /// @param amount The amount locked.
    event Locked(address sender, bytes32 recipient, address token, uint256 amount);

    /// @dev The vault where ERC20 tokens are locked.
    ERC20Vault public immutable vault;

    /// @dev The channel used to enqueue messages for lock and create functions.
    OutboundChannel public immutable outboundChannel;

    /// @notice Initializes the NativeTokens contract with a vault and channels.
    /// @param _vault The vault to use to `lock`/`unlock` tokens.
    /// @param inboundChannel The owning channel allowed to call `handle` function.
    /// @param _outboundChannel The channel used to queue lock and create messages.
    constructor(ERC20Vault _vault, address inboundChannel, OutboundChannel _outboundChannel) {
        vault = _vault;
        outboundChannel = _outboundChannel;
        //TODO: Potentially move this to deployment/setup scripts and then we can drop inboundChannel parameter.
        transferOwnership(inboundChannel);
    }

    /// @notice Locks tokens to mint on substrate.
    /// @dev Locks an amount of ERC20 Tokens in the vault and enqueues a mint message. Requires the allowance to be set on the ERC20 token where the spender is the vault.
    /// @param token The token to lock.
    /// @param recipient The recipient on the substrate side.
    /// @param amount The amount to lock.
    function lock(address token, bytes32 recipient, uint256 amount) public {
        require(amount > 0, "NativeTokes: non zero amount");
        vault.deposit(msg.sender, token, amount);

        /// TODO: Encode a call
        bytes memory call;
        /// TODO: Get weight
        uint64 weight = 1_000_000;
        outboundChannel.submit(msg.sender, call, weight);
        emit Locked(msg.sender, recipient, token, amount);
    }

    function handle(bytes32 origin, bytes calldata message) external onlyOwner {}

    function create(
        address token,
        string calldata name,
        string calldata symbol,
        uint8 decimals
    ) public {}
}
