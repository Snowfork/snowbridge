// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.20;

library Counter {
    function createCounter(uint256 length) internal pure returns (uint256[] memory) {
        uint256 counterLength = length / 16 + (length % 16 == 0 ? 0 : 1);
        return new uint256[](counterLength);
    }

    function get(uint256[] storage self, uint256 index) internal view returns (uint16) {
        uint256 element = index >> 4;
        uint8 inside = uint8(index) & 0x0F;
        return uint16((self[element] >> (16 * inside)) & 0xFFFF);
    }

    function set(uint256[] storage self, uint256 index, uint16 value) internal {
        uint256 element = index >> 4;
        uint8 inside = uint8(index) & 0x0F;
        uint256 zero = ~(uint256(0xFFFF) << (16 * inside));
        uint256 shiftedValue = uint256(value) << (16 * inside);
        self[element] = self[element] & zero | shiftedValue;
    }
}
