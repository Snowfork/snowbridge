// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

import "./Bits.sol";

library Bitfield {
    /**
     * @dev Constants used to efficiently calculate the hamming weight of a bitfield. See
     * https://en.wikipedia.org/wiki/Hamming_weight#Efficient_implementation for an explanation of those constants.
     */
    uint256 internal constant M1 = 0x5555555555555555555555555555555555555555555555555555555555555555;
    uint256 internal constant M2 = 0x3333333333333333333333333333333333333333333333333333333333333333;
    uint256 internal constant M4 = 0x0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f;
    uint256 internal constant M8 = 0x00ff00ff00ff00ff00ff00ff00ff00ff00ff00ff00ff00ff00ff00ff00ff00ff;
    uint256 internal constant M16 = 0x0000ffff0000ffff0000ffff0000ffff0000ffff0000ffff0000ffff0000ffff;
    uint256 internal constant M32 = 0x00000000ffffffff00000000ffffffff00000000ffffffff00000000ffffffff;
    uint256 internal constant M64 = 0x0000000000000000ffffffffffffffff0000000000000000ffffffffffffffff;
    uint256 internal constant M128 = 0x00000000000000000000000000000000ffffffffffffffffffffffffffffffff;

    uint256 internal constant ONE = uint256(1);

    /**
     * @notice Draws a random number, derives an index in the bitfield, and sets the bit if it is in the `prior` and not
     * yet set. Repeats that `n` times.
     * @param seed Source of randomness for selecting validator signatures.
     * @param prior Bitfield indicating which validators claim to have signed the commitment.
     * @param n Number of unique bits in prior that must be set in the result. Must be <= number of set bits in `prior`.
     * @param length Length of the bitfield prior to draw bits from. Must be <= prior.length * 256.
     */
    function randomNBitsWithPriorCheck(uint256 seed, uint256[] memory prior, uint256 n, uint256 length)
        internal
        pure
        returns (uint256[] memory bitfield)
    {
        bitfield = new uint256[](prior.length);
        uint256 found = 0;

        for (uint256 i = 0; found < n;) {
            uint256 index = makeIndex(seed, i, length);

            (uint256 element, uint8 within) = toLocation(index);

            // require randomly selected bit to be set in prior and not yet set in bitfield
            if (isNotSetInAOrIsSetInB(prior, bitfield, element, within)) {
                unchecked {
                    i++;
                }
                continue;
            }

            set(bitfield, element, within);

            unchecked {
                found++;
                i++;
            }
        }

        return bitfield;
    }

    /**
     * @dev Helper to create a bitfield.
     */
    function createBitfield(uint256[] calldata bitsToSet, uint256 length)
        public
        pure
        returns (uint256[] memory bitfield)
    {
        // Calculate length of uint256 array based on rounding up to number of uint256 needed
        uint256 arrayLength = (length + 255) / 256;

        bitfield = new uint256[](arrayLength);

        for (uint256 i = 0; i < bitsToSet.length; i++) {
            (uint256 element, uint8 within) = toLocation(bitsToSet[i]);
            set(bitfield, element, within);
        }

        return bitfield;
    }

    /**
     * @notice Calculates the number of set bits by using the hamming weight of the bitfield.
     * The algorithm below is implemented after https://en.wikipedia.org/wiki/Hamming_weight#Efficient_implementation.
     * Further improvements are possible, see the article above.
     */
    function countSetBits(uint256[] memory self) public pure returns (uint256) {
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

    function toLocation(uint256 index) internal pure returns (uint256, uint8) {
        unchecked {
            uint256 element = index / 256;
            uint8 within = uint8(index % 256);
            return (element, within);
        }
    }

    function isSet(uint256[] memory self, uint256 element, uint8 within) internal pure returns (bool) {
        return Bits.bit(self[element], within) == 1;
    }

    function isNotSetInAOrIsSetInB(uint256[] memory a, uint256[] memory b, uint256 element, uint8 within)
        internal
        pure
        returns (bool)
    {
        return Bits.bit(a[element], within) == 0 || Bits.bit(b[element], within) == 1;
    }

    function set(uint256[] memory self, uint256 element, uint8 within) internal pure {
        self[element] = Bits.setBit(self[element], within);
    }

    function clear(uint256[] memory self, uint256 element, uint8 within) internal pure {
        self[element] = Bits.clearBit(self[element], within);
    }

    function makeIndex(uint256 seed, uint256 iteration, uint256 length) internal pure returns (uint256 index) {
        assembly {
            mstore(0x00, seed)
            mstore(0x20, iteration)
            index := mod(keccak256(0x00, 0x40), length)
        }
    }
}
