// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.22;

/**
 * @title A utility library for 16 bit counters packed in 256 bit array.
 * @dev The BeefyClient needs to store a count of how many times a validators signature is used. In solidity
 * a uint16 would take up as much space as a uin256 in storage, making storing counters for 1000 validators
 * expensive in terms of gas. The BeefyClient only needs 16 bits per counter. This library allows us to pack
 * 16 uint16 into a single uint256 and save 16x storage.
 *
 * Layout of 8 counters (2 uint256)
 * We store all counts in a single large uint256 array and convert from index from the logical uint16 array
 * to the physical uint256 array.
 *
 *           0                                               1                                               2
 * uint256[] |-- -- -- -- -- -- -- -- -- -- -- -- YY -- -- --|-- -- -- -- -- -- XX -- -- -- -- -- -- -- -- --|
 * uint16[]  |--|--|--|--|--|--|--|--|--|--|--|--|YY|--|--|--|--|--|--|--|--|--|XX|--|--|--|--|--|--|--|--|--|
 *           0  1  2  3  4  5  6  7  8  9  10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 32
 *
 * Logical Index Layout
 * We use the first 4
 * |-------...---------|----|
 * 256                 4    0
 *        ^index          ^bit-index
 *
 * In the above table counter YY is at logical index 12 in the uint16 array. It will convert to a physical
 * index of 0 in the physical uint256 array and then to bit-index of 192 to 207 of that uint256. In the
 * above table counter XX is at logical index 22. It will convert to a physical index of 1 in the array and
 * then to bit-index 128 to 143 of uint256[1].
 */
library Uint16Array {
    struct Array {
        uint256[] data;
    }
    /**
     * @dev Creates a new counter which can store at least `length` counters.
     * @param length The amount of counters.
     */

    function create(uint256 length) internal pure returns (Array memory) {
        // create space for `length` elements and round up if needed.
        uint256 bufferLength = length / 16 + (length % 16 == 0 ? 0 : 1);
        return Array({data: new uint256[](bufferLength)});
    }

    /**
     * @dev Gets the counter at the logical index
     * @param self The counter array.
     * @param index The logical index.
     */
    function get(Array storage self, uint256 index) internal view returns (uint16) {
        // Right-shift the index by 4. This truncates the first 4 bits (bit-index) leaving us with the index
        // into the array.
        uint256 element = index >> 4;
        // Mask out the first 4 bytes of the logical index to give us the bit-index.
        uint8 inside = uint8(index) & 0x0F;
        // find the element in the array, shift until its bit index and mask to only take the first 16 bits.
        return uint16((self.data[element] >> (16 * inside)) & 0xFFFF);
    }

    /**
     * @dev Sets the counter at the logical index.
     * @param self The counter array.
     * @param index The logical index of the counter in the array.
     * @param value The value to set the counter to.
     */
    function set(Array storage self, uint256 index, uint16 value) internal {
        // Right-shift the index by 4. This truncates the first 4 bits (bit-index) leaving us with the index
        // into the array.
        uint256 element = index >> 4;
        // Mask out the first 4 bytes of the logical index to give us the bit-index.
        uint8 inside = uint8(index) & 0x0F;
        // Create a zero mask which will clear the existing value at the bit-index.
        uint256 zero = ~(uint256(0xFFFF) << (16 * inside));
        // Shift the value to the bit index.
        uint256 shiftedValue = uint256(value) << (16 * inside);
        // Take the element, apply the zero mask to clear the existing value, and then apply the shifted value with bitwise or.
        self.data[element] = self.data[element] & zero | shiftedValue;
    }
}
