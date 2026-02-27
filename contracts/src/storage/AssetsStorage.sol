// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.33;

import {TokenInfo} from "../types/Common.sol";
import {ParaID} from "../v1/Types.sol";

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
        // Previously used in V1 for registering a native token, this is now obsolete as token registration has been moved to V2 without on-chain fees.
        uint256 __obsolete_1;
        // Foreign token registry by token ID
        mapping(bytes32 foreignID => address) tokenAddressOf;
        uint8 foreignTokenDecimals;
        // The maximum fee that can be sent to a destination parachain to pay for execution (DOT).
        // Has two functions:
        // * Reduces the ability of users to perform arbitrage using a favourable exchange rate
        // * Prevents users from mistakenly providing too much fees, which would drain AssetHub's
        //   sovereign account here on Ethereum.
        uint128 maxDestinationFee;
    }

    bytes32 internal constant SLOT = keccak256("org.snowbridge.storage.assets");

    function layout() internal pure returns (Layout storage $) {
        bytes32 slot = SLOT;
        assembly {
            $.slot := slot
        }
    }
}
