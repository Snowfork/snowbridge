// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import {Test} from "forge-std/Test.sol";
import {console} from "forge-std/console.sol";
import {BitfieldWrapper} from "./mocks/BitfieldWrapper.sol";
import {Bitfield} from "../src/utils/Bitfield.sol";

import {stdJson} from "forge-std/StdJson.sol";

contract BitfieldTest is Test {
    using stdJson for string;

     uint256 public constant SEED = 2954466101346023252933346884990731083400112195551952331583346342070284928184;

    function testBitfieldSubsampling() public {
        BitfieldWrapper bw = new BitfieldWrapper();

        string memory json =
            vm.readFile(string.concat(vm.projectRoot(), "/test/data/beefy-validator-set.json"));
        uint32 setSize = uint32(json.readUint(".validatorSetSize"));
        uint256[] memory bitSetArray = json.readUintArray(".participants");

        uint256[] memory initialBitField = bw.createBitfield(bitSetArray, setSize);
        uint256[] memory finalBitfield = bw.subsample(SEED, initialBitField, setSize, 30);

        uint256 counter = 0;
        for (uint256 i = 0; i < bitSetArray.length; i++) {
            if (Bitfield.isSet(finalBitfield, bitSetArray[i])) {
                counter++;
            }
        }
        assertEq(30, counter);
        assertEq(Bitfield.countSetBits(finalBitfield), counter);
    }

    function testBitfieldWithZeroLength() public {
        BitfieldWrapper bw = new BitfieldWrapper();

        // Empty bitfield with zero length
        uint256[] memory emptyBits;
        emptyBits = new uint256[](0);

        // This should create a valid bitfield with 0 length
        uint256[] memory initialBitField = bw.createBitfield(emptyBits, 0);

        // When length is 0, subsample should handle it gracefully without infinite loop
        // Since we're asking for 0 bits, it should return an empty bitfield
        uint256[] memory finalBitfield = bw.subsample(67, initialBitField, 0, 0);

        // Ensure the returned bitfield has the expected length and no set bits
        assertEq(finalBitfield.length, initialBitField.length);
        assertEq(Bitfield.countSetBits(finalBitfield), 0);
    }

    function testBoundedCountSetBits() public {
        BitfieldWrapper bw = new BitfieldWrapper();

        // Create a bitfield with some known set bits
        // Set bits at positions: 0, 5, 10, 255, 256, 300, 500
        uint256[] memory bitsToSet = new uint256[](7);
        bitsToSet[0] = 0;
        bitsToSet[1] = 5;
        bitsToSet[2] = 10;
        bitsToSet[3] = 255;
        bitsToSet[4] = 256;
        bitsToSet[5] = 300;
        bitsToSet[6] = 500;

        uint256[] memory bitfield = bw.createBitfield(bitsToSet, 600);

        // Test counting first 1 bit (should find bit 0)
        assertEq(bw.countSetBits(bitfield, 1), 1);

        // Test counting first 6 bits (should find bit 0 and 5)
        assertEq(bw.countSetBits(bitfield, 6), 2);

        // Test counting first 11 bits (should find bits 0, 5, 10)
        assertEq(bw.countSetBits(bitfield, 11), 3);

        // Test counting first 256 bits (should find bits 0, 5, 10, 255)
        assertEq(bw.countSetBits(bitfield, 256), 4);

        // Test counting first 257 bits (should find bits 0, 5, 10, 255, 256)
        assertEq(bw.countSetBits(bitfield, 257), 5);

        // Test counting first 301 bits (should find bits 0, 5, 10, 255, 256, 300)
        assertEq(bw.countSetBits(bitfield, 301), 6);

        // Test counting all bits (should find all 7 bits)
        assertEq(bw.countSetBits(bitfield, 600), 7);

        // Test counting more than available bits (should still find all 7)
        assertEq(bw.countSetBits(bitfield, 1000), 7);
    }

    function testBoundedCountSetBitsEdgeCases() public {
        BitfieldWrapper bw = new BitfieldWrapper();

        // Test with empty bitfield
        uint256[] memory emptyBits = new uint256[](0);
        uint256[] memory emptyBitfield = bw.createBitfield(emptyBits, 0);
        assertEq(bw.countSetBits(emptyBitfield, 10), 0);

        // Test with maxBits = 0
        uint256[] memory someBits = new uint256[](2);
        someBits[0] = 1;
        someBits[1] = 100;
        uint256[] memory bitfield = bw.createBitfield(someBits, 200);
        assertEq(bw.countSetBits(bitfield, 0), 0);

        // Test with all bits set in first 256 positions
        uint256[] memory allFirstBits = new uint256[](256);
        for (uint256 i = 0; i < 256; i++) {
            allFirstBits[i] = i;
        }
        uint256[] memory fullBitfield = bw.createBitfield(allFirstBits, 512);

        // Should count exactly 256 bits when maxBits = 256
        assertEq(bw.countSetBits(fullBitfield, 256), 256);

        // Should count exactly 100 bits when maxBits = 100
        assertEq(bw.countSetBits(fullBitfield, 100), 100);
    }

    function testBoundedCountSetBitsVsUnbounded() public {
        BitfieldWrapper bw = new BitfieldWrapper();

        string memory json =
            vm.readFile(string.concat(vm.projectRoot(), "/test/data/beefy-validator-set.json"));
        uint32 setSize = uint32(json.readUint(".validatorSetSize"));
        uint256[] memory bitSetArray = json.readUintArray(".participants");

        uint256[] memory bitfield = bw.createBitfield(bitSetArray, setSize);

        // When maxBits >= total bitfield size, bounded should equal unbounded
        assertEq(bw.countSetBits(bitfield, setSize), Bitfield.countSetBits(bitfield));
        assertEq(bw.countSetBits(bitfield, setSize + 100), Bitfield.countSetBits(bitfield));

        // Bounded count should be <= unbounded count
        assertTrue(bw.countSetBits(bitfield, setSize / 2) <= Bitfield.countSetBits(bitfield));
        assertTrue(bw.countSetBits(bitfield, 10) <= Bitfield.countSetBits(bitfield));
    }

    function testBitfieldSubsamplingWithInvalidParams() public {
        BitfieldWrapper bw = new BitfieldWrapper();

        string memory json =
            vm.readFile(string.concat(vm.projectRoot(), "/test/data/beefy-validator-set.json"));
        uint32 setSize = uint32(json.readUint(".validatorSetSize"));

        uint256 length = (setSize+255) / 256;
        uint256 N = 26;
        uint256[] memory initialBitField = new uint256[](length);
        for (uint256 i = 0; i < N; i++) {
            Bitfield.set(initialBitField, i);
        }

        uint256[] memory finalBitfield = bw.subsample(SEED, initialBitField, setSize, N);
        assertEq(Bitfield.countSetBits(finalBitfield), N);

        // Test setSize overflow
        vm.expectRevert(Bitfield.InvalidSamplingParams.selector);
        finalBitfield = bw.subsample(SEED, initialBitField, setSize * 2, N);

        // Test N overflow
        vm.expectRevert(Bitfield.InvalidSamplingParams.selector);
        finalBitfield = bw.subsample(SEED, initialBitField, setSize, N+1);
    }
}
