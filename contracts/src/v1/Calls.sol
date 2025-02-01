// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.28;

import {IERC20} from "../interfaces/IERC20.sol";
import {SafeNativeTransfer, SafeTokenTransferFrom} from "../utils/SafeTransfer.sol";

import {AssetsStorage, TokenInfo} from "../storage/AssetsStorage.sol";
import {CoreStorage} from "../storage/CoreStorage.sol";
import {PricingStorage} from "../storage/PricingStorage.sol";
import {SubstrateTypes} from "../SubstrateTypes.sol";
import {MultiAddress} from "../MultiAddress.sol";
import {Address} from "../utils/Address.sol";
import {AgentExecutor} from "../AgentExecutor.sol";
import {Agent} from "../Agent.sol";
import {Call} from "../utils/Call.sol";
import {Token} from "../Token.sol";
import {Functions} from "../Functions.sol";
import {
    TokenInfo,
    OperatingMode,
    ParaID,
    Channel,
    ChannelID,
    AgentExecuteCommand,
    Ticket,
    Costs
} from "./Types.sol";
import {IGatewayBase} from "../interfaces/IGatewayBase.sol";
import {IGatewayV1} from "./IGateway.sol";
import {UD60x18, ud60x18, convert} from "prb/math/src/UD60x18.sol";

