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
