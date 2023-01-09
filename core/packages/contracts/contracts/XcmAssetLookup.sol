// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "./XcmFungibleAsset.sol";

/// @dev the interface for looking up assets.
interface XcmAssetLookup {
    /// @dev looks up token for a subtrate asset.
    /// @param assetHash the hash of the substrate location.
    function lookup(bytes32 assetHash) external view returns (XcmFungibleAsset);

    /// @dev looks up or creates a token for a subtrate asset.
    /// @param assetHash the hash of the substrate location.
    function lookupOrCreate(bytes32 assetHash) external returns (XcmFungibleAsset);
}
