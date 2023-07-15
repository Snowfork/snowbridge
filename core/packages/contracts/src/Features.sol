// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.20;

import {Ownable} from "openzeppelin/access/Ownable.sol";
import {AccessControl} from "openzeppelin/access/AccessControl.sol";
import {IERC20} from "openzeppelin/token/ERC20/IERC20.sol";
import {SafeERC20} from "openzeppelin/token/ERC20/utils/SafeERC20.sol";

import {FeaturesStorage} from "./storage/FeaturesStorage.sol";

import {SubstrateTypes} from "./SubstrateTypes.sol";
import {ParaID} from "./Types.sol";

library Features {
    using SafeERC20 for IERC20;

    /// @dev Emitted once the funds are locked and a message is successfully queued.
    event NativeTokensLocked(address token, ParaID destParaID, bytes recipient, uint128 amount);
    event NativeTokenRegistered(address token);

    /* Errors */

    error InvalidAmount();
    error NoFundsforCreateToken();

    struct Storage {
        ParaID assetHubParaID;
        address assetHubAgent;
        uint256 createTokenFee;
        bytes2 createTokenCallId;
    }

    /// @dev Locks an amount of ERC20 Tokens in the vault and enqueues a mint message.
    /// Requires the allowance to be set on the ERC20 token where the spender is the vault.
    /// @param token The token to lock.
    /// @param recipient The recipient on the substrate side. This is an encoded VersionedMultiLocation
    /// @param amount The amount to lock.
    function lockNativeToken(
        address token,
        address sender,
        ParaID finalDestPara,
        bytes calldata recipient,
        uint128 amount
    ) external returns (ParaID dest, bytes memory payload) {
        FeaturesStorage.Layout storage $ = FeaturesStorage.layout();

        if (amount == 0) {
            revert InvalidAmount();
        }
        IERC20(token).safeTransferFrom(sender, $.assetHubAgent, amount);

        dest = $.assetHubParaID;
        payload = SubstrateTypes.MintNativeToken($.assetHubAgent, token, finalDestPara, recipient, amount);

        emit NativeTokensLocked(token, finalDestPara, recipient, amount);
    }

    /// @dev Enqueues a create native token message to substrate.
    /// @param token The ERC20 token address.
    function registerNativeToken(address token) external returns (ParaID dest, bytes memory payload) {
        FeaturesStorage.Layout storage $ = FeaturesStorage.layout();

        // to avoid spam, charge a fee for creating a new token
        if (msg.value < $.createTokenFee) {
            revert NoFundsforCreateToken();
        }

        dest = $.assetHubParaID;
        payload = SubstrateTypes.CreateNativeToken($.assetHubAgent, token, $.createTokenCallId);
        emit NativeTokenRegistered(token);
    }
}
