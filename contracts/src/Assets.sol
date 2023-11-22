// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.22;

import {IERC20} from "./interfaces/IERC20.sol";
import {IGateway} from "./interfaces/IGateway.sol";

import {SafeTokenTransferFrom} from "./utils/SafeTransfer.sol";

import {AssetsStorage} from "./storage/AssetsStorage.sol";
import {SubstrateTypes} from "./SubstrateTypes.sol";
import {ParaID, MultiAddress} from "./Types.sol";
import {Address} from "./utils/Address.sol";

/// @title Library for implementing Ethereum->Polkadot ERC20 transfers.
library Assets {
    using Address for address;
    using SafeTokenTransferFrom for IERC20;

    /* Errors */
    error InvalidToken();
    error InvalidAmount();
    error InvalidDestination();
    error Unsupported();

    // This library requires state which must be initialized in the gateway's storage.
    function initialize(uint256 _registerTokenFee, uint256 _sendTokenFee) external {
        AssetsStorage.Layout storage $ = AssetsStorage.layout();

        $.registerTokenFee = _registerTokenFee;
        $.sendTokenFee = _sendTokenFee;
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

    function sendTokenFee(ParaID assetHubParaID, ParaID destinationChain) external view returns (uint256) {
        AssetsStorage.Layout storage $ = AssetsStorage.layout();
        if (assetHubParaID == destinationChain) {
            return $.sendTokenFee;
        }
        // If the final destination chain is not AssetHub, then the fee needs to additionally
        // include the cost of executing an XCM on the final destination parachain.
        return 2 * $.sendTokenFee;
    }

    function sendToken(
        ParaID assetHubParaID,
        address assetHubAgent,
        address token,
        address sender,
        ParaID destinationChain,
        MultiAddress calldata destinationAddress,
        uint128 amount
    ) external returns (bytes memory payload, uint256 extraFee) {
        AssetsStorage.Layout storage $ = AssetsStorage.layout();

        _transferToAgent(assetHubAgent, token, sender, amount);

        if (destinationChain == assetHubParaID) {
            if (destinationAddress.isAddress32()) {
                payload = SubstrateTypes.SendTokenToAssetHubAddress32(token, destinationAddress.asAddress32(), amount);
            } else {
                // AssetHub does not support 20-byte account IDs
                revert Unsupported();
            }
            extraFee = $.sendTokenFee;
        } else {
            if (destinationAddress.isAddress32()) {
                payload = SubstrateTypes.SendTokenToAddress32(
                    token, destinationChain, destinationAddress.asAddress32(), amount
                );
            } else if (destinationAddress.isAddress20()) {
                payload = SubstrateTypes.SendTokenToAddress20(
                    token, destinationChain, destinationAddress.asAddress20(), amount
                );
            } else {
                revert Unsupported();
            }
            // If the final destination chain is not AssetHub, then the fee needs to additionally
            // include the cost of executing an XCM on the final destination parachain.
            extraFee = 2 * $.sendTokenFee;
        }

        emit IGateway.TokenSent(sender, token, destinationChain, destinationAddress, amount);
    }

    function registerTokenFee() external view returns (uint256) {
        AssetsStorage.Layout storage $ = AssetsStorage.layout();
        return $.registerTokenFee;
    }

    /// @dev Enqueues a create native token message to substrate.
    /// @param token The ERC20 token address.
    function registerToken(address token) external returns (bytes memory payload, uint256 extraFee) {
        AssetsStorage.Layout storage $ = AssetsStorage.layout();

        if (!token.isContract()) {
            revert InvalidToken();
        }

        payload = SubstrateTypes.RegisterToken(token);
        extraFee = $.registerTokenFee;

        emit IGateway.TokenRegistrationSent(token);
    }
}
