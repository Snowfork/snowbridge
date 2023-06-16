// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.20;

type ParaID is uint32;

using {eq as ==, ne as !=, isNone} for ParaID global;

function eq(ParaID a, ParaID b) pure returns (bool) {
    return ParaID.unwrap(a) == ParaID.unwrap(b);
}

function ne(ParaID a, ParaID b) pure returns (bool) {
    return !eq(a, b);
}

function isNone(ParaID a) pure returns (bool) {
    return ParaID.unwrap(a) == 0;
}
