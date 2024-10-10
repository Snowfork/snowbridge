// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.25;

import {IERC20} from "./interfaces/IERC20.sol";
import {IGateway} from "./interfaces/IGateway.sol";

import {SafeTokenTransferFrom} from "./utils/SafeTransfer.sol";

import {AssetsStorage, TokenInfo} from "./storage/AssetsStorage.sol";
import {CoreStorage} from "./storage/CoreStorage.sol";

import {SubstrateTypes} from "./SubstrateTypes.sol";
import {ParaID, MultiAddress, Ticket, Costs, TransferKind, TicketV2} from "./Types.sol";
import {Address} from "./utils/Address.sol";
import {AgentExecutor} from "./AgentExecutor.sol";
import {Agent} from "./Agent.sol";
import {Call} from "./utils/Call.sol";
import {Token} from "./Token.sol";
import {WETH9} from "canonical-weth/WETH9.sol";

/// @title Library for implementing Ethereum->Polkadot ERC20 transfers.
library Assets {
    using Address for address;
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

    /*
    *     _____   __________ .___          ____
    *    /  _  \  \______   \|   | ___  __/_   |
    *   /  /_\  \  |     ___/|   | \  \/ / |   |
    *  /    |    \ |    |    |   |  \   /  |   |
    *  \____|__  / |____|    |___|   \_/   |___|
    *          \/
    */

    function isTokenRegistered(address token) external view returns (bool) {
        return AssetsStorage.layout().tokenRegistry[token].isRegistered;
    }

    /// @dev transfer tokens from the sender to the specified agent
    function _transferToAgent(
        address agent,
        address token,
        address sender,
        uint128 amount
    ) internal {
        if (!token.isContract()) {
            revert InvalidToken();
        }

        if (amount == 0) {
            revert InvalidAmount();
        }

        IERC20(token).safeTransferFrom(sender, agent, amount);
    }

    /// @dev Registers a token (only native tokens at this time)
    /// @param token The ERC20 token address.
    function registerToken(address token) external returns (TicketV2 memory ticket) {}

    // @dev Register a new fungible Polkadot token for an agent
    function registerForeignToken(
        bytes32 foreignTokenID,
        string memory name,
        string memory symbol,
        uint8 decimals
    ) external {
        AssetsStorage.Layout storage $ = AssetsStorage.layout();
        if ($.tokenAddressOf[foreignTokenID] != address(0)) {
            revert TokenAlreadyRegistered();
        }
        Token token = new Token(name, symbol, decimals);
        TokenInfo memory info =
            TokenInfo({isRegistered: true, foreignID: foreignTokenID});

        $.tokenAddressOf[foreignTokenID] = address(token);
        $.tokenRegistry[address(token)] = info;

        emit IGateway.ForeignTokenRegistered(foreignTokenID, address(token));
    }

    // @dev Mint foreign token from Polkadot
    function mintForeignToken(bytes32 foreignTokenID, address recipient, uint256 amount)
        external
    {
        address token = _ensureTokenAddressOf(foreignTokenID);
        Token(token).mint(recipient, amount);
    }

    // @dev Transfer ERC20 to `recipient`
    function transferNativeToken(
        address executor,
        address agent,
        address token,
        address recipient,
        uint128 amount
    ) external {
        bytes memory call =
            abi.encodeCall(AgentExecutor.transferToken, (token, recipient, amount));
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

    function _isTokenRegistered(address token) internal view returns (bool) {
        AssetsStorage.Layout storage $ = AssetsStorage.layout();
        return $.tokenRegistry[token].isRegistered;
    }

    /*
    *     _____   __________ .___         ________
    *    /  _  \  \______   \|   | ___  __\_____  \
    *   /  /_\  \  |     ___/|   | \  \/ / /  ____/§
    *  /    |    \ |    |    |   |  \   / /       \
    *  \____|__  / |____|    |___|   \_/  \_______ \
    *          \/                                 \/
    */

    function sendMessage(bytes calldata xcm, bytes[] calldata assets)
        external
        returns (TicketV2 memory)
    {
        AssetsStorage.Layout storage $ = AssetsStorage.layout();

        bytes[] memory xfers = new bytes[](assets.length);
        for (uint256 i = 0; i < assets.length; i++) {
            xfers[i] = _handleAsset(assets[i]);
        }

        return TicketV2({costs: Costs({foreign: 0, native: 0}), xfers: xfers, xcm: xcm});
    }

    function _handleAsset(bytes calldata asset) internal returns (bytes memory) {
        uint8 assetKind;
        assembly {
            assetKind := calldataload(asset.offset)
        }
        if (assetKind == 0) {
            (, address token, uint128 amount) =
                abi.decode(asset, (uint8, address, uint128));
            return _handleAssetERC20(token, amount);
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
            _transferToAgent($.assetHubAgent, token, msg.sender, amount);
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
