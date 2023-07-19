// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.20;

import {ERC1967} from "./utils/ERC1967.sol";

contract GatewayProxy {
    error NativeCurrencyNotAccepted();

    constructor(address implementation, bytes memory initParams) {
        ERC1967.store(implementation);
        (bool success,) = implementation.delegatecall(abi.encodeCall(GatewayProxy.initialize, initParams));
        if (!success) {
            revert("initialization failed");
        }
    }

    // solhint-disable-next-line no-empty-blocks
    function initialize(bytes calldata params) external {}

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

    receive() external payable {
        revert NativeCurrencyNotAccepted();
    }
}
