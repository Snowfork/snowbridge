// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.28;

import "../Gateway.sol";

// New Gateway logic contract with an fee initializer
contract GatewayWithFeeInitializer is Gateway {
    constructor(
        address beefyClient,
        address agentExecutor
    )
        Gateway(
            beefyClient,
            agentExecutor
        )
    {}

    struct Config {
        uint8 foreignTokenDecimals;
        uint128 maxDestinationFee;
    }

    // Override parent initializer to prevent re-initialization of storage.
    function initialize(bytes calldata data) external override {
        // Ensure that arbitrary users cannot initialize storage in this logic contract.
        if (ERC1967.load() == address(0)) {
            revert Unauthorized();
        }

        Config memory config = abi.decode(data, (Config));

        AssetsStorage.Layout storage assets = AssetsStorage.layout();
        assets.foreignTokenDecimals = config.foreignTokenDecimals;
        assets.maxDestinationFee = config.maxDestinationFee;
    }
}
