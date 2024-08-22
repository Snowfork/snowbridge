// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.25;

import "../../Gateway.sol";

import {AssetsStorage} from "../../storage/AssetsStorage.sol";
import {LegacyAssetsStorage} from "../../storage/LegacyAssetsStorage.sol";

contract GatewayWithAssetStorageV2 is Gateway {
    constructor(
        address beefyClient,
        address agentExecutor,
        ParaID bridgeHubParaID,
        bytes32 bridgeHubAgentID,
        uint8 foreignTokenDecimals,
        uint128 destinationMaxTransferFee
    )
        Gateway(
            beefyClient,
            agentExecutor,
            bridgeHubParaID,
            bridgeHubAgentID,
            foreignTokenDecimals,
            destinationMaxTransferFee
        )
    {}

    function initialize(bytes memory data) external override {
        // Prevent initialization of storage in implementation contract
        if (ERC1967.load() == address(0)) {
            revert Unauthorized();
        }

        address[] memory tokens = abi.decode(data, (address[]));

        LegacyAssetsStorage.Layout storage $ = LegacyAssetsStorage.layout();
        AssetsStorage.Layout storage $v2 = AssetsStorage.layout();

        $v2.assetHubAgent = $.assetHubAgent;
        $v2.assetHubParaID = $.assetHubParaID;
        $v2.assetHubCreateAssetFee = $.assetHubCreateAssetFee;
        $v2.assetHubReserveTransferFee = $.assetHubReserveTransferFee;
        $v2.registerTokenFee = $.registerTokenFee;
        for (uint256 i = 0; i < tokens.length; i++) {
            $v2.tokenRegistry[tokens[i]].isRegistered = $.tokenRegistry[tokens[i]].isRegistered;
        }
    }
}
