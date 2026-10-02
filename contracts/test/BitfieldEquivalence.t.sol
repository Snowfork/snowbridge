// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.34;

import {Test} from "forge-std/Test.sol";
import {Bitfield} from "../src/utils/Bitfield.sol";

/// @dev `subsample` (assembly) and `toIndices` must match the previous Solidity versions bit for
/// bit: relayers and the Fiat-Shamir fixtures depend on the exact sample.
contract BitfieldEquivalenceTest is Test {
    function referenceSubsample(uint256 seed, uint256[] memory prior, uint256 size, uint256 n)
        internal
        pure
        returns (uint256[] memory out)
    {
        out = new uint256[](prior.length);
        uint256 found = 0;
        for (uint256 i = 0; found < n; i++) {
            uint256 index = uint256(keccak256(abi.encode(seed, i))) % size;
            uint256 bit = uint256(1) << (index & 0xff);
            if (prior[index >> 8] & bit == 0 || out[index >> 8] & bit != 0) {
                continue;
            }
            out[index >> 8] |= bit;
            found++;
        }
    }

    function referenceToIndices(uint256[] memory self) internal pure returns (uint256[] memory) {
        uint256 count;
        for (uint256 i = 0; i < self.length * 256; i++) {
            if (self[i >> 8] & (uint256(1) << (i & 0xff)) != 0) count++;
        }
        uint256[] memory indices = new uint256[](count);
        uint256 k;
        for (uint256 i = 0; i < self.length * 256; i++) {
            if (self[i >> 8] & (uint256(1) << (i & 0xff)) != 0) indices[k++] = i;
        }
        return indices;
    }

    /// Random claim over `size` validators with every bit below `size` set with ~2/3 chance.
    function claim(uint256 entropy, uint256 size) internal pure returns (uint256[] memory bf) {
        bf = new uint256[]((size + 255) / 256);
        for (uint256 i = 0; i < size; i++) {
            if (uint256(keccak256(abi.encode(entropy, i))) % 3 != 0) {
                bf[i >> 8] |= uint256(1) << (i & 0xff);
            }
        }
    }

    function testFuzzSubsampleMatchesReference(
        uint256 seed,
        uint256 entropy,
        uint16 rawSize,
        uint8 rawN
    ) public pure {
        uint256 size = bound(rawSize, 1, 1000);
        uint256[] memory prior = claim(entropy, size);
        uint256 set = Bitfield.countSetBits(prior, size);
        vm.assume(set > 0);
        uint256 n = bound(rawN, 0, set < 128 ? set : 128);

        uint256[] memory got = Bitfield.subsample(seed, prior, size, n);
        uint256[] memory want = referenceSubsample(seed, prior, size, n);
        assertEq(got, want);
        assertEq(Bitfield.toIndices(got, n), referenceToIndices(want));
    }

    /// Claims barely larger than the sample (n + 0..3 bits), so the rejection loop runs long
    /// and the last picks have few candidates left.
    function testFuzzSubsampleSparsePriorMatchesReference(
        uint256 seed,
        uint256 entropy,
        uint16 rawSize,
        uint8 rawN,
        uint8 rawExtra
    ) public pure {
        uint256 size = bound(rawSize, 1, 1000);
        uint256 n = bound(rawN, 1, size < 128 ? size : 128);
        uint256 count = n + bound(rawExtra, 0, 3);
        if (count > size) count = size;
        uint256[] memory prior = new uint256[]((size + 255) / 256);
        for (uint256 j = 0; j < count; j++) {
            uint256 i = uint256(keccak256(abi.encode(entropy, j))) % size;
            while (prior[i >> 8] & (uint256(1) << (i & 0xff)) != 0) {
                i = (i + 1) % size;
            }
            prior[i >> 8] |= uint256(1) << (i & 0xff);
        }

        uint256[] memory got = Bitfield.subsample(seed, prior, size, n);
        assertEq(got, referenceSubsample(seed, prior, size, n));
    }

    /// The assembly writes only scratch space and its own output: the claimed bitfield and the
    /// memory past the output are untouched.
    function testFuzzSubsampleLeavesSurroundingMemoryIntact(
        uint256 seed,
        uint256 entropy,
        uint16 rawSize,
        uint8 rawN
    ) public pure {
        uint256 size = bound(rawSize, 1, 1000);
        uint256[] memory prior = claim(entropy, size);
        uint256 set = Bitfield.countSetBits(prior, size);
        vm.assume(set > 0);
        uint256 n = bound(rawN, 0, set < 128 ? set : 128);
        uint256[] memory before = new uint256[](prior.length);
        for (uint256 i = 0; i < prior.length; i++) {
            before[i] = prior[i];
        }

        uint256 sentinel = uint256(keccak256("sentinel"));
        uint256 free;
        assembly {
            free := mload(0x40)
            for { let o := 0 } lt(o, 0x400) { o := add(o, 0x20) } {
                mstore(add(free, o), sentinel)
            }
        }
        uint256[] memory out = Bitfield.subsample(seed, prior, size, n);

        // Check before anything else allocates: assertion messages land past the output.
        uint256 outPtr;
        bool intact = true;
        assembly {
            outPtr := out
            for { let p := add(out, shl(5, add(mload(out), 1))) } lt(p, add(free, 0x400)) {
                p := add(p, 0x20)
            } {
                if iszero(eq(mload(p), sentinel)) { intact := false }
            }
        }
        assertEq(outPtr, free, "output not allocated at the free pointer");
        assertTrue(intact, "memory past the output changed");
        assertEq(prior, before, "claimed bitfield changed");
    }

    function testFuzzToIndicesMatchesReference(uint256 a, uint256 b, uint256 c) public pure {
        uint256[] memory bf = new uint256[](3);
        bf[0] = a;
        bf[1] = b;
        bf[2] = c;
        uint256[] memory want = referenceToIndices(bf);
        assertEq(Bitfield.toIndices(bf, want.length), want);
    }

    function testToIndicesHandlesTopAndBottomBits() public pure {
        uint256[] memory bf = new uint256[](2);
        bf[0] = 1 | (uint256(1) << 255);
        bf[1] = 1 << 7;
        uint256[] memory want = new uint256[](3);
        want[0] = 0;
        want[1] = 255;
        want[2] = 263;
        assertEq(Bitfield.toIndices(bf, 3), want);
    }

    function toIndicesExternal(uint256[] memory bf, uint256 n)
        external
        pure
        returns (uint256[] memory)
    {
        return Bitfield.toIndices(bf, n);
    }

    function testToIndicesRejectsWrongCount() public {
        uint256[] memory bf = new uint256[](1);
        bf[0] = 0x7; // three bits
        vm.expectRevert(Bitfield.InvalidSamplingParams.selector);
        this.toIndicesExternal(bf, 2);
        vm.expectRevert(Bitfield.InvalidSamplingParams.selector);
        this.toIndicesExternal(bf, 4);
    }
}
