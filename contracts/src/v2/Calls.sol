// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.25;

import {IERC20} from "../interfaces/IERC20.sol";
import {IGateway} from "../interfaces/IGateway.sol";
import {WETH9} from "canonical-weth/WETH9.sol";

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

import {Ticket, TransferKind, OperatingMode} from "./Types.sol";

import {UD60x18, ud60x18, convert} from "prb/math/src/UD60x18.sol";

/// @title Library for implementing Ethereum->Polkadot ERC20 transfers.
library CallsV2 {
    using Address for address;
    using SafeTokenTransfer for IERC20;
    using SafeNativeTransfer for address payable;

    error InvalidProof();
    error InvalidNonce();
    error NotEnoughGas();
    error FeePaymentToLow();
    error Unauthorized();
    error Disabled();
    error AgentAlreadyCreated();
    error AgentDoesNotExist();
    error ChannelAlreadyCreated();
    error ChannelDoesNotExist();
    error InvalidChannelUpdate();
    error AgentExecutionFailed(bytes returndata);
    error InvalidAgentExecutionPayload();
    error InvalidConstructorParams();
    error AlreadyInitialized();
    error TokenNotRegistered();

    error InvalidAsset();

    address public constant WETH_ADDRESS = 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2;

    // Send an XCM with assets to Polkadot Asset Hub
    //
    // Params:
    //   * `xcm` (bytes): SCALE-encoded XCM message
    //   * `assets` (bytes[]): Array of asset transfer instructions
    //
    // The specified assets will be locked/burned locally, and their foreign equivalents will
    // be minted/unlocked on Polkadot Asset Hub.
    //
    // Supported asset instructions:
    // * ERC20: abi.encode(0, tokenAddress, value)
    //
    function sendMessage(bytes calldata xcm, bytes[] calldata assets) external {
        bytes[] memory encodedAssets = new bytes[](assets.length);
        for (uint256 i = 0; i < assets.length; i++) {
            encodedAssets[i] = _handleAsset(assets[i]);
        }

        Ticket memory ticket =
            Ticket({origin: msg.sender, assets: encodedAssets, xcm: xcm});
        _submitOutbound(ticket);
    }

    /*
    * Internal functions
    */

    // Submit an outbound message to Polkadot, after taking fees
    function _submitOutbound(Ticket memory ticket) internal {
        CoreStorage.Layout storage $ = CoreStorage.layout();

        // Ensure outbound messaging is allowed
        _ensureOutboundMessagingEnabled();

        // Wrap provided ether and transfer to AssetHub agent
        address assetHubAgent = Functions.ensureAgent(Constants.ASSET_HUB_AGENT_ID);
        WETH9(payable(WETH_ADDRESS)).deposit{value: msg.value}();
        IERC20(WETH_ADDRESS).safeTransfer(assetHubAgent, msg.value);

        $.outboundNonce = $.outboundNonce + 1;

        bytes memory payload =
            SubstrateTypes.encodePayloadV2(ticket.origin, ticket.assets, ticket.xcm);

        emit IGateway.OutboundMessageAccepted($.outboundNonce, msg.value, payload);
    }

    /// @dev Outbound message can be disabled globally or on a per-channel basis.
    function _ensureOutboundMessagingEnabled() internal view {
        CoreStorage.Layout storage $ = CoreStorage.layout();
        if ($.mode != OperatingMode.Normal) {
            revert Disabled();
        }
    }

    function _handleAsset(bytes calldata asset) internal returns (bytes memory) {
        uint8 assetKind;
        assembly {
            assetKind := calldataload(asset.offset)
        }
        if (assetKind == 0) {
            // ERC20: abi.encode(0, tokenAddress, value)
            (, address token, uint128 amount) =
                abi.decode(asset, (uint8, address, uint128));
            return _handleAssetERC20(token, amount);
        } else {
            revert InvalidAsset();
        }
    }

    function _handleAssetERC20(address token, uint128 amount)
        internal
        returns (bytes memory)
    {
        AssetsStorage.Layout storage $ = AssetsStorage.layout();
        TokenInfo storage info = $.tokenRegistry[token];

        if (!info.isRegistered) {
            revert TokenNotRegistered();
        }

        if (info.foreignID == bytes32(0)) {
            Functions.transferToAgent($.assetHubAgent, token, msg.sender, amount);
            return SubstrateTypes.encodeTransferNativeTokenERC20(token, amount);
        } else {
            Token(token).burn(msg.sender, amount);
            return SubstrateTypes.encodeTransferForeignTokenERC20(info.foreignID, amount);
        }
    }
}
