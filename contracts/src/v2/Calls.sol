// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.28;

import {IERC20} from "../interfaces/IERC20.sol";
import {WETH9} from "canonical-weth/WETH9.sol";

import {IGatewayBase} from "../interfaces/IGatewayBase.sol";
import {IGatewayV2} from "./IGateway.sol";

import {SafeNativeTransfer, SafeTokenTransfer} from "../utils/SafeTransfer.sol";

import {AssetsStorage, TokenInfo, TokenInfoFunctions} from "../storage/AssetsStorage.sol";
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

import {
    Payload, OperatingMode, Asset, makeNativeAsset, makeForeignAsset, Network
} from "./Types.sol";

import {UD60x18, ud60x18, convert} from "prb/math/src/UD60x18.sol";

/// @title Library for implementing Ethereum->Polkadot ERC20 transfers.
library CallsV2 {
    using Address for address;
    using SafeTokenTransfer for IERC20;
    using SafeNativeTransfer for address payable;
    using TokenInfoFunctions for TokenInfo;

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
        bytes calldata claimer,
        uint128 executionFee,
        uint128 relayerFee
    ) external {
        _sendMessage(msg.sender, xcm, assets, claimer, executionFee, relayerFee);
    }

    // Register Ethereum-native token on AHP, using `xcmFeeAHP` of `msg.value`
    // to pay for execution on AHP
    function registerToken(
        address token,
        Network network,
        uint128 executionFee,
        uint128 relayerFee
    ) internal {
        // Build XCM for token registration on AHP and possibly AHK
        bytes memory xcm;

        require(msg.value <= type(uint128).max, IGatewayV2.ExceededMaximumValue());
        require(msg.value >= executionFee + relayerFee, IGatewayV2.InsufficientValue());
        uint128 etherValue = uint128(msg.value) - executionFee - relayerFee;

        if (network == Network.Polkadot) {
            // Build XCM that executes on AHP only
            xcm = bytes.concat(
                hex"deadbeef", abi.encodePacked(token), hex"deadbeef", abi.encodePacked(etherValue)
            );
        } else if (network == Network.Kusama) {
            // Build XCM that is executed on AHP and forwards another message to AHK
            xcm = bytes.concat(
                hex"deadbeef", abi.encodePacked(token), hex"deadbeef", abi.encodePacked(etherValue)
            );
            xcm = bytes.concat(hex"deadbeef", abi.encodePacked(token), hex"deadbeef");
        } else {
            revert IGatewayV2.ShouldNotReachHere();
        }

        Functions.registerNativeToken(token);

        _sendMessage(address(this), xcm, new bytes[](0), "", executionFee, relayerFee);
    }

    /*
    * Internal functions
    */

    function _sendMessage(
        address origin,
        bytes memory xcm,
        bytes[] memory assets,
        bytes memory claimer,
        uint128 executionFee,
        uint128 relayerFee
    ) internal {
        // Ensure outbound messaging is allowed
        _ensureOutboundMessagingEnabled();

        require(msg.value <= type(uint128).max, IGatewayV2.ExceededMaximumValue());
        require(msg.value >= executionFee + relayerFee, IGatewayV2.InsufficientValue());
        address assetHubAgent = Functions.ensureAgent(Constants.ASSET_HUB_AGENT_ID);
        payable(assetHubAgent).safeNativeTransfer(msg.value);

        require(assets.length <= MAX_ASSETS, IGatewayBase.TooManyAssets());
        Asset[] memory preparedAssets = new Asset[](assets.length);
        for (uint256 i = 0; i < assets.length; i++) {
            preparedAssets[i] = _handleAsset(assets[i]);
        }

        CoreStorage.Layout storage $ = CoreStorage.layout();
        $.outboundNonce = $.outboundNonce + 1;

        Payload memory payload = Payload({
            origin: origin,
            value: uint128(msg.value),
            assets: preparedAssets,
            xcm: xcm,
            claimer: claimer,
            executionFee: executionFee,
            relayerFee: relayerFee
        });

        emit IGatewayV2.OutboundMessageAccepted($.outboundNonce, payload);
    }

    /// @dev Outbound message can be disabled globally or on a per-channel basis.
    function _ensureOutboundMessagingEnabled() internal view {
        CoreStorage.Layout storage $ = CoreStorage.layout();
        if ($.mode != OperatingMode.Normal) {
            revert IGatewayBase.Disabled();
        }
    }

    function _handleAsset(bytes memory asset) internal returns (Asset memory) {
        uint8 assetKind;
        assembly {
            assetKind := byte(31, mload(add(asset, 32)))
        }
        if (assetKind == 0) {
            // ERC20: abi.encode(0, tokenAddress, value)
            (, address token, uint128 amount) = abi.decode(asset, (uint8, address, uint128));
            return _handleAssetERC20(token, amount);
        } else {
            revert IGatewayV2.InvalidAsset();
        }
    }

    function _handleAssetERC20(address token, uint128 amount) internal returns (Asset memory) {
        AssetsStorage.Layout storage $ = AssetsStorage.layout();
        TokenInfo storage info = $.tokenRegistry[token];

        if (!info.exists()) {
            revert IGatewayBase.TokenNotRegistered();
        }

        if (info.isNative()) {
            Functions.transferToAgent($.assetHubAgent, token, msg.sender, amount);
            return makeNativeAsset(token, amount);
        } else if (info.isForeign()) {
            Token(token).burn(msg.sender, amount);
            return makeForeignAsset(info.foreignID, amount);
        } else {
            revert IGatewayV2.ShouldNotReachHere();
        }
    }
}