/// @title Library for implementing Ethereum->Polkadot ERC20 transfers.
library CallsV1 {
    using Address for address;
    using SafeNativeTransfer for address payable;
    using SafeTokenTransferFrom for IERC20;
    using SafeNativeTransfer for address payable;

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
    error InvalidProof();
    error InvalidNonce();
    error NotEnoughGas();
    error FeePaymentToLow();
    error Unauthorized();
    error Disabled();
    error AgentAlreadyCreated();
    error ChannelAlreadyCreated();
    error ChannelDoesNotExist();
    error InvalidChannelUpdate();
    error AgentExecutionFailed(bytes returndata);
    error InvalidAgentExecutionPayload();
    error InvalidConstructorParams();
    error AlreadyInitialized();

    /*
    * External API
    */

    /// @dev Registers a token (only native tokens at this time)
    /// @param token The ERC20 token address.
    function registerToken(address token) external {
        if (!token.isContract()) {
            revert InvalidToken();
        }

        AssetsStorage.Layout storage $ = AssetsStorage.layout();

        // NOTE: Explicitly allow a token to be re-registered. This offers resiliency
        // in case a previous registration attempt of the same token failed on the remote side.
        // It means that registration can be retried.
        Functions.registerNativeToken(token);

        Ticket memory ticket = Ticket({
            dest: $.assetHubParaID,
            costs: _registerTokenCosts(),
            payload: SubstrateTypes.RegisterToken(token, $.assetHubCreateAssetFee),
            value: 0
        });

        emit IGatewayBase.TokenRegistrationSent(token);

        _submitOutbound(ticket);
    }

    function quoteRegisterTokenFee() external view returns (uint256) {
        return _calculateFee(_registerTokenCosts());
    }

    function sendToken(
        address token,
        address sender,
        ParaID destinationChain,
        MultiAddress calldata destinationAddress,
        uint128 destinationChainFee,
        uint128 amount
    ) external {
        AssetsStorage.Layout storage $ = AssetsStorage.layout();

        if (amount == 0) {
            revert InvalidAmount();
        }

        TokenInfo storage info = $.tokenRegistry[token];

        if (!info.isRegistered) {
            revert TokenNotRegistered();
        }

        if (info.isNative()) {
            _submitOutbound(
                _sendNativeTokenOrEther(
                    token,
                    sender,
                    destinationChain,
                    destinationAddress,
                    destinationChainFee,
                    amount
                )
            );
        } else {
            _submitOutbound(
                _sendForeignToken(
                    info.foreignID,
                    token,
                    sender,
                    destinationChain,
                    destinationAddress,
                    destinationChainFee,
                    amount
                )
            );
        }
    }

    function quoteSendTokenFee(address token, ParaID destinationChain, uint128 destinationChainFee)
        external
        view
        returns (uint256)
    {
        AssetsStorage.Layout storage $ = AssetsStorage.layout();
        TokenInfo storage info = $.tokenRegistry[token];
        if (!info.isRegistered) {
            revert TokenNotRegistered();
        }
        return _calculateFee(_sendTokenCosts(destinationChain, destinationChainFee));
    }

    function pricingParameters() external view returns (UD60x18, uint128) {
        PricingStorage.Layout storage pricing = PricingStorage.layout();
        return (pricing.exchangeRate, pricing.deliveryCost);
    }

    function channelNoncesOf(ChannelID channelID) external view returns (uint64, uint64) {
        Channel storage ch = Functions.ensureChannel(channelID);
        return (ch.inboundNonce, ch.outboundNonce);
    }

    function channelOperatingModeOf(ChannelID channelID) external view returns (OperatingMode) {
        Channel storage ch = Functions.ensureChannel(channelID);
        return ch.mode;
    }

    // @dev Get token address by tokenID
    function tokenAddressOf(bytes32 tokenID) external view returns (address) {
        AssetsStorage.Layout storage $ = AssetsStorage.layout();
        return $.tokenAddressOf[tokenID];
    }

    /*
    * Internal functions
    */

    // Convert foreign currency to native currency (WND/PAS/KSM/DOT -> ETH)
    function _convertToNative(UD60x18 exchangeRate, UD60x18 multiplier, UD60x18 amount)
        internal
        view
        returns (uint256)
    {
        AssetsStorage.Layout storage $ = AssetsStorage.layout();

        UD60x18 ethDecimals = convert(1e18);
        UD60x18 foreignDecimals = convert(10).pow(convert(uint256($.foreignTokenDecimals)));
        UD60x18 nativeAmount =
            multiplier.mul(amount).mul(exchangeRate).div(foreignDecimals).mul(ethDecimals);
        return convert(nativeAmount);
    }

    // Calculate the fee for accepting an outbound message
    function _calculateFee(Costs memory costs) internal view returns (uint256) {
        PricingStorage.Layout storage pricing = PricingStorage.layout();
        UD60x18 amount = convert(pricing.deliveryCost + costs.foreign);
        return costs.native + _convertToNative(pricing.exchangeRate, pricing.multiplier, amount);
    }

    /// @dev Outbound message can be disabled globally or on a per-channel basis.
    function _ensureOutboundMessagingEnabled(Channel storage ch) internal view {
        CoreStorage.Layout storage $ = CoreStorage.layout();
        if ($.mode != OperatingMode.Normal || ch.mode != OperatingMode.Normal) {
            revert Disabled();
        }
    }

    // Submit an outbound message to Polkadot, after taking fees
    function _submitOutbound(Ticket memory ticket) internal {
        ChannelID channelID = ticket.dest.into();
        Channel storage channel = Functions.ensureChannel(channelID);

        // Ensure outbound messaging is allowed
        _ensureOutboundMessagingEnabled(channel);

        // Destination fee always in DOT
        uint256 fee = _calculateFee(ticket.costs);

        // Ensure the user has provided enough ether for this message to be accepted.
        // This includes:
        // 1. The bridging fee, which is collected in this gateway contract
        // 2. The ether value being bridged over to Polkadot, which is locked into the AssetHub
        //    agent contract.
        uint256 totalRequiredEther = fee + ticket.value;
        if (msg.value < totalRequiredEther) {
            revert IGatewayBase.InsufficientEther();
        }
        if (ticket.value > 0) {
            payable(AssetsStorage.layout().assetHubAgent).safeNativeTransfer(ticket.value);
        }

        channel.outboundNonce = channel.outboundNonce + 1;

        // Reimburse excess fee payment
        uint256 excessFee = msg.value - totalRequiredEther;
        if (excessFee > Functions.dustThreshold()) {
            payable(msg.sender).safeNativeTransfer(excessFee);
        }

        // Generate a unique ID for this message
        bytes32 messageID = keccak256(abi.encodePacked(channelID, channel.outboundNonce));

        emit IGatewayV1.OutboundMessageAccepted(
            channelID, channel.outboundNonce, messageID, ticket.payload
        );
    }

    function isTokenRegistered(address token) external view returns (bool) {
        return AssetsStorage.layout().tokenRegistry[token].isRegistered;
    }

    function _sendTokenCosts(ParaID destinationChain, uint128 destinationChainFee)
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
            if (destinationChainFee > $.maxDestinationFee) {
                revert InvalidDestinationFee();
            }

            // If the final destination chain is not AssetHub, then the fee needs to additionally
            // include the cost of executing an XCM on the final destination parachain.
            costs.foreign = $.assetHubReserveTransferFee + destinationChainFee;
        }
        // We don't charge any extra fees beyond delivery costs
        costs.native = 0;
    }

    function _sendNativeTokenOrEther(
        address token,
        address sender,
        ParaID destinationChain,
        MultiAddress calldata destinationAddress,
        uint128 destinationChainFee,
        uint128 amount
    ) internal returns (Ticket memory ticket) {
        AssetsStorage.Layout storage $ = AssetsStorage.layout();

        if (token != address(0)) {
            // Lock ERC20
            Functions.transferToAgent($.assetHubAgent, token, sender, amount);
            ticket.value = 0;
        } else {
            // Track the ether to bridge to Polkadot. This will be handled
            // in `Gateway._submitOutbound`.
            ticket.value = amount;
        }

        ticket.dest = $.assetHubParaID;
        ticket.costs = _sendTokenCosts(destinationChain, destinationChainFee);

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

        emit IGatewayV1.TokenSent(token, sender, destinationChain, destinationAddress, amount);
    }

    // @dev Transfer Polkadot-native tokens back to Polkadot
    function _sendForeignToken(
        bytes32 foreignID,
        address token,
        address sender,
        ParaID destinationChain,
        MultiAddress calldata destinationAddress,
        uint128 destinationChainFee,
        uint128 amount
    ) internal returns (Ticket memory ticket) {
        AssetsStorage.Layout storage $ = AssetsStorage.layout();

        Token(token).burn(sender, amount);

        ticket.dest = $.assetHubParaID;
        ticket.costs = _sendTokenCosts(destinationChain, destinationChainFee);
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

        emit IGatewayV1.TokenSent(token, sender, destinationChain, destinationAddress, amount);
    }

    function _registerTokenCosts() internal view returns (Costs memory costs) {
        AssetsStorage.Layout storage $ = AssetsStorage.layout();

        // Cost of registering this asset on AssetHub
        costs.foreign = $.assetHubCreateAssetFee;

        // Extra fee to prevent spamming
        costs.native = $.registerTokenFee;
    }

    function _isTokenRegistered(address token) internal view returns (bool) {
        AssetsStorage.Layout storage $ = AssetsStorage.layout();
        return $.tokenRegistry[token].isRegistered;
    }
}
