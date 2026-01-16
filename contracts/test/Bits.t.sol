// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.33;

import {Test} from "forge-std/Test.sol";
import {Bits} from "../src/utils/Bits.sol";

contract BitsTest is Test {
    using Bits for uint256;

    BitsCaller bc;

    function setUp() public {
        bc = new BitsCaller();
    }

    function test_set_clear_toggle_bit() public {
        uint256 v = 0;
        v = v.setBit(5);
        assertEq(v.bit(5), 1);
        assertTrue(v.bitSet(5));

        v = v.clearBit(5);
        assertEq(v.bit(5), 0);

        v = v.toggleBit(3);
        assertEq(v.bit(3), 1);
        v = v.toggleBit(3);
        assertEq(v.bit(3), 0);
    }

    function test_bit_ops_and_equality() public {
        uint256 a = 0;
        a = a.setBit(1).setBit(3);
        uint256 b = 0;
        b = b.setBit(3);

        // bitEqual: bit 3 equal, bit 1 differs
        assertTrue(a.bitEqual(b, 3));
        assertFalse(a.bitEqual(b, 1));

        // bitwise single-bit ops
        assertEq(a.bitAnd(b, 3), 1);
        assertEq(a.bitOr(b, 2), 0);
        assertEq(a.bitXor(b, 1), 1);
        assertEq(a.bitNot(3), 0);
    }

    function test_bits_range_and_bounds() public {
        uint256 v = 0;
        v = v.setBit(0).setBit(1).setBit(2).setBit(10);

        // bits(startIndex, numBits)
        assertEq(v.bits(0, 3), 7);
        assertEq(v.bits(1, 2), 3);

        // invalid: numBits == 0 (call via helper so expectRevert catches it)
        vm.expectRevert(bytes("out of bounds"));
        bc.bits(v, 0, 0);

        // invalid: start+numBits > 256
        vm.expectRevert(bytes("out of bounds"));
        bc.bits(v, 250, 8);
    }

    function test_highest_and_lowest_bit_set() public {
        uint256 v = 0;
        v = v.setBit(7).setBit(2);
        assertEq(v.highestBitSet(), 7);
        assertEq(v.lowestBitSet(), 2);

        vm.expectRevert(bytes("should not be zero"));
        bc.highestBitSet(0);

        vm.expectRevert(bytes("should not be zero"));
        bc.lowestBitSet(0);
    }
}

contract BitsCaller {
    using Bits for uint256;

    function bits(uint256 self, uint8 startIndex, uint16 numBits) external pure returns (uint256) {
        return self.bits(startIndex, numBits);
    }

    function highestBitSet(uint256 self) external pure returns (uint8) {
        return self.highestBitSet();
    }

    function lowestBitSet(uint256 self) external pure returns (uint8) {
        return self.lowestBitSet();
    }
}
