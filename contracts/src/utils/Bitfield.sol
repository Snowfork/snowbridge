// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity ^0.8.33;

import {Bits} from "./Bits.sol";

library Bitfield {
    using Bits for uint256;

    error InvalidSamplingParams();
    error InvalidBitfieldPadding();

    /**
     * @dev Constants used to efficiently calculate the hamming weight of a bitfield. See
     * https://en.wikipedia.org/wiki/Hamming_weight#Efficient_implementation for an explanation of those constants.
     */
    uint256 internal constant M1 =
        0x5555555555555555555555555555555555555555555555555555555555555555;
    uint256 internal constant M2 =
        0x3333333333333333333333333333333333333333333333333333333333333333;
    uint256 internal constant M4 =
        0x0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f;
    uint256 internal constant M8 =
        0x00ff00ff00ff00ff00ff00ff00ff00ff00ff00ff00ff00ff00ff00ff00ff00ff;
    uint256 internal constant M16 =
        0x0000ffff0000ffff0000ffff0000ffff0000ffff0000ffff0000ffff0000ffff;
    uint256 internal constant M32 =
        0x00000000ffffffff00000000ffffffff00000000ffffffff00000000ffffffff;
    uint256 internal constant M64 =
        0x0000000000000000ffffffffffffffff0000000000000000ffffffffffffffff;
    uint256 internal constant M128 =
        0x00000000000000000000000000000000ffffffffffffffffffffffffffffffff;

    uint256 internal constant ONE = uint256(1);

    /**
     * @dev Core subsampling algorithm. Draws a random number, derives an index in the bitfield,
     * and sets the bit if it is in the `priorBitfield` and not yet set. Repeats that `n` times.
     * @param seed Source of randomness for selecting validator signatures.
     * @param priorBitfield Bitfield indicating which validators claim to have signed the commitment.
     * @param priorBitfieldSize Number of bits in priorBitfield Must be <= priorBitfield.length * 256.
     * @param n Number of unique bits in priorBitfield that must be set in the output.
     *          Must be <= number of set bits in priorBitfield.
     */
    function subsample(
        uint256 seed,
        uint256[] memory priorBitfield,
        uint256 priorBitfieldSize,
        uint256 n
    ) internal pure returns (uint256[] memory outputBitfield) {
        if (
            priorBitfield.length != Bitfield.containerLength(priorBitfieldSize)
                || n > countSetBits(priorBitfield, priorBitfieldSize)
        ) {
            revert InvalidSamplingParams();
        }

        outputBitfield = new uint256[](priorBitfield.length);
        uint256 found = 0;

        for (uint256 i = 0; found < n;) {
            uint256 index = makeIndex(seed, i, priorBitfieldSize);

            // require randomly selected bit to be set in priorBitfield and not yet set in bitfield
            if (!isSet(priorBitfield, index) || isSet(outputBitfield, index)) {
                unchecked {
                    i++;
                }
                continue;
            }

            set(outputBitfield, index);

            unchecked {
                found++;
                i++;
            }
        }
    }

    /**
     * @dev Helper to create a bitfield.
     */
    function createBitfield(uint256[] calldata bitsToSet, uint256 length)
        internal
        pure
        returns (uint256[] memory bitfield)
    {
        bitfield = new uint256[](containerLength(length));

        for (uint256 i = 0; i < bitsToSet.length; i++) {
            set(bitfield, bitsToSet[i]);
        }

        return bitfield;
    }

    /**
     * @notice Calculates the number of set bits by using the hamming weight of the bitfield.
     * The algorithm below is implemented after https://en.wikipedia.org/wiki/Hamming_weight#Efficient_implementation.
     * Further improvements are possible, see the article above.
     */
    function countSetBits(uint256[] memory self) internal pure returns (uint256) {
        unchecked {
            uint256 count = 0;
            for (uint256 i = 0; i < self.length; i++) {
                uint256 x = self[i];
                x = (x & M1) + ((x >> 1) & M1); //put count of each  2 bits into those  2 bits
                x = (x & M2) + ((x >> 2) & M2); //put count of each  4 bits into those  4 bits
                x = (x & M4) + ((x >> 4) & M4); //put count of each  8 bits into those  8 bits
                x = (x & M8) + ((x >> 8) & M8); //put count of each 16 bits into those 16 bits
                x = (x & M16) + ((x >> 16) & M16); //put count of each 32 bits into those 32 bits
                x = (x & M32) + ((x >> 32) & M32); //put count of each 64 bits into those 64 bits
                x = (x & M64) + ((x >> 64) & M64); //put count of each 128 bits into those 128 bits
                x = (x & M128) + ((x >> 128) & M128); //put count of each 256 bits into those 256 bits
                count += x;
            }
            return count;
        }
    }

    /**
     * @notice Calculates the number of set bits in the first `maxBits` bits of the bitfield.
     * This is a bounded variant of `countSetBits` that only counts bits within the specified range.
     *
     * @dev Example usage:
     * If a bitfield has bits set at positions [0, 5, 10, 256, 300]:
     * - countSetBits(bitfield, 11) returns 3 (bits 0, 5, 10)
     * - countSetBits(bitfield, 257) returns 4 (bits 0, 5, 10, 256)
     * - countSetBits(bitfield, 1000) returns 5 (all bits)
     *
     * @param self The bitfield to count bits in
     * @param maxBits The maximum number of bits to count (counting from bit 0)
     * @return count The number of set bits in the first `maxBits` positions
     */
    function countSetBits(uint256[] memory self, uint256 maxBits) internal pure returns (uint256) {
        if (maxBits == 0 || self.length == 0) {
            return 0;
        }

        unchecked {
            uint256 count = 0;
            uint256 fullElements = maxBits / 256;
            uint256 remainingBits = maxBits % 256;

            // Count bits in full 256-bit elements
            for (uint256 i = 0; i < fullElements && i < self.length; i++) {
                uint256 x = self[i];
                x = (x & M1) + ((x >> 1) & M1); //put count of each  2 bits into those  2 bits
                x = (x & M2) + ((x >> 2) & M2); //put count of each  4 bits into those  4 bits
                x = (x & M4) + ((x >> 4) & M4); //put count of each  8 bits into those  8 bits
                x = (x & M8) + ((x >> 8) & M8); //put count of each 16 bits into those 16 bits
                x = (x & M16) + ((x >> 16) & M16); //put count of each 32 bits into those 32 bits
                x = (x & M32) + ((x >> 32) & M32); //put count of each 64 bits into those 64 bits
                x = (x & M64) + ((x >> 64) & M64); //put count of each 128 bits into those 128 bits
                x = (x & M128) + ((x >> 128) & M128); //put count of each 256 bits into those 256 bits
                count += x;
            }

            // Count bits in the partial element (if any)
            if (remainingBits > 0 && fullElements < self.length) {
                uint256 mask = (ONE << remainingBits) - 1;
                uint256 x = self[fullElements] & mask;
                x = (x & M1) + ((x >> 1) & M1);
                x = (x & M2) + ((x >> 2) & M2);
                x = (x & M4) + ((x >> 4) & M4);
                x = (x & M8) + ((x >> 8) & M8);
                x = (x & M16) + ((x >> 16) & M16);
                x = (x & M32) + ((x >> 32) & M32);
                x = (x & M64) + ((x >> 64) & M64);
                x = (x & M128) + ((x >> 128) & M128);
                count += x;
            }

            return count;
        }
    }

    function isSet(uint256[] memory self, uint256 index) internal pure returns (bool) {
        uint256 element = index >> 8;
        return self[element].bit(uint8(index)) == 1;
    }

    function set(uint256[] memory self, uint256 index) internal pure {
        uint256 element = index >> 8;
        self[element] = self[element].setBit(uint8(index));
    }

    function unset(uint256[] memory self, uint256 index) internal pure {
        uint256 element = index >> 8;
        self[element] = self[element].clearBit(uint8(index));
    }

    function makeIndex(uint256 seed, uint256 iteration, uint256 length)
        internal
        pure
        returns (uint256 index)
    {
        // Handle case where length is 0 to prevent infinite loop in subsample
        if (length == 0) {
            return 0;
        }

        assembly {
            mstore(0x00, seed)
            mstore(0x20, iteration)
            index := mod(keccak256(0x00, 0x40), length)
        }
    }

    // Calculate length of uint256 bitfield array based on rounding up to number of uint256 needed
    function containerLength(uint256 bitfieldSize) internal pure returns (uint256) {
        return (bitfieldSize + 255) / 256;
    }

    /**
     * @dev Validate that all padding bits in the bitfield (beyond length) are zero.
     * @param bitfield The bitfield to validate
     * @param length The number of valid bits (padding starts after this)
     */
    function validatePadding(uint256[] memory bitfield, uint256 length) internal pure {
        uint256 containerLen = containerLength(length);
        if (containerLen == 0 || bitfield.length == 0) {
            return;
        }

        // Check if there are padding bits in the last element
        uint256 validBitsInLastElement = length % 256;
        if (validBitsInLastElement == 0) {
            // All bits in last element are valid, no padding
            return;
        }

        // Create a mask for padding bits: all bits from validBitsInLastElement to 255
        uint256 paddingMask = type(uint256).max << validBitsInLastElement;
        uint256 lastElement = bitfield[containerLen - 1];
        if ((lastElement & paddingMask) != 0) {
            revert InvalidBitfieldPadding();
        }
    }
}
