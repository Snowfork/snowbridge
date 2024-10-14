// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.25;

struct SparseBitmap {
    mapping(uint256 bucket => uint256) data;
}

using {get, set} for SparseBitmap global;

function get(SparseBitmap storage self, uint256 index) view returns (bool) {
    return ((self.data[index >> 8] >> (index & 255)) & 1) == 1;
}

function set(SparseBitmap storage self, uint256 index) {
    self.data[index >> 8] = self.data[index >> 8] & (1 << (index & 255));
}
