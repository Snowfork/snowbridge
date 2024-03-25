// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.23;

import {Assets} from "./Assets.sol";
import {Agent} from "./Agent.sol";
import {Channel, ChannelID, OperatingMode, ParaID, MultiAddress, Ticket, Costs, TokenInfo} from "./Types.sol";
import {IGatewayOutbound} from "./interfaces/IGatewayOutbound.sol";
import {SafeNativeTransfer} from "./utils/SafeTransfer.sol";

import {CoreStorage} from "./storage/CoreStorage.sol";
import {PricingStorage} from "./storage/PricingStorage.sol";

import {UD60x18, ud60x18, convert} from "prb/math/src/UD60x18.sol";

contract GatewayOutbound is IGatewayOutbound {
    using SafeNativeTransfer for address payable;

    error Disabled();
    error FeePaymentToLow();
    error ChannelNotExist();

    /**
     * Assets
     */
    function isTokenRegistered(address token) external view returns (bool) {
        return Assets.isTokenRegistered(token);
    }

    // Total fee for registering a token
    function quoteRegisterTokenFee() external view returns (uint256) {
        return _calculateFee(Assets.registerTokenCosts());
    }

    // Register an Ethereum-native token in the gateway and on AssetHub
    function registerToken(address token) external payable {
        _submitOutbound(Assets.registerToken(token));
    }

    // Total fee for sending a token
    function quoteSendTokenFee(address token, ParaID destinationChain, uint128 destinationFee)
        external
        view
        returns (uint256)
    {
        return _calculateFee(Assets.sendTokenCosts(token, destinationChain, destinationFee));
    }

    // Transfer ERC20 tokens to a Polkadot parachain
    function sendToken(
        address token,
        ParaID destinationChain,
        MultiAddress calldata destinationAddress,
        uint128 destinationFee,
        uint128 amount
    ) external payable {
        _submitOutbound(
            Assets.sendToken(token, msg.sender, destinationChain, destinationAddress, destinationFee, amount)
        );
    }

    // Transfer polkadot native tokens back
    function transferToken(
        address token,
        ParaID destinationChain,
        MultiAddress calldata destinationAddress,
        uint128 destinationFee,
        uint128 amount
    ) external payable {
        address executor = CoreStorage.layout().agentExecutor;
        _submitOutbound(
            Assets.transferToken(
                executor, token, msg.sender, destinationChain, destinationAddress, destinationFee, amount
            )
        );
    }

    function getTokenInfo(bytes32 tokenID) external view returns (TokenInfo memory) {
        return Assets.getTokenInfo(tokenID);
    }

    // Convert foreign currency to native currency (ROC/KSM/DOT -> ETH)
    function _convertToNative(UD60x18 exchangeRate, UD60x18 multiplier, UD60x18 amount)
        internal
        view
        returns (uint256)
    {
        uint8 foreignTokenDecimals = CoreStorage.layout().foreignTokenDecimals;
        UD60x18 ethDecimals = convert(1e18);
        UD60x18 foreignDecimals = convert(10).pow(convert(uint256(foreignTokenDecimals)));
        UD60x18 nativeAmount = multiplier.mul(amount).mul(exchangeRate).div(foreignDecimals).mul(ethDecimals);
        return convert(nativeAmount);
    }

    // Calculate the fee for accepting an outbound message
    function _calculateFee(Costs memory costs) internal view returns (uint256) {
        PricingStorage.Layout storage pricing = PricingStorage.layout();
        UD60x18 amount = convert(pricing.deliveryCost + costs.foreign);
        return costs.native + _convertToNative(pricing.exchangeRate, pricing.multiplier, amount);
    }

    // Submit an outbound message to Polkadot, after taking fees
    function _submitOutbound(Ticket memory ticket) internal {
        ChannelID channelID = ticket.dest.into();
        Channel storage channel = CoreStorage.layout().channels[channelID];
        if (channel.agent == address(0)) {
            revert ChannelNotExist();
        }
        // Ensure outbound messaging is allowed
        _ensureOutboundMessagingEnabled(channel);

        uint256 fee = _calculateFee(ticket.costs);

        // Ensure the user has enough funds for this message to be accepted
        if (msg.value < fee) {
            revert FeePaymentToLow();
        }

        channel.outboundNonce = channel.outboundNonce + 1;

        // Deposit total fee into agent's contract
        payable(channel.agent).safeNativeTransfer(fee);

        // Reimburse excess fee payment
        if (msg.value > fee) {
            payable(msg.sender).safeNativeTransfer(msg.value - fee);
        }

        // Generate a unique ID for this message
        bytes32 messageID = keccak256(abi.encodePacked(channelID, channel.outboundNonce));

        emit IGatewayOutbound.OutboundMessageAccepted(channelID, channel.outboundNonce, messageID, ticket.payload);
    }

    /// @dev Outbound message can be disabled globally or on a per-channel basis.
    function _ensureOutboundMessagingEnabled(Channel storage ch) internal view {
        CoreStorage.Layout storage $ = CoreStorage.layout();
        if ($.mode != OperatingMode.Normal || ch.mode != OperatingMode.Normal) {
            revert Disabled();
        }
    }
}
