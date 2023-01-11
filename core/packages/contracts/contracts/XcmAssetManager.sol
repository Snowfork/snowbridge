// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "./XcmAssetLookup.sol";

/// @dev Translates substrate assets to token addresses.
contract XcmAssetManager is XcmAssetLookup {
    /// @dev A mapping or 32 byte hashed asset locations to token addresses
    mapping(bytes32 => XcmFungibleAsset) public fungibleAssets;

    function lookupOrCreate(bytes32 assetHash) external override returns (XcmFungibleAsset) {
        XcmFungibleAsset asset = fungibleAssets[assetHash];
        if (address(asset) != address(0)) {
            return asset;
        }

        XcmFungibleAsset created = new XcmFungibleAsset();
        created.transferOwnership(msg.sender);
        fungibleAssets[assetHash] = created;
        return created;
    }

    function lookup(bytes32 assetHash) external view override returns (XcmFungibleAsset) {
        return fungibleAssets[assetHash];
    }
}
