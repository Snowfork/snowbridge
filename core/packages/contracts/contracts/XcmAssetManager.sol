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

        string memory name = iToHex(assetHash);
        bytes memory symbol = new bytes(3);
        symbol[0] = bytes(name)[0];
        symbol[1] = bytes(name)[1];
        symbol[2] = bytes(name)[2];
        XcmFungibleAsset created = new XcmFungibleAsset(name, string(symbol));
        created.transferOwnership(msg.sender);
        fungibleAssets[assetHash] = created;
        return created;
    }

    function lookup(bytes32 assetHash) external view override returns (XcmFungibleAsset) {
        return fungibleAssets[assetHash];
    }

    function iToHex(bytes32 buffer) internal pure returns (string memory) {
        // Fixed buffer size for hexadecimal convertion
        bytes memory converted = new bytes(buffer.length * 2);

        bytes memory _base = "0123456789abcdef";

        for (uint256 i = 0; i < buffer.length; i++) {
            converted[i * 2] = _base[uint8(buffer[i]) / _base.length];
            converted[i * 2 + 1] = _base[uint8(buffer[i]) % _base.length];
        }

        return string(converted);
    }
}
