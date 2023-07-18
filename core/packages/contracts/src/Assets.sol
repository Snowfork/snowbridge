// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.20;

import {IERC20} from "./interfaces/IERC20.sol";
import {SafeTokenTransferFrom} from "./utils/SafeTransfer.sol";

import {AssetsStorage} from "./storage/AssetsStorage.sol";
import {SubstrateTypes} from "./SubstrateTypes.sol";
import {ParaID, Config} from "./Types.sol";
import {Address} from "./utils/Address.sol";

library Assets {
    using Address for address;
    using SafeTokenTransferFrom for IERC20;

    /// @dev Emitted once the funds are locked and a message is successfully queued.
    event NativeTokensLocked(address token, bytes recipient, uint128 amount);
    event NativeTokenRegistered(address token);

    /* Errors */

    error InvalidToken();
    error InvalidAmount();
    error NoFundsforCreateToken();

    function initialize(uint256 registerNativeTokenFee, uint256 sendNativeTokenFee) external {
        AssetsStorage.Layout storage $ = AssetsStorage.layout();

        $.registerNativeTokenFee = registerNativeTokenFee;
        $.sendNativeTokenFee = sendNativeTokenFee;
    }

    /// @dev Locks an amount of ERC20 Tokens in the vault and enqueues a mint message.
    /// Requires the allowance to be set on the ERC20 token where the spender is the vault.
    /// @param token The token to lock.
    /// @param recipient The recipient on Polkadot side. This is an encoded VersionedMultiLocation
    /// @param amount The amount to lock.
    function sendNativeToken(
        address assetHubAgent,
        address token,
        address sender,
        bytes calldata recipient,
        uint128 amount
    ) external returns (bytes memory payload, uint256 extraFee) {
        AssetsStorage.Layout storage $ = AssetsStorage.layout();

        if (!token.isContract()) {
            revert InvalidToken();
        }

        if (amount == 0) {
            revert InvalidAmount();
        }

        IERC20(token).safeTransferFrom(sender, assetHubAgent, amount);

        payload = SubstrateTypes.MintNativeToken(assetHubAgent, token, recipient, amount);
        extraFee = $.sendNativeTokenFee;

        emit NativeTokensLocked(token, recipient, amount);
    }

    /// @dev Enqueues a create native token message to substrate.
    /// @param token The ERC20 token address.
    function registerNativeToken(address assetHubAgent, bytes2 createTokenCallID, address token)
        external
        returns (bytes memory payload, uint256 extraFee)
    {
        AssetsStorage.Layout storage $ = AssetsStorage.layout();

        if (!token.isContract()) {
            revert InvalidToken();
        }

        payload = SubstrateTypes.CreateNativeToken(assetHubAgent, token, createTokenCallID);
        extraFee = $.registerNativeTokenFee;

        emit NativeTokenRegistered(token);
    }
}
