// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.28;

import {Upgrade} from "./Upgrade.sol";
import {IInitializable} from "./interfaces/IInitializable.sol";
import {IUpgradable} from "./interfaces/IUpgradable.sol";
import {IShell} from "./interfaces/IShell.sol";
import {ERC1967} from "./utils/ERC1967.sol";

/**
 * @title Shell
 * Only used in the initial deployment of the GatewayProxy, which was deployed along
 * with this Shell as its logic contract. The Shell was then upgraded using a trusted
 * operator to the full Gateway contract. Currently this code is no longer in use but is
 * kept around for archival purposes.
 */
contract Shell is IShell, IUpgradable, IInitializable {
    address public immutable OPERATOR;

    constructor(address _operator) {
        OPERATOR = _operator;
    }

    function upgrade(address impl, bytes32 implCodeHash, bytes calldata initializerParams) external {
        if (msg.sender != OPERATOR) {
            revert Unauthorized();
        }
        Upgrade.upgrade(impl, implCodeHash, initializerParams);
    }

    function initialize(bytes memory) external view {
        // Prevent initialization of storage in implementation contract
        if (ERC1967.load() == address(0)) {
            revert Unauthorized();
        }
    }

    function operator() external view returns (address) {
        return OPERATOR;
    }
}
