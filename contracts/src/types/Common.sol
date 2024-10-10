// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.25;

import {
    MultiAddress, multiAddressFromUint32, multiAddressFromBytes32, multiAddressFromBytes20
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
