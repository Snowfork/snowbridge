// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.20;

import {Registry} from "./Registry.sol";

abstract contract RegistryLookup {
    Registry public immutable registry;

    error LookupError();

    constructor(Registry _registry) {
        registry = _registry;
    }

    function resolve(bytes32 contractID) internal view returns (address) {
        address contractAddress = registry.lookupContract(contractID);
        if (contractAddress == address(0)) {
            revert LookupError();
        }
        return contractAddress;
    }
}
