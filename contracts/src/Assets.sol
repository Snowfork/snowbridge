// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.20;

import {IERC20} from "./interfaces/IERC20.sol";
import {IGateway} from "./interfaces/IGateway.sol";

import {SafeTokenTransferFrom} from "./utils/SafeTransfer.sol";

import {AssetsStorage} from "./storage/AssetsStorage.sol";
import {SubstrateTypes} from "./SubstrateTypes.sol";
import {ParaID, Config} from "./Types.sol";
import {Address} from "./utils/Address.sol";

/// @title Library for implementing Ethereum->Polkadot ERC20 transfers.
library Assets {
    using Address for address;
    using SafeTokenTransferFrom for IERC20;

    /// @dev Emitted once the funds are locked and a message is successfully queued.
    event TokenSent(
        address indexed token, address indexed sender, ParaID destinationChain, bytes destinationAddress, uint128 amount
    );
    event TokenRegistrationSent(address token);

    /* Errors */
    error InvalidToken();
    error InvalidAmount();
    error InvalidDestination();

    // This library requires state which must be initialized in the gateway's storage.
    function initialize(uint256 registerTokenFee, uint256 sendTokenFee) external {
        AssetsStorage.Layout storage $ = AssetsStorage.layout();

        $.registerTokenFee = registerTokenFee;
        $.sendTokenFee = sendTokenFee;
    }

    function sendToken(
        ParaID assetHubParaID,
        address assetHubAgent,
        address token,
        address sender,
        ParaID destinationChain,
        bytes32 destinationAddress,
        uint128 amount
    ) external returns (bytes memory payload, uint256 extraFee) {
        AssetsStorage.Layout storage $ = AssetsStorage.layout();

        _transferToAgent(assetHubAgent, token, sender, amount);
        if (destinationChain == assetHubParaID) {
            payload = SubstrateTypes.SendToken(address(this), token, destinationAddress, amount);
        } else {
            payload = SubstrateTypes.SendToken(address(this), token, destinationChain, destinationAddress, amount);
        }
        extraFee = $.sendTokenFee;

        emit TokenSent(sender, token, destinationChain, abi.encodePacked(destinationAddress), amount);
    }

    function sendToken(
        ParaID assetHubParaID,
        address assetHubAgent,
        address token,
        address sender,
        ParaID destinationChain,
        address destinationAddress,
        uint128 amount
    ) external returns (bytes memory payload, uint256 extraFee) {
        AssetsStorage.Layout storage $ = AssetsStorage.layout();
        if (destinationChain == assetHubParaID) {
            // AssetHub parachain doesn't support Ethereum-style addresses
            revert InvalidDestination();
        }

        _transferToAgent(assetHubAgent, token, sender, amount);

        payload = SubstrateTypes.SendToken(address(this), token, destinationChain, destinationAddress, amount);
        extraFee = $.sendTokenFee;
        emit TokenSent(sender, token, destinationChain, abi.encodePacked(destinationAddress), amount);
    }

    /// @dev transfer tokens from the sender to the specified
    function _transferToAgent(address assetHubAgent, address token, address sender, uint128 amount) internal {
        if (!token.isContract()) {
            revert InvalidToken();
        }

        if (amount == 0) {
            revert InvalidAmount();
        }

        IERC20(token).safeTransferFrom(sender, assetHubAgent, amount);
    }

    /// @dev Enqueues a create native token message to substrate.
    /// @param token The ERC20 token address.
    function registerToken(address token) external returns (bytes memory payload, uint256 extraFee) {
        AssetsStorage.Layout storage $ = AssetsStorage.layout();

        if (!token.isContract()) {
            revert InvalidToken();
        }

        payload = SubstrateTypes.RegisterToken(address(this), token);
        extraFee = $.registerTokenFee;

        emit TokenRegistrationSent(token);
    }
}
