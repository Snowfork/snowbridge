// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.20;

import {ERC1967} from "./utils/ERC1967.sol";

contract GatewayProxy {
    error InitializationFailed();
    error Unauthorized();
    error NativeCurrencyNotAccepted();

    constructor(address implementation, bytes memory params) {
        // Store the address of the implementation contract
        ERC1967.store(implementation);
        // Initialize storage by calling the implementation's `initialize(bytes)` function
        // using `delegatecall`.
        (bool success,) = implementation.delegatecall(abi.encodeCall(GatewayProxy.initialize, params));
        if (!success) {
            revert InitializationFailed();
        }
    }

    // Prevent fallback() from calling `initialize(bytes)` on the implementation contract
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

    // Prevent users from unwittingly sending ether to the gateway, as these funds
    // would otherwise be lost forever.
    receive() external payable {
        revert NativeCurrencyNotAccepted();
    }
}
