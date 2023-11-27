// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.22;

import {IERC20} from "./interfaces/IERC20.sol";
import {IGateway} from "./interfaces/IGateway.sol";

import {SafeTokenTransferFrom} from "./utils/SafeTransfer.sol";

import {AssetsStorage} from "./storage/AssetsStorage.sol";
import {SubstrateTypes} from "./SubstrateTypes.sol";
import {ParaID, MultiAddress, Ticket, Costs} from "./Types.sol";
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

    function sendTokenCosts(ParaID assetHubParaID, ParaID destinationChain, uint128 destinationChainFee)
        external
        view
        returns (Costs memory costs)
    {
        return _sendTokenCosts(assetHubParaID, destinationChain, destinationChainFee);
    }

    function _sendTokenCosts(ParaID assetHubParaID, ParaID destinationChain, uint128 destinationChainFee)
        internal
        view
        returns (Costs memory costs)
    {
        AssetsStorage.Layout storage $ = AssetsStorage.layout();
        if (assetHubParaID == destinationChain) {
            costs.foreign = $.assetHubReserveTransferFee;
        } else {
            // If the final destination chain is not AssetHub, then the fee needs to additionally
            // include the cost of executing an XCM on the final destination parachain.
            costs.foreign = $.assetHubReserveTransferFee + destinationChainFee;
        }
        costs.native = 0;
    }

    function sendToken(
        ParaID assetHubParaID,
        address assetHubAgent,
        address token,
        address sender,
        ParaID destinationChain,
        MultiAddress calldata destinationAddress,
        uint128 destinationChainFee,
        uint128 amount
    ) external returns (Ticket memory ticket) {
        AssetsStorage.Layout storage $ = AssetsStorage.layout();

        _transferToAgent(assetHubAgent, token, sender, amount);
        ticket.cost = _sendTokenCosts(assetHubParaID, destinationChain, destinationChainFee);

        if (destinationChain == assetHubParaID) {
            if (destinationAddress.isAddress32()) {
                ticket.payload = SubstrateTypes.SendTokenToAssetHubAddress32(
                    token, destinationAddress.asAddress32(), $.assetHubReserveTransferFee, amount
                );
            } else {
                // AssetHub does not support 20-byte account IDs
                revert Unsupported();
            }
        } else {
            if (destinationAddress.isAddress32()) {
                ticket.payload = SubstrateTypes.SendTokenToAddress32(
                    token,
                    destinationChain,
                    destinationAddress.asAddress32(),
                    $.assetHubReserveTransferFee,
                    destinationChainFee,
                    amount
                );
            } else if (destinationAddress.isAddress20()) {
                ticket.payload = SubstrateTypes.SendTokenToAddress20(
                    token,
                    destinationChain,
                    destinationAddress.asAddress20(),
                    $.assetHubReserveTransferFee,
                    destinationChainFee,
                    amount
                );
            } else {
                revert Unsupported();
            }
        }
        emit IGateway.TokenSent(sender, token, destinationChain, destinationAddress, amount);
    }


    function registerTokenCosts() external view returns (Costs memory costs) {
        return _registerTokenCosts();
    }

    function _registerTokenCosts() internal view returns (Costs memory costs) {
        AssetsStorage.Layout storage $ = AssetsStorage.layout();
        costs.foreign = $.assetHubCreateAssetFee;
        costs.native = $.registerTokenFee;
    }

    /// @dev Enqueues a create native token message to substrate.
    /// @param token The ERC20 token address.
    function registerToken(address token) external returns (Ticket memory ticket) {
        AssetsStorage.Layout storage $ = AssetsStorage.layout();

        if (!token.isContract()) {
            revert InvalidToken();
        }

        ticket.cost = _registerTokenCosts();
        ticket.payload = SubstrateTypes.RegisterToken(token);

        emit IGateway.TokenRegistrationSent(token);
    }
}
