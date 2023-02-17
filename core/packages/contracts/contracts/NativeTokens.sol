// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "@openzeppelin/contracts/access/Ownable.sol";

import "./ERC20Vault.sol";
import "./OutboundChannel.sol";

/// @title Native Tokens
/// @notice A contract for managing ethereum native tokens.
/// @dev Manages locking, unlocking ERC20 tokens in the vault. Initializes ethereum native tokens on the substrate side via create.
contract NativeTokens is Ownable {
    struct Message {
        Action action;
        bytes payload;
    }

    enum Action {
        Unlock
    }

    struct UnlockPayload {
        address token;
        address recipient;
        uint128 amount;
    }

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

    function handle(bytes32 origin, bytes calldata message) external onlyOwner {}

    function lock(address token, bytes32 recipient, uint256 amount) public {}

    function create(
        address token,
        string calldata name,
        string calldata symbol,
        uint8 decimals
    ) public {}
}
