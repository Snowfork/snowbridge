// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.28;

import {TokenInfo, ParaID} from "../Types.sol";

library AssetsStorage {
    struct Layout {
        // Native token registry by token address
        mapping(address token => TokenInfo) tokenRegistry;
        address assetHubAgent;
        ParaID assetHubParaID;
        // XCM fee charged by AssetHub for registering a token (DOT)
        uint128 assetHubCreateAssetFee;
        // XCM fee charged by AssetHub for receiving a token from the Gateway (DOT)
        uint128 assetHubReserveTransferFee;
        // Extra fee for registering a token, to discourage spamming (Ether)
        uint256 registerTokenFee;
        // Foreign token registry by token ID
        mapping(bytes32 foreignID => address) tokenAddressOf;
    }

    bytes32 internal constant SLOT = keccak256("org.snowbridge.storage.assets");

    function layout() internal pure returns (Layout storage $) {
        bytes32 slot = SLOT;
        assembly {
            $.slot := slot
        }
    }
}
