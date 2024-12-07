// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.28;

import "../../Gateway.sol";
import {AssetsStorage} from "../../storage/AssetsStorage.sol";
import {TokenInfo} from "../../Types.sol";

contract Gateway202411 is Gateway {
    constructor(address beefyClient, address agentExecutor) Gateway(beefyClient, agentExecutor) {}

    function initialize(bytes memory) external view override {
        // Prevent initialization of storage in implementation contract
        if (ERC1967.load() == address(0)) {
            revert Unauthorized();
        }
    }

    function tokenInfo(address token) external view returns (TokenInfo memory) {
        return AssetsStorage.layout().tokenRegistry[token];
    }
}
