// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.34;

import {IERC20} from "../interfaces/IERC20.sol";
import {SafeNativeTransfer, SafeTokenTransferFrom} from "../utils/SafeTransfer.sol";

import {AssetsStorage, TokenInfo} from "../storage/AssetsStorage.sol";
import {CoreStorage} from "../storage/CoreStorage.sol";
import {PricingStorage} from "../storage/PricingStorage.sol";
import {SubstrateTypes} from "../SubstrateTypes.sol";
import {MultiAddress} from "./MultiAddress.sol";
import {Address} from "../utils/Address.sol";
import {Token} from "../Token.sol";
import {Functions} from "../Functions.sol";
import {OperatingMode, ParaID, Channel, ChannelID, Ticket, Costs} from "./Types.sol";
import {IGatewayBase} from "../interfaces/IGatewayBase.sol";
import {IGatewayV1} from "./IGateway.sol";
import {UD60x18, convert} from "prb/math/src/UD60x18.sol";

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
    error ChannelDoesNotExist();
    error InvalidChannelUpdate();
    error AgentExecutionFailed(bytes returndata);
    error InvalidAgentExecutionPayload();
    error InvalidConstructorParams();
    error AlreadyInitialized();

    /*
    * External API
    */

    function sendToken(
        address token,
        address sender,
        ParaID destinationChain,
        MultiAddress calldata destinationAddress,
        uint128 destinationChainFee,
        uint128 amount
    ) external {
        revert Unsupported();
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
}
