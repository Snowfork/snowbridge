// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.34;

import {Test} from "forge-std/Test.sol";
import {Bitfield} from "../src/utils/Bitfield.sol";
import {SpecSubsample} from "./utils/SpecSubsample.sol";

/// @dev `subsample` (assembly) and `toIndices` must match the previous Solidity versions bit for
/// bit: relayers and the Fiat-Shamir fixtures depend on the exact sample. `subsample` is also
/// checked against `SpecSubsample`, a model written from the sampling rule, so a flaw shared by
/// the library and the old loop shows up as a disagreement.
/// forge-config: default.fuzz.runs = 4096
/// forge-config: production.fuzz.runs = 4096
contract BitfieldEquivalenceTest is Test {
    /// The loop deployed on mainnet before the assembly (67b9407e^), kept verbatim, on the
    /// library's own `makeIndex`, `isSet` and `set`.
    function referenceSubsample(uint256 seed, uint256[] memory prior, uint256 size, uint256 n)
        internal
        pure
        returns (uint256[] memory out)
    {
        out = new uint256[](prior.length);
        uint256 found = 0;

        for (uint256 i = 0; found < n;) {
            uint256 index = Bitfield.makeIndex(seed, i, size);

            // require randomly selected bit to be set in priorBitfield and not yet set in bitfield
            if (!Bitfield.isSet(prior, index) || Bitfield.isSet(out, index)) {
                unchecked {
                    i++;
                }
                continue;
            }

            Bitfield.set(out, index);

            unchecked {
                found++;
                i++;
            }
        }
    }

    /// The library, the pre-assembly loop and the model all give the same sample.
    function assertSubsampleAgrees(uint256 seed, uint256[] memory prior, uint256 size, uint256 n)
        internal
        pure
        returns (uint256[] memory got)
    {
        uint256[] memory want = referenceSubsample(seed, prior, size, n);
        (bool ok, uint256[] memory model) = SpecSubsample.subsample(seed, prior, size, n);
        assertTrue(ok, "model rejected valid parameters");
        assertEq(model, want, "model and reference disagree");
        got = Bitfield.subsample(seed, prior, size, n);
        assertEq(got, want, "library and reference disagree");
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

        uint256[] memory got = assertSubsampleAgrees(seed, prior, size, n);
        assertEq(Bitfield.toIndices(got, n), referenceToIndices(got));
    }

    /// Properties checked without the reference, so a bug shared by both is still caught: the
    /// sample has exactly `n` distinct bits, all claimed and all below `size`, even when the
    /// claim carries padding bits past `size`.
    function testFuzzSubsampleProperties(
        uint256 seed,
        uint256 entropy,
        uint16 rawSize,
        uint8 rawN,
        uint256 padding
    ) public pure {
        uint256 size = bound(rawSize, 1, 1000);
        uint256[] memory prior = claim(entropy, size);
        uint256 set = Bitfield.countSetBits(prior, size);
        vm.assume(set > 0);
        uint256 n = bound(rawN, 0, set < 128 ? set : 128);
        if (size % 256 != 0) {
            prior[prior.length - 1] |= padding << (size % 256);
        }

        uint256[] memory got = assertSubsampleAgrees(seed, prior, size, n);
        assertEq(got.length, prior.length, "length");
        assertEq(Bitfield.countSetBits(got), n, "count");
        for (uint256 w = 0; w < got.length; w++) {
            assertEq(got[w] & ~prior[w], 0, "unclaimed bit");
        }
        assertEq(Bitfield.countSetBits(got, size), n, "bit past size");
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

        assertSubsampleAgrees(seed, prior, size, n);
    }

    function subsampleExternal(uint256 seed, uint256[] memory prior, uint256 size, uint256 n)
        external
        pure
        returns (uint256[] memory)
    {
        return Bitfield.subsample(seed, prior, size, n);
    }

    /// Wrong container length or too few claims: the library reverts exactly when the model
    /// reports invalid parameters, and otherwise they agree.
    function testFuzzSubsampleRejectsTheSameParametersAsTheModel(
        uint256 seed,
        uint256 entropy,
        uint16 rawSize,
        uint8 rawLength,
        uint16 rawN
    ) public view {
        uint256 size = bound(rawSize, 0, 1000);
        uint256 length = bound(rawLength, 0, 5);
        uint256[] memory prior = new uint256[](length);
        for (uint256 w = 0; w < length; w++) {
            prior[w] = uint256(keccak256(abi.encode(entropy, w)));
        }
        uint256 n = bound(rawN, 0, size + 2);

        (bool ok, uint256[] memory model) = SpecSubsample.subsample(seed, prior, size, n);
        try this.subsampleExternal(seed, prior, size, n) returns (uint256[] memory got) {
            assertTrue(ok, "library accepted parameters the model rejects");
            assertEq(got, model, "library and model disagree");
        } catch (bytes memory reason) {
            assertFalse(ok, "library rejected parameters the model accepts");
            assertEq(bytes4(reason), Bitfield.InvalidSamplingParams.selector, "revert reason");
        }
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

    /// Across several words, `toIndices` succeeds exactly when `n` is the number of set bits.
    function testFuzzToIndicesRejectsEveryWrongCount(
        uint256 a,
        uint256 b,
        uint256 c,
        uint256 delta,
        bool above
    ) public {
        uint256[] memory bf = new uint256[](3);
        (bf[0], bf[1], bf[2]) = (a, b, c);
        uint256 count = Bitfield.countSetBits(bf);
        assertEq(this.toIndicesExternal(bf, count), referenceToIndices(bf));
        delta = bound(delta, 1, 8);
        if (!above && count < delta) return;
        vm.expectRevert(Bitfield.InvalidSamplingParams.selector);
        this.toIndicesExternal(bf, above ? count + delta : count - delta);
    }

    /// On success, `toIndices` writes only its own output; with more set bits than `n` it reverts.
    function testFuzzToIndicesLeavesSurroundingMemoryIntact(uint256 a, uint256 b, uint8 rawN)
        public
        view
    {
        uint256[] memory bf = new uint256[](2);
        (bf[0], bf[1]) = (a, b);
        uint256 n = bound(rawN, 0, Bitfield.countSetBits(bf));
        try this.toIndicesGuarded(bf, n) {}
        catch (bytes memory reason) {
            // More set bits than `n` is a clean revert; anything else is a failed guard.
            assertEq(bytes4(reason), Bitfield.InvalidSamplingParams.selector, "guard failed");
            assertLt(n, Bitfield.countSetBits(bf), "valid count reverted");
        }
    }

    function toIndicesGuarded(uint256[] memory bf, uint256 n) external pure {
        uint256 sentinel = uint256(keccak256("sentinel"));
        uint256 free;
        assembly {
            free := mload(0x40)
            for { let o := 0 } lt(o, 0x2400) { o := add(o, 0x20) } {
                mstore(add(free, o), sentinel)
            }
        }
        uint256[] memory out = Bitfield.toIndices(bf, n);
        bool intact = true;
        assembly {
            for { let p := add(out, shl(5, add(mload(out), 1))) } lt(p, add(free, 0x2400)) {
                p := add(p, 0x20)
            } {
                if iszero(eq(mload(p), sentinel)) { intact := false }
            }
        }
        require(intact, "memory past the output changed");
    }
}
