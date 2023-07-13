// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.20;

import {Ownable} from "openzeppelin/access/Ownable.sol";
import {AccessControl} from "openzeppelin/access/AccessControl.sol";
import {IERC20} from "openzeppelin/token/ERC20/IERC20.sol";
import {SafeERC20} from "openzeppelin/token/ERC20/utils/SafeERC20.sol";

import {SubstrateTypes} from "./SubstrateTypes.sol";
import {ExecutorTypes} from "./ExecutorTypes.sol";
import {ParaID} from "./Types.sol";
import {Gateway} from "./Gateway.sol";

contract OutboundExecutor {
    using SafeERC20 for IERC20;

    /// @dev Emitted once the funds are locked and a message is successfully queued.
    event NativeTokenLocked(bytes recipient, address token, uint128 amount);
    event NativeTokenRegistered(address token);

    /* State */

    // Parachain ID of AssetHub (aka Statemint)
    ParaID public immutable assetHubParaID;

    address public immutable gateway;
    address public immutable assetHubAgent;

    uint256 public createTokenFee;

    /* Constants */

    // Call index for ForeignAssets::create dispatchable on AssetHub parachain
    bytes2 public immutable createCallId;

    /* Errors */

    error InvalidAmount();
    error NoFundsforCreateToken();

    constructor(
        address _gateway,
        address _assetHubAgent,
        ParaID _assetHubParaID,
        uint256 _createTokenFee,
        bytes2 _createCallId
    ) {
        gateway = _gateway;
        assetHubAgent = _assetHubAgent;
        assetHubParaID = _assetHubParaID;
        createTokenFee = _createTokenFee;
        createCallId = _createCallId;
    }

    /// @dev Locks an amount of ERC20 Tokens in the vault and enqueues a mint message.
    /// Requires the allowance to be set on the ERC20 token where the spender is the vault.
    /// @param token The token to lock.
    /// @param recipient The recipient on the substrate side. This is an encoded VersionedMultiLocation
    /// @param amount The amount to lock.
    function lockNativeToken(address token, ParaID dest, bytes calldata recipient, uint128 amount) external payable {
        if (amount == 0) {
            revert InvalidAmount();
        }

        IERC20(token).safeTransferFrom(msg.sender, assetHubAgent, amount);

        bytes memory payload = ExecutorTypes.MintNativeToken(assetHubAgent, token, dest, recipient, amount);
        gateway.submitOutbound{value: msg.value}(assetHubParaID, payload);

        emit NativeTokenLocked(recipient, token, amount);
    }

    /// @dev Enqueues a create native token message to substrate.
    /// @param token The ERC20 token address.
    function registerNativeToken(address token) external payable {
        bytes memory payload = ExecutorTypes.CreateNativeToken(assetHubAgent, token, createCallId);
        gateway.submitOutbound{value: msg.value}(assetHubParaID, payload);

        emit NativeTokenRegistered(token);
    }
}
