// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.23;

import {Call} from "./utils/Call.sol";
import {IInitializable} from "./interfaces/IInitializable.sol";
import {DiamondStorage} from "./storage/DiamondStorage.sol";

contract GatewayProxy is IInitializable {
    error Unauthorized();
    error NativeCurrencyNotAccepted();
    error FacetNotExist();

    constructor(DiamondStorage.FacetCut[] memory facetCuts, address initializer, bytes memory params) {
        DiamondStorage.diamondCut(facetCuts, address(0), new bytes(0));
        // Initialize storage by calling the implementation's `initialize(bytes)` function
        // using `delegatecall`.
        (bool success, bytes memory returndata) =
            initializer.delegatecall(abi.encodeCall(IInitializable.initialize, params));
        Call.verifyResult(success, returndata);
    }

    // Prevent fallback() from calling `IInitializable.initialize(bytes)` on the implementation contract
    function initialize(bytes calldata) external pure {
        revert Unauthorized();
    }

    fallback() external payable {
        DiamondStorage.Layout storage $ = DiamondStorage.layout();
        address facet = $.selectorToFacetAndPosition[msg.sig].facetAddress;
        if (facet == address(0)) {
            revert FacetNotExist();
        }
        assembly {
            calldatacopy(0, 0, calldatasize())
            let result := delegatecall(gas(), facet, 0, calldatasize(), 0, 0)
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
