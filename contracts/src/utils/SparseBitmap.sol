// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

struct SparseBitmap {
    mapping(uint256 bucket => uint256) data;
}

using {get, set} for SparseBitmap global;

function get(SparseBitmap storage self, uint256 index) view returns (bool) {
    uint256 bucket = index >> 8;
    uint256 mask = 1 << (index & 0xff);
    return self.data[bucket] & mask != 0;
}

function set(SparseBitmap storage self, uint256 index) {
    uint256 bucket = index >> 8;
    uint256 mask = 1 << (index & 0xff);
    self.data[bucket] |= mask;
}
