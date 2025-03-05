// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import {IInitializable} from "../../src/interfaces/IInitializable.sol";

library AdditionalStorage {
    struct Layout {
        uint256 value;
    }

    bytes32 internal constant SLOT = keccak256("org.snowbridge.storage.additionalStorage");

    function layout() internal pure returns (Layout storage sp) {
        bytes32 slot = SLOT;
        assembly {
            sp.slot := slot
        }
    }
}

// Used to test upgrades.
contract MockGatewayV2 is IInitializable {
    // Reinitialize gateway with some additional storage fields
    function initialize(bytes memory params) external {
        AdditionalStorage.Layout storage $ = AdditionalStorage.layout();

        uint256 value = abi.decode(params, (uint256));

        if (value == 666) {
            revert("initialize failed");
        }

        $.value = value;
    }

    function getValue() external view returns (uint256) {
        return AdditionalStorage.layout().value;
    }
}
