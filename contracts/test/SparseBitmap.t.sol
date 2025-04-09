// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import "forge-std/Test.sol";
import "forge-std/console.sol";

import {SparseBitmap} from "../src/utils/SparseBitmap.sol";

contract SparseBitmapTest is Test {
    SparseBitmap bitmap;

    function setUp() public {}

    function testSetAndGetSingleIndex() public {
        uint64 index = 384;
        assertFalse(bitmap.get(index), "Bit should initially be unset");
        bitmap.set(index);
        assertTrue(bitmap.get(index), "Bit should be set after calling set()");
    }

    function testUnsetBitsRemainFalse() public {
        assertFalse(bitmap.get(0));
        assertFalse(bitmap.get(127));
        assertFalse(bitmap.get(128));
        assertFalse(bitmap.get(10000));
    }

    function testBucketBoundaries() public {
        // Bit 127 and 128 are in different buckets
        bitmap.set(127);
        bitmap.set(128);

        assertTrue(bitmap.get(127), "Bit 127 should be set");
        assertTrue(bitmap.get(128), "Bit 128 should be set");
        assertFalse(bitmap.get(129), "Bit 129 should not be set");
    }

    function testMultipleBitsInSameBucket() public {
        // All these are in the same bucket (bucket 3)
        bitmap.set(384);  // 3 * 128
        bitmap.set(385);
        bitmap.set(511);  // Last bit of bucket 3

        assertTrue(bitmap.get(384));
        assertTrue(bitmap.get(385));
        assertTrue(bitmap.get(511));
        assertFalse(bitmap.get(383));
    }

    function testLargeIndexes() public {
        uint64[5] memory indices;
        indices[0] = 1 << 12;            // 4096
        indices[1] = 1 << 20;            // 1,048,576
        indices[2] = 1 << 32;            // 4,294,967,296
        indices[3] = (1 << 40) + 12345;  // Large index
        indices[4] = type(uint64).max;   // Max possible value

        for (uint i = 0; i < indices.length; i++) {
            uint64 index = indices[i];
            assertFalse(bitmap.get(index), "Bit should be initially unset");
            bitmap.set(index);
            assertTrue(bitmap.get(index), "Bit should be set after calling set()");
        }
    }

    function testRepeatedSetIsIdempotent() public {
        bitmap.set(999);
        assertTrue(bitmap.get(999));
        bitmap.set(999); // Set again
        assertTrue(bitmap.get(999), "Bit should still be set after second set");
    }
}
