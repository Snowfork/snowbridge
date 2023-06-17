// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.20;

import {Gateway} from "./Gateway.sol";
import {Registry} from "./Registry.sol";

abstract contract UpgradeTask is Gateway {
    constructor(Registry registry) Gateway(registry) {}
    function run() external virtual;
}
