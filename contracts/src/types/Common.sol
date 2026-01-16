// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.33;

import {UD60x18} from "prb/math/src/UD60x18.sol";

enum OperatingMode {
    Normal,
    RejectingOutboundMessages
}

struct TokenInfo {
    bool isRegistered;
    bytes32 foreignID;
}

using {isNative, isForeign} for TokenInfo global;

function isNative(TokenInfo storage self) view returns (bool) {
    return self.foreignID == bytes32(0);
}

function isForeign(TokenInfo storage self) view returns (bool) {
    return !isNative(self);
}
