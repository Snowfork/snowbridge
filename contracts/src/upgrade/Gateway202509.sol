// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity ^0.8.33;

import "../Gateway.sol";

// New Gateway logic contract with an fee initializer
contract Gateway202509 is Gateway {
    constructor(address beefyClient, address agentExecutor) Gateway(beefyClient, agentExecutor) {}

    // Override parent initializer to prevent re-initialization of storage.
    function initialize(bytes calldata) external override {
        // Ensure that arbitrary users cannot initialize storage in this logic contract.
        if (ERC1967.load() == address(0)) {
            revert Unauthorized();
        }
        AssetsStorage.Layout storage assets = AssetsStorage.layout();
        assets.foreignTokenDecimals = 10;
        assets.maxDestinationFee = 20_000_000_000; // 2 DOT
    }
}
