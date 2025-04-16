// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

struct SparseBitmap {
    mapping(uint64 => uint128) data;
}

using {get, set} for SparseBitmap global;

function get(SparseBitmap storage self, uint64 index) view returns (bool) {
    uint64 bucket = index >> 7; // Divide by 128
    uint128 mask = uint128(1) << (index & 127); // Bit within the bucket
    return self.data[bucket] & mask != 0;
}

function set(SparseBitmap storage self, uint64 index) {
    uint64 bucket = index >> 7;
    uint128 mask = uint128(1) << (index & 127);
    self.data[bucket] |= mask;
}
