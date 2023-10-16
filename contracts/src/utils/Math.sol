// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.20;

import {Bits} from "./Bits.sol";

library Math {
    /**
     * @dev Returns the largest of two numbers.
     */
    function max(uint256 a, uint256 b) internal pure returns (uint256) {
        return a > b ? a : b;
    }

    /**
     * @dev Returns the smallest of two numbers.
     */
    function min(uint256 a, uint256 b) internal pure returns (uint256) {
        return a < b ? a : b;
    }

    /**
     * @dev Returns the floor of log2 of a number using bitwise arithmetic.
     */
    function floorOfLog2(uint256 x) internal pure returns (uint256) {
        // Using highest bit set to yield floor(log2(x))
        return Bits.highestBitSet(x);
    }

    /**
     * @dev Returns the ceiling of log2 of a number using bitwise arithmetic.
     */
    function ceilingOfLog2(uint256 x) internal pure returns (uint256) {
        uint256 result = floorOfLog2(x);
        // We round up if there were any extra bits set.
        if (x > (1 << result)) {
            result++;
        }
        return result;
    }
}
