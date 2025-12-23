// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import {Test} from "forge-std/Test.sol";
import {console} from "forge-std/console.sol";
import {BitfieldWrapper} from "./mocks/BitfieldWrapper.sol";
import {Bitfield} from "../src/utils/Bitfield.sol";

import {stdJson} from "forge-std/StdJson.sol";

contract BitfieldTest is Test {
    using stdJson for string;

    uint256 public constant SEED =
        2_954_466_101_346_023_252_933_346_884_990_731_083_400_112_195_551_952_331_583_346_342_070_284_928_184;

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

        uint256 length = (setSize + 255) / 256;
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
        finalBitfield = bw.subsample(SEED, initialBitField, setSize, N + 1);
    }

    function testValidatePaddingWithLength512() public {
        BitfieldWrapper bw = new BitfieldWrapper();
        uint256 length = 512;

        // Create a bitfield with bits set at various positions
        uint256[] memory bitsToSet = new uint256[](5);
        bitsToSet[0] = 0; // First bit in first container
        bitsToSet[1] = 127; // Middle of first container
        bitsToSet[2] = 255; // Last valid bit in first container
        bitsToSet[3] = 256; // First bit in second container
        bitsToSet[4] = 511; // Last valid bit in second container

        uint256[] memory bitfield = bw.createBitfield(bitsToSet, length);

        // Should pass: no padding bits set (512 is multiple of 256)
        bw.validatePadding(bitfield, length);
    }

    function testValidatePaddingWithLength512AllBits() public {
        BitfieldWrapper bw = new BitfieldWrapper();
        uint256 length = 512;

        // Create a bitfield with all bits set
        uint256[] memory bitsToSet = new uint256[](512);
        for (uint256 i = 0; i < 512; i++) {
            bitsToSet[i] = i;
        }

        uint256[] memory bitfield = bw.createBitfield(bitsToSet, length);

        // Should pass: no padding bits exist when length is multiple of 256
        bw.validatePadding(bitfield, length);
    }

    function testValidatePaddingWithLength600() public {
        BitfieldWrapper bw = new BitfieldWrapper();
        uint256 length = 600;

        // Create a bitfield with bits set strategically
        uint256[] memory bitsToSet = new uint256[](6);
        bitsToSet[0] = 0; // First bit
        bitsToSet[1] = 255; // Last valid bit in first container
        bitsToSet[2] = 256; // First bit in second container
        bitsToSet[3] = 511; // Last valid bit in second container
        bitsToSet[4] = 512; // First bit in third container
        bitsToSet[5] = 599; // Last valid bit (599 is the highest valid index)

        uint256[] memory bitfield = bw.createBitfield(bitsToSet, length);

        // Should pass: no padding bits set
        bw.validatePadding(bitfield, length);
    }

    function testValidatePaddingWithBitstoSetOverflow() public {
        BitfieldWrapper bw = new BitfieldWrapper();
        uint256 length = 600;

        // bitsToSet contains out-of-bounds index
        uint256[] memory bitsToSet = new uint256[](3);
        bitsToSet[0] = 0;
        bitsToSet[1] = 300;
        bitsToSet[2] = 700;

        uint256[] memory bitfield = bw.createBitfield(bitsToSet, length);

        // Should revert: padding bits are set
        vm.expectRevert(Bitfield.InvalidBitfieldPadding.selector);
        bw.validatePadding(bitfield, length);
    }

    function testValidatePaddingWithLength600PaddingBitSet() public {
        BitfieldWrapper bw = new BitfieldWrapper();
        uint256 length = 600;

        // Create a valid bitfield first
        uint256[] memory bitsToSet = new uint256[](6);
        bitsToSet[0] = 0;
        bitsToSet[1] = 255;
        bitsToSet[2] = 256;
        bitsToSet[3] = 511;
        bitsToSet[4] = 512;
        bitsToSet[5] = 599;

        uint256[] memory bitfield = bw.createBitfield(bitsToSet, length);

        // Now set a padding bit (bit 600 in global indexing, which is bit 88 in container 2)
        // Container index: 600 / 256 = 2 (third container)
        // Bit index: 600 % 256 = 88
        bitfield[2] |= (uint256(1) << 88);

        // Should revert: padding bit is set
        vm.expectRevert(Bitfield.InvalidBitfieldPadding.selector);
        bw.validatePadding(bitfield, length);
    }

    function testValidatePaddingWithLength600HighPaddingBitSet() public {
        BitfieldWrapper bw = new BitfieldWrapper();
        uint256 length = 600;

        // Create a valid bitfield
        uint256[] memory bitsToSet = new uint256[](3);
        bitsToSet[0] = 0;
        bitsToSet[1] = 300;
        bitsToSet[2] = 599;

        uint256[] memory bitfield = bw.createBitfield(bitsToSet, length);

        // Set a high padding bit (bit 767 in global, which is bit 255 in container 2)
        // This is the highest bit in the last container that could be padding
        bitfield[2] |= (uint256(1) << 255);

        // Should revert: padding bit is set
        vm.expectRevert(Bitfield.InvalidBitfieldPadding.selector);
        bw.validatePadding(bitfield, length);
    }

    function testValidatePaddingWithLength600MultiplePaddingBits() public {
        BitfieldWrapper bw = new BitfieldWrapper();
        uint256 length = 600;

        // Create a valid bitfield
        uint256[] memory bitsToSet = new uint256[](2);
        bitsToSet[0] = 0;
        bitsToSet[1] = 599;

        uint256[] memory bitfield = bw.createBitfield(bitsToSet, length);

        // Set multiple padding bits
        bitfield[2] |= (uint256(1) << 88); // First padding bit
        bitfield[2] |= (uint256(1) << 100); // Another padding bit
        bitfield[2] |= (uint256(1) << 255); // Highest padding bit

        // Should revert: padding bits are set
        vm.expectRevert(Bitfield.InvalidBitfieldPadding.selector);
        bw.validatePadding(bitfield, length);
    }

    function testFuzzValidatePaddingWithRandomLengths(uint256 length) public {
        // Bound the length to reasonable values (avoid extremely large lengths)
        length = bound(length, 1, 2048);

        BitfieldWrapper bw = new BitfieldWrapper();

        // Create a bitfield with random valid bits only
        uint256 maxBits = length < 10 ? length : 10;
        uint256[] memory bitsToSet = new uint256[](maxBits);
        for (uint256 i = 0; i < maxBits; i++) {
            uint256 randomBit = uint256(keccak256(abi.encodePacked(length, i))) % length;
            bitsToSet[i] = randomBit;
        }

        uint256[] memory bitfield = bw.createBitfield(bitsToSet, length);

        // Should always pass: bitfield created with valid bits only
        bw.validatePadding(bitfield, length);
    }

    function testFuzzValidatePaddingDetectsPaddingBits(uint256 length, uint256 paddingBitOffset)
        public
    {
        // Bound the length to avoid edge cases
        length = bound(length, 256, 2048);

        BitfieldWrapper bw = new BitfieldWrapper();

        // Create a valid bitfield
        uint256[] memory bitsToSet = new uint256[](3);
        bitsToSet[0] = 0;
        bitsToSet[1] = length / 2;
        bitsToSet[2] = length > 1 ? length - 1 : 0;

        uint256[] memory bitfield = bw.createBitfield(bitsToSet, length);

        // Only inject padding bits if length is not a multiple of 256
        uint256 validBitsInLast = length % 256;
        if (validBitsInLast != 0) {
            uint256 containerLen = (length + 255) / 256;
            uint256 paddingCount = 256 - validBitsInLast;
            paddingBitOffset = bound(paddingBitOffset, 0, paddingCount - 1);
            uint256 paddingBitGlobal =
                (containerLen - 1) * 256 + validBitsInLast + paddingBitOffset;

            // Set the padding bit
            bitfield[containerLen - 1] |= (uint256(1) << (paddingBitGlobal & 0xff));

            // Should revert: padding bit is set
            vm.expectRevert(Bitfield.InvalidBitfieldPadding.selector);
            bw.validatePadding(bitfield, length);
        } else {
            // No padding exists; validatePadding should pass
            bw.validatePadding(bitfield, length);
        }
    }

    function testFuzzValidatePaddingWithRandomBitfields(uint256 seed, uint256 length) public {
        // Bound inputs
        length = bound(length, 1, 1024);

        BitfieldWrapper bw = new BitfieldWrapper();

        // Create a bitfield with pseudo-random bits
        uint256 containerLen = (length + 255) / 256;
        uint256[] memory bitfield = new uint256[](containerLen);

        // Fill bitfield with random bits, but respect the length boundary
        for (uint256 i = 0; i < containerLen; i++) {
            uint256 element = i;
            uint256 maxBitInElement = (element == containerLen - 1) ? (length % 256) : 256;

            // Only set bits that are valid for this container
            for (uint256 j = 0; j < maxBitInElement; j++) {
                uint256 bitHash = uint256(keccak256(abi.encodePacked(seed, element, j)));
                if ((bitHash & 1) == 1) {
                    bitfield[element] |= (uint256(1) << j);
                }
            }
        }

        // Should always pass: we only set valid bits
        bw.validatePadding(bitfield, length);
    }
}
