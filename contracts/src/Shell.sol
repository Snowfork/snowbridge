// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.23;

import {Upgrade} from "./Upgrade.sol";
import {IInitializable} from "./interfaces/IInitializable.sol";
import {IUpgradable} from "./interfaces/IUpgradable.sol";
import {IShell} from "./interfaces/IShell.sol";

// address recoveryOperator = vm.envOr("RECOVERY_OPERATOR", address(0));

contract Shell is IShell, IUpgradable, IInitializable {
    address public immutable operator;

    error Unauthorised();

    constructor(address _operator) {
        operator = _operator;
    }

    function upgrade(address impl, bytes32 implCodeHash, bytes calldata initializerParams) external {
        if (msg.sender != operator) {
            revert Unauthorised();
        }
        Upgrade.upgrade(impl, implCodeHash, initializerParams);
    }

    function initialize(bytes memory params) external {}
}
