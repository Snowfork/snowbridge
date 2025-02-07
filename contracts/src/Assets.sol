// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.28;

import {IERC20} from "./interfaces/IERC20.sol";
import {IGateway} from "./interfaces/IGateway.sol";

import {SafeTokenTransferFrom} from "./utils/SafeTransfer.sol";

import {AssetsStorage, TokenInfo} from "./storage/AssetsStorage.sol";
import {CoreStorage} from "./storage/CoreStorage.sol";

import {SubstrateTypes} from "./SubstrateTypes.sol";
import {ChannelID, ParaID, MultiAddress, Ticket, Costs} from "./Types.sol";
import {Address} from "./utils/Address.sol";
import {SafeNativeTransfer} from "./utils/SafeTransfer.sol";
import {AgentExecutor} from "./AgentExecutor.sol";
import {Agent} from "./Agent.sol";
import {Call} from "./utils/Call.sol";
import {Token} from "./Token.sol";

/// @title Library for implementing Ethereum->Polkadot ERC20 transfers.
library Assets {
    using Address for address;
    using SafeNativeTransfer for address payable;
    using SafeTokenTransferFrom for IERC20;

    /* Errors */
    error InvalidToken();
    error InvalidAmount();
    error InvalidDestination();
    error TokenNotRegistered();
    error Unsupported();
    error InvalidDestinationFee();
    error AgentDoesNotExist();
    error TokenAlreadyRegistered();
    error TokenMintFailed();
    error TokenTransferFailed();

    function isTokenRegistered(address token) external view returns (bool) {
        return AssetsStorage.layout().tokenRegistry[token].isRegistered;
    }

    /// @dev transfer tokens from the sender to the specified agent
    function _transferToAgent(address agent, address token, address sender, uint128 amount) internal {
        if (!token.isContract()) {
            revert InvalidToken();
        }

        if (amount == 0) {
            revert InvalidAmount();
        }

        IERC20(token).safeTransferFrom(sender, agent, amount);
    }

    function sendTokenCosts(
        address token,
        ParaID destinationChain,
        uint128 destinationChainFee,
        uint128 maxDestinationChainFee
    ) external view returns (Costs memory costs) {
        AssetsStorage.Layout storage $ = AssetsStorage.layout();
        TokenInfo storage info = $.tokenRegistry[token];
        if (!info.isRegistered) {
            revert TokenNotRegistered();
        }

        return _sendTokenCosts(destinationChain, destinationChainFee, maxDestinationChainFee);
    }

    function _sendTokenCosts(ParaID destinationChain, uint128 destinationChainFee, uint128 maxDestinationChainFee)
        internal
        view
        returns (Costs memory costs)
    {
        AssetsStorage.Layout storage $ = AssetsStorage.layout();
        if ($.assetHubParaID == destinationChain) {
            costs.foreign = $.assetHubReserveTransferFee;
        } else {
            // Reduce the ability for users to perform arbitrage by exploiting a
            // favourable exchange rate. For example supplying Ether
            // and gaining a more valuable amount of DOT on the destination chain.
            //
            // Also prevents users from mistakenly sending more fees than would be required
            // which has negative effects like draining AssetHub's sovereign account.
            //
            // For safety, `maxDestinationChainFee` should be less valuable
            // than the gas cost to send tokens.
            if (destinationChainFee > maxDestinationChainFee) {
                revert InvalidDestinationFee();
            }

            // If the final destination chain is not AssetHub, then the fee needs to additionally
            // include the cost of executing an XCM on the final destination parachain.
            costs.foreign = $.assetHubReserveTransferFee + destinationChainFee;
        }
        // We don't charge any extra fees beyond delivery costs
        costs.native = 0;
    }

    function sendToken(
        address token,
        address sender,
        ParaID destinationChain,
        MultiAddress calldata destinationAddress,
        uint128 destinationChainFee,
        uint128 maxDestinationChainFee,
        uint128 amount
    ) external returns (Ticket memory ticket) {
        AssetsStorage.Layout storage $ = AssetsStorage.layout();

        if (amount == 0) {
            revert InvalidAmount();
        }

        TokenInfo storage info = $.tokenRegistry[token];

        if (!info.isRegistered) {
            revert TokenNotRegistered();
        }

        if (info.isNativeToken()) {
            return _sendNativeTokenOrEther(
                token, sender, destinationChain, destinationAddress, destinationChainFee, maxDestinationChainFee, amount
            );
        } else {
            return _sendForeignToken(
                info.foreignID,
                token,
                sender,
                destinationChain,
                destinationAddress,
                destinationChainFee,
                maxDestinationChainFee,
                amount
            );
        }
    }

    // @dev Transfer ERC20(Ethereum-native) tokens to Polkadot
    function _sendNativeTokenOrEther(
        address token,
        address sender,
        ParaID destinationChain,
        MultiAddress calldata destinationAddress,
        uint128 destinationChainFee,
        uint128 maxDestinationChainFee,
        uint128 amount
    ) internal returns (Ticket memory ticket) {
        AssetsStorage.Layout storage $ = AssetsStorage.layout();

        if (token != address(0)) {
            // Lock ERC20
            _transferToAgent($.assetHubAgent, token, sender, amount);
            ticket.value = 0;
        } else {
            // Track the ether to bridge to Polkadot. This will be handled
            // in `Gateway._submitOutbound`.
            ticket.value = amount;
        }

        ticket.dest = $.assetHubParaID;
        ticket.costs = _sendTokenCosts(destinationChain, destinationChainFee, maxDestinationChainFee);

        // Construct a message payload
        if (destinationChain == $.assetHubParaID) {
            // The funds will be minted into the receiver's account on AssetHub
            if (destinationAddress.isAddress32()) {
                // The receiver has a 32-byte account ID
                ticket.payload = SubstrateTypes.SendTokenToAssetHubAddress32(
                    token, destinationAddress.asAddress32(), $.assetHubReserveTransferFee, amount
                );
            } else {
                // AssetHub does not support 20-byte account IDs
                revert Unsupported();
            }
        } else {
            if (destinationChainFee == 0) {
                revert InvalidDestinationFee();
            }
            // The funds will be minted into sovereign account of the destination parachain on AssetHub,
            // and then reserve-transferred to the receiver's account on the destination parachain.
            if (destinationAddress.isAddress32()) {
                // The receiver has a 32-byte account ID
                ticket.payload = SubstrateTypes.SendTokenToAddress32(
                    token,
                    destinationChain,
                    destinationAddress.asAddress32(),
                    $.assetHubReserveTransferFee,
                    destinationChainFee,
                    amount
                );
            } else if (destinationAddress.isAddress20()) {
                // The receiver has a 20-byte account ID
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

        emit IGateway.TokenSent(token, sender, destinationChain, destinationAddress, amount);
    }

    // @dev Transfer Polkadot-native tokens back to Polkadot
    function _sendForeignToken(
        bytes32 foreignID,
        address token,
        address sender,
        ParaID destinationChain,
        MultiAddress calldata destinationAddress,
        uint128 destinationChainFee,
        uint128 maxDestinationChainFee,
        uint128 amount
    ) internal returns (Ticket memory ticket) {
        AssetsStorage.Layout storage $ = AssetsStorage.layout();

        Token(token).burn(sender, amount);

        ticket.dest = $.assetHubParaID;
        ticket.costs = _sendTokenCosts(destinationChain, destinationChainFee, maxDestinationChainFee);
        ticket.value = 0;

        // Construct a message payload
        if (destinationChain == $.assetHubParaID && destinationAddress.isAddress32()) {
            // The funds will be minted into the receiver's account on AssetHub
            // The receiver has a 32-byte account ID
            ticket.payload = SubstrateTypes.SendForeignTokenToAssetHubAddress32(
                foreignID, destinationAddress.asAddress32(), $.assetHubReserveTransferFee, amount
            );
        } else {
            revert Unsupported();
        }

        emit IGateway.TokenSent(token, sender, destinationChain, destinationAddress, amount);
    }

    function registerTokenCosts() external view returns (Costs memory costs) {
        return _registerTokenCosts();
    }

    function _registerTokenCosts() internal view returns (Costs memory costs) {
        AssetsStorage.Layout storage $ = AssetsStorage.layout();

        // Cost of registering this asset on AssetHub
        costs.foreign = $.assetHubCreateAssetFee;

        // Extra fee to prevent spamming
        costs.native = $.registerTokenFee;
    }

    /// @dev Registers a token (only native tokens at this time)
    /// @param token The ERC20 token address.
    function registerToken(address token) external returns (Ticket memory ticket) {
        if (!token.isContract()) {
            revert InvalidToken();
        }

        AssetsStorage.Layout storage $ = AssetsStorage.layout();

        // NOTE: Explicitly allow a token to be re-registered. This offers resiliency
        // in case a previous registration attempt of the same token failed on the remote side.
        // It means that registration can be retried.
        // But register a PNA here is not allowed
        TokenInfo storage info = $.tokenRegistry[token];
        if (info.foreignID != bytes32(0)) {
            revert TokenAlreadyRegistered();
        }
        info.isRegistered = true;

        ticket.dest = $.assetHubParaID;
        ticket.costs = _registerTokenCosts();
        ticket.payload = SubstrateTypes.RegisterToken(token, $.assetHubCreateAssetFee);
        ticket.value = 0;

        emit IGateway.TokenRegistrationSent(token);
    }

    // @dev Register a new fungible Polkadot token for an agent
    function registerForeignToken(bytes32 foreignTokenID, string memory name, string memory symbol, uint8 decimals)
        external
    {
        AssetsStorage.Layout storage $ = AssetsStorage.layout();
        if ($.tokenAddressOf[foreignTokenID] != address(0)) {
            revert TokenAlreadyRegistered();
        }
        Token token = new Token(name, symbol, decimals);
        TokenInfo memory info = TokenInfo({isRegistered: true, foreignID: foreignTokenID});

        $.tokenAddressOf[foreignTokenID] = address(token);
        $.tokenRegistry[address(token)] = info;

        emit IGateway.ForeignTokenRegistered(foreignTokenID, address(token));
    }

    // @dev Mint foreign token from Polkadot
    function mintForeignToken(ChannelID channelID, bytes32 foreignTokenID, address recipient, uint256 amount)
        external
    {
        AssetsStorage.Layout storage $ = AssetsStorage.layout();
        if (channelID != $.assetHubParaID.into()) {
            revert TokenMintFailed();
        }
        address token = _ensureTokenAddressOf(foreignTokenID);
        Token(token).mint(recipient, amount);
    }

    // @dev Transfer ERC20 to `recipient`
    function transferNativeToken(address executor, address agent, address token, address recipient, uint128 amount)
        external
    {
        bytes memory call;
        if (token != address(0)) {
            // ERC20
            call = abi.encodeCall(AgentExecutor.transferToken, (token, recipient, amount));
        } else {
            // Native ETH
            call = abi.encodeCall(AgentExecutor.transferEther, (payable(recipient), amount));
        }
        (bool success,) = Agent(payable(agent)).invoke(executor, call);
        if (!success) {
            revert TokenTransferFailed();
        }
    }

    // @dev Get token address by tokenID
    function tokenAddressOf(bytes32 tokenID) external view returns (address) {
        AssetsStorage.Layout storage $ = AssetsStorage.layout();
        return $.tokenAddressOf[tokenID];
    }

    // @dev Get token address by tokenID
    function _ensureTokenAddressOf(bytes32 tokenID) internal view returns (address) {
        AssetsStorage.Layout storage $ = AssetsStorage.layout();
        if ($.tokenAddressOf[tokenID] == address(0)) {
            revert TokenNotRegistered();
        }
        return $.tokenAddressOf[tokenID];
    }
}
