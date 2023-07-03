// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.20;

import {Gateway} from "./Gateway.sol";
import {Registry} from "./Registry.sol";
import {UpgradeProxy} from "./UpgradeProxy.sol";

abstract contract UpgradeTask is Gateway {
    constructor(Registry registry) Gateway(registry) {}
    function run(bytes calldata params) external virtual;

    function createUpgradeMessage() external view returns (bytes memory) {
        return abi.encode(
            UpgradeProxy.Message(
                UpgradeProxy.Action.Upgrade,
                abi.encode(UpgradeProxy.UpgradePayload(address(this), createUpgradeParams()))
            )
        );
    }

    function createUpgradeParams() public view virtual returns (bytes memory) {
        return "0x";
    }
}
