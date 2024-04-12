// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.23;

import "../../Gateway.sol";

import {UD60x18, convert} from "prb/math/src/UD60x18.sol";
import {PricingStorage} from "../../storage/PricingStorage.sol";
import {AssetsStorage} from "../../storage/AssetsStorage.sol";

contract GatewayV2 is Gateway {
    constructor(
        address beefyClient,
        address agentExecutor,
        ParaID bridgeHubParaID,
        bytes32 bridgeHubAgentID,
        uint8 foreignTokenDecimals
    ) Gateway(beefyClient, agentExecutor, bridgeHubParaID, bridgeHubAgentID, foreignTokenDecimals) {}

    function initialize(bytes memory data) external override {
        // Prevent initialization of storage in implementation contract
        if (ERC1967.load() == address(0)) {
            revert Unauthorized();
        }

        PricingStorage.Layout storage pricing = PricingStorage.layout();
        AssetsStorage.Layout storage assets = AssetsStorage.layout();

        if (pricing.multiplier != convert(0) || assets.destinationMaxTransferFee != 0) {
            revert AlreadyInitialized();
        }

        (UD60x18 multiplier, uint128 maxDestinationFee) = abi.decode(data, (UD60x18, uint128));

        pricing.multiplier = multiplier;
        assets.destinationMaxTransferFee = maxDestinationFee;
    }
}
