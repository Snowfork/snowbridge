// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.28;

import {ERC1967} from "./utils/ERC1967.sol";
import {Call} from "./utils/Call.sol";
import {IInitializable} from "./interfaces/IInitializable.sol";

contract BeefyClientWrapperProxy is IInitializable {
    error Unauthorized();

    constructor(address implementation, bytes memory params) {
        // Store the address of the implementation contract
        ERC1967.store(implementation);
        // Initialize storage by calling the implementation's `initialize(bytes)` function
        // using `delegatecall`.
        (bool success, bytes memory returndata) =
            implementation.delegatecall(abi.encodeCall(IInitializable.initialize, params));
        Call.verifyResult(success, returndata);
    }

    // Prevent fallback() from calling `IInitializable.initialize(bytes)` on the implementation
    // contract
    function initialize(bytes calldata) external pure {
        revert Unauthorized();
    }

    fallback() external payable {
        address implementation = ERC1967.load();
        assembly {
            calldatacopy(0, 0, calldatasize())
            let result := delegatecall(gas(), implementation, 0, calldatasize(), 0, 0)
            returndatacopy(0, 0, returndatasize())
            switch result
            case 0 { revert(0, returndatasize()) }
            default { return(0, returndatasize()) }
        }
    }

    // Note: No receive() needed - fallback() handles plain ETH transfers
    // by delegating to the implementation's receive() function
}
