// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.25;

import {IERC20} from "../interfaces/IERC20.sol";
import {WETH9} from "canonical-weth/WETH9.sol";

import {IGatewayBase} from "../interfaces/IGatewayBase.sol";
import {IGatewayV2} from "./IGateway.sol";

import {SafeNativeTransfer, SafeTokenTransfer} from "../utils/SafeTransfer.sol";

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
import {Upgrade} from "../Upgrade.sol";
import {Functions} from "../Functions.sol";
import {Constants} from "../Constants.sol";

import {Ticket, OperatingMode} from "./Types.sol";

import {UD60x18, ud60x18, convert} from "prb/math/src/UD60x18.sol";

/// @title Library for implementing Ethereum->Polkadot ERC20 transfers.
library CallsV2 {
    using Address for address;
    using SafeTokenTransfer for IERC20;
    using SafeNativeTransfer for address payable;

    uint8 public constant MAX_ASSETS = 8;

    // Send an XCM with arbitrary assets to Polkadot Asset Hub
    //
    // Params:
    //   * `xcm` (bytes): SCALE-encoded XCM message
    //   * `assets` (bytes[]): Array of asset specs.
    //
    // Supported asset specs:
    // * ERC20: abi.encode(0, tokenAddress, value)
    //
    // On Asset Hub, the assets will be received into the assets holding register.
    //
    // The `xcm` should contain the necessary instructions to:
    // 1. Pay XCM execution fees, either from assets in holding,
    //    or from the sovereign account of `msg.sender`.
    // 2. Handle the assets in holding, either depositing them into
    //    some account, or forwarding them to another destination.
    //
    function sendMessage(
        bytes calldata xcm,
        bytes[] calldata assets,
        bytes calldata claimer
    ) external {
        _sendMessage(msg.sender, xcm, assets, claimer);
    }

    // Register Ethereum-native token on AHP, using `xcmFeeAHP` of `msg.value`
    // to pay for execution on AHP
    function registerToken(address token, uint128 xcmFeeAHP) external {
        _registerToken(token, xcmFeeAHP, 0);
    }

    // Register Ethereum-native token on AHK, using `xcmFeeAHP` and `xcmFeeAHK`
    // of `msg.value` to pay for execution on AHP and AHK respectively.
    function registerTokenOnKusama(address token, uint128 xcmFeeAHP, uint128 xcmFeeAHK)
        external
    {
        _registerToken(token, xcmFeeAHP, xcmFeeAHK);
    }

    /*
    * Internal functions
    */

    function _sendMessage(
        address origin,
        bytes memory xcm,
        bytes[] memory assets,
        bytes memory claimer
    ) internal {
        if (assets.length > MAX_ASSETS) {
            revert IGatewayBase.TooManyAssets();
        }

        bytes[] memory encodedAssets = new bytes[](assets.length);
        uint128 etherValue = 0;
        uint128 totalEtherValue = 0;

        for (uint256 i = 0; i < assets.length; i++) {
            (encodedAssets[i], etherValue) = _handleAsset(assets[i]);
            totalEtherValue += etherValue;
        }

        if (totalEtherValue > msg.value) {
            revert IGatewayV2.InvalidEtherValue();
        }

        Ticket memory ticket = Ticket({
            origin: origin,
            assets: encodedAssets,
            xcm: xcm,
            claimer: claimer,
            reward: msg.value - totalEtherValue
        });
        _submitOutbound(ticket);
    }

    function _registerToken(address token, uint128 xcmFeeAHP, uint128 xcmFeeAHK)
        internal
    {
        // Build XCM for token registration on AHP and possibly AHK
        // The XCM includes the necessary `PaysFee` instructions that:
        // 1. Buy `xcmFeeAHP` worth of execution on AHP
        // 2. Buy `xcmFeeAHK` worth of execution on AHK
        bytes memory xcm;
        if (xcmFeeAHK > 0) {
            // Build XCM that is executed on AHP and forwards another message to AHK
            xcm = bytes.concat(hex"deadbeef", abi.encodePacked(token), hex"deadbeef");
        } else {
            // Build XCM that executes on AHP only
            xcm = bytes.concat(hex"deadbeef", abi.encodePacked(token), hex"deadbeef");
        }

        uint256 xcmFee = xcmFeeAHP + xcmFeeAHK;

        // Lock up the total xcm fee
        if (xcmFee > msg.value) {
            revert IGatewayV2.InvalidFee();
        }

        bytes[] memory assets = new bytes[](1);
        assets[0] = abi.encode(0, xcmFee);

        _sendMessage(address(this), xcm, assets, "");
    }

    // Submit an outbound message to Polkadot, after taking fees
    function _submitOutbound(Ticket memory ticket) internal {
        CoreStorage.Layout storage $ = CoreStorage.layout();

        // Ensure outbound messaging is allowed
        _ensureOutboundMessagingEnabled();

        // Lock up the relayer reward
        _lockEther(ticket.reward);

        $.outboundNonce = $.outboundNonce + 1;

        bytes memory payload = SubstrateTypes.encodePayloadV2(
            ticket.origin, ticket.assets, ticket.xcm, ticket.claimer
        );

        emit IGatewayV2.OutboundMessageAccepted($.outboundNonce, ticket.reward, payload);
    }

    // Lock wrapped ether into the AssetHub Agent
    function _lockEther(uint256 value) internal {
        address weth = Functions.weth();
        address assetHubAgent = Functions.ensureAgent(Constants.ASSET_HUB_AGENT_ID);
        WETH9(payable(weth)).deposit{value: value}();
        IERC20(weth).safeTransfer(assetHubAgent, value);
    }

    /// @dev Outbound message can be disabled globally or on a per-channel basis.
    function _ensureOutboundMessagingEnabled() internal view {
        CoreStorage.Layout storage $ = CoreStorage.layout();
        if ($.mode != OperatingMode.Normal) {
            revert IGatewayBase.Disabled();
        }
    }

    function _handleAsset(bytes memory asset) internal returns (bytes memory, uint128) {
        uint8 assetKind;
        assembly {
            assetKind := byte(31, mload(add(asset, 32)))
        }
        if (assetKind == 0) {
            // Ether: abi.encode(0, value)
            (, uint128 amount) = abi.decode(asset, (uint8, uint128));
            return _handleAssetEther(amount);
        } else if (assetKind == 1) {
            // ERC20: abi.encode(1, tokenAddress, value)
            (, address token, uint128 amount) =
                abi.decode(asset, (uint8, address, uint128));
            return _handleAssetERC20(token, amount);
        } else {
            revert IGatewayV2.InvalidAsset();
        }
    }

    function _handleAssetEther(uint128 amount)
        internal
        returns (bytes memory, uint128)
    {
        _lockEther(amount);
        return (
            SubstrateTypes.encodeTransferNativeTokenERC20(Functions.weth(), amount),
            amount
        );
    }

    function _handleAssetERC20(address token, uint128 amount)
        internal
        returns (bytes memory, uint128)
    {
        AssetsStorage.Layout storage $ = AssetsStorage.layout();
        TokenInfo storage info = $.tokenRegistry[token];

        if (!info.isRegistered) {
            revert IGatewayBase.TokenNotRegistered();
        }

        if (info.foreignID == bytes32(0)) {
            Functions.transferToAgent($.assetHubAgent, token, msg.sender, amount);
            return (SubstrateTypes.encodeTransferNativeTokenERC20(token, amount), 0);
        } else {
            Token(token).burn(msg.sender, amount);
            return (
                SubstrateTypes.encodeTransferForeignTokenERC20(info.foreignID, amount), 0
            );
        }
    }
}
