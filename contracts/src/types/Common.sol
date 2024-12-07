// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.28;

import {
    MultiAddress,
    multiAddressFromUint32,
    multiAddressFromBytes32,
    multiAddressFromBytes20
} from "./MultiAddress.sol";

import {UD60x18} from "prb/math/src/UD60x18.sol";

enum OperatingMode {
    Normal,
    RejectingOutboundMessages
}

struct TokenInfo {
    bool isRegistered;
    bytes32 foreignID;
}

library TokenInfoFunctions {
    function exists(TokenInfo storage self) internal view returns (bool) {
        return self.isRegistered;
    }

    function isNativeToken(TokenInfo storage self) internal view returns (bool) {
        return self.foreignID == bytes32(0);
    }

    function isForeignToken(TokenInfo storage self) internal view returns (bool) {
        return self.foreignID != bytes32(0);
    }
}
