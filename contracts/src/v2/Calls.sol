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

    // Refer to `IGateway.sendMessage` for documentation
    function sendMessage(
        bytes calldata xcm,
        bytes[] calldata assets,
        bytes calldata claimer,
        uint128 executionFee,
        uint128 relayerFee
    ) external {
        _sendMessage(msg.sender, xcm, assets, claimer, executionFee, relayerFee);
    }

    // Refer to `IGateway.registerToken` for documentation
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
            assets: preparedAssets,
            xcm: xcm,
            claimer: claimer,
            value: uint128(msg.value) - executionFee - relayerFee,
            executionFee: executionFee,
            relayerFee: relayerFee
        });

        emit IGatewayV2.OutboundMessageAccepted($.outboundNonce, payload);
    }

    /// @dev Outbound message can be disabled globally or on a per-channel basis.
    function _ensureOutboundMessagingEnabled() internal view {
        CoreStorage.Layout storage $ = CoreStorage.layout();
        require($.mode == OperatingMode.Normal, IGatewayBase.Disabled());
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
        TokenInfo storage tokenInfo = $.tokenRegistry[token];

        require(tokenInfo.exists(), IGatewayBase.TokenNotRegistered());
        require(amount > 0, IGatewayBase.InvalidAmount());

        if (tokenInfo.isNative()) {
            Functions.transferToAgent($.assetHubAgent, token, msg.sender, amount);
            return makeNativeAsset(token, amount);
        } else if (tokenInfo.isForeign()) {
            Token(token).burn(msg.sender, amount);
            return makeForeignAsset(tokenInfo.foreignID, amount);
        } else {
            revert IGatewayV2.ShouldNotReachHere();
        }
    }

    function outboundNonce() external view returns (uint64) {
        return CoreStorage.layout().outboundNonce;
    }
}
