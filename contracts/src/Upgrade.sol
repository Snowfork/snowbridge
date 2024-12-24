// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.28;

import {ERC1967} from "./utils/ERC1967.sol";
import {Call} from "./utils/Call.sol";
import {Address} from "./utils/Address.sol";
import {IInitializable} from "./interfaces/IInitializable.sol";
import {IUpgradable} from "./interfaces/IUpgradable.sol";

/// @dev Upgrades implementation contract
library Upgrade {
    using Address for address;

    function upgrade(address impl, bytes32 implCodeHash, bytes memory initializerParams) internal {
        // Verify that the implementation is actually a contract
        if (!impl.isContract()) {
            revert IUpgradable.InvalidContract();
        }

        // As a sanity check, ensure that the codehash of implementation contract
        // matches the codehash in the upgrade proposal
        if (impl.codehash != implCodeHash) {
            revert IUpgradable.InvalidCodeHash();
        }

        // Update the proxy with the address of the new implementation
        ERC1967.store(impl);

        // Call the initializer
        (bool success, bytes memory returndata) =
            impl.delegatecall(abi.encodeCall(IInitializable.initialize, initializerParams));
        Call.verifyResult(success, returndata);

        emit IUpgradable.Upgraded(impl);
    }
}
