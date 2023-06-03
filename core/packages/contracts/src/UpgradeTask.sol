// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

import {Gateway} from "./Gateway.sol";
import {Registry} from "./Registry.sol";

abstract contract UpgradeTask is Gateway {
    constructor(Registry registry) Gateway(registry) {}
    function run() external virtual;
}
