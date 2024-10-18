// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.25;

import {IERC20} from "../interfaces/IERC20.sol";
import {IGateway} from "../interfaces/IGateway.sol";

import {SafeTokenTransferFrom} from "../utils/SafeTransfer.sol";

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

import {Ticket, TransferKind} from "./Types.sol";

import {UD60x18, ud60x18, convert} from "prb/math/src/UD60x18.sol";

/// @title Library for implementing Ethereum->Polkadot ERC20 transfers.
library CallsV2 {
    using Address for address;
    using SafeTokenTransferFrom for IERC20;

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
    function sendMessage(bytes calldata xcm, bytes[] calldata assets)
        external
        returns (Ticket memory)
    {
        bytes[] memory xfers = new bytes[](assets.length);
        for (uint256 i = 0; i < assets.length; i++) {
            xfers[i] = _handleAsset(assets[i]);
        }

        return Ticket({xfers: xfers, xcm: xcm});
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
            return
                SubstrateTypes.encodeTransfer(TransferKind.LocalReserve, token, amount);
        } else {
            Token(token).burn(msg.sender, amount);
            return SubstrateTypes.encodeTransfer(
                TransferKind.DestinationReserve, token, amount
            );
        }
    }
}
