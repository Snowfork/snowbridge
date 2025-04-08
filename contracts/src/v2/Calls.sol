// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.28;

import {IERC20} from "../interfaces/IERC20.sol";

import {IGatewayBase} from "../interfaces/IGatewayBase.sol";
import {IGatewayV2} from "./IGateway.sol";

import {SafeNativeTransfer, SafeTokenTransfer} from "../utils/SafeTransfer.sol";

import {AssetsStorage, TokenInfo} from "../storage/AssetsStorage.sol";
import {CoreStorage} from "../storage/CoreStorage.sol";
import {PricingStorage} from "../storage/PricingStorage.sol";
import {SubstrateTypes} from "../SubstrateTypes.sol";
import {Address} from "../utils/Address.sol";
import {AgentExecutor} from "../AgentExecutor.sol";
import {Agent} from "../Agent.sol";
import {Call} from "../utils/Call.sol";
import {Token} from "../Token.sol";
import {Upgrade} from "../Upgrade.sol";
import {Functions} from "../Functions.sol";
import {Constants} from "../Constants.sol";

import {
    Payload,
    OperatingMode,
    Asset,
    makeNativeAsset,
    makeForeignAsset,
    Network,
    Xcm,
    makeRawXCM,
    makeCreateAssetXCM
} from "./Types.sol";

import {UD60x18, ud60x18, convert} from "prb/math/src/UD60x18.sol";

/// @title Library for implementing Ethereum->Polkadot ERC20 transfers.
library CallsV2 {
    using Address for address;
    using SafeTokenTransfer for IERC20;
    using SafeNativeTransfer for address payable;

    uint8 public constant MAX_ASSETS = 8;

    // Refer to `IGateway.v2_createAgent` for documentation
    function createAgent(bytes32 id) external {
        CoreStorage.Layout storage core = CoreStorage.layout();
        address agent = core.agents[id];
        if (agent == address(0)) {
            agent = address(new Agent(id));
            core.agents[id] = agent;
            emit IGatewayV2.AgentCreated(id, agent);
        } else {
            revert IGatewayV2.AgentAlreadyExists();
        }
    }

    // Refer to `IGateway.v2_sendMessage` for documentation
    function sendMessage(
        bytes calldata xcm,
        bytes[] calldata assets,
        bytes calldata claimer,
        uint128 executionFee,
        uint128 relayerFee
    ) external {
        _sendMessage(msg.sender, makeRawXCM(xcm), assets, claimer, executionFee, relayerFee);
    }

    // Refer to `IGateway.v2_registerToken` for documentation
    function registerToken(
        address token,
        Network network,
        uint128 executionFee,
        uint128 relayerFee
    ) internal {
        require(msg.value <= type(uint128).max, IGatewayV2.ExceededMaximumValue());
        require(msg.value >= executionFee + relayerFee, IGatewayV2.InsufficientValue());

        Xcm memory xcm = makeCreateAssetXCM(token, network);

        Functions.registerNativeToken(token);

        _sendMessage(address(this), xcm, new bytes[](0), "", executionFee, relayerFee);
    }

    /*
    * Internal functions
    */

    function _sendMessage(
        address origin,
        Xcm memory xcm,
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

        require(assets.length <= MAX_ASSETS, IGatewayV2.TooManyAssets());

        // Check for duplicate assets
        _checkDuplicateAssets(assets);

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

        require(tokenInfo.isRegistered, IGatewayBase.TokenNotRegistered());
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

    /// @dev Checks for duplicate assets in the provided array and reverts if any are found
    /// @param assets Array of encoded asset data
    function _checkDuplicateAssets(bytes[] memory assets) internal pure {
        if (assets.length > 1) {
            // Create mappings to track seen assets
            mapping(address => bool) memory seenNativeTokens;
            mapping(bytes32 => bool) memory seenForeignTokens;
            
            for (uint256 i = 0; i < assets.length; i++) {
                uint8 assetKind;
                
                // Extract the asset kind from the encoded data
                assembly {
                    // Get the first byte to determine asset kind
                    let assetData := mload(add(add(assets, 32), mul(i, 32)))
                    assetKind := byte(31, mload(add(assetData, 32)))
                }
                
                if (assetKind == 0) { // Native ERC20 token
                    // Decode the token address
                    (, address token,) = abi.decode(assets[i], (uint8, address, uint128));
                    
                    // Check if this token has already been processed
                    if (seenNativeTokens[token]) {
                        revert IGatewayV2.DuplicateAsset();
                    }
                    
                    // Mark this token as seen
                    seenNativeTokens[token] = true;
                } 
                else if (assetKind == 1) { // Foreign ERC20 token
                    // Decode the foreign token ID
                    (, bytes32 foreignID,) = abi.decode(assets[i], (uint8, bytes32, uint128));
                    
                    // Check if this foreign token has already been processed
                    if (seenForeignTokens[foreignID]) {
                        revert IGatewayV2.DuplicateAsset();
                    }
                    
                    // Mark this foreign token as seen
                    seenForeignTokens[foreignID] = true;
                }
                // If an unknown asset kind is encountered, it will be handled by
                // _handleAsset which will revert with InvalidAsset error
            }
        }
    }
}
