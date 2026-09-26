// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.34;

// Tests for the assembly in `SubstrateMerkleProof.computeMultiRoot`: agreement with the
// Solidity reference, the `position + 1` wrap at `type(uint256).max`, and memory safety.

import {Test} from "forge-std/Test.sol";
import {SubstrateMerkleProof} from "../src/utils/SubstrateMerkleProof.sol";
import {MerkleLibSubstrate} from "./utils/MerkleLib.sol";
import {ReferenceMultiRoot} from "./utils/ReferenceMultiRoot.sol";

contract SubstrateMerkleProofAssemblyTest is Test {
    uint256 constant MAX = type(uint256).max;
    uint256 constant GUARD_WORDS = 4;

    // External, so each implementation gets its own copy of the arrays.
    function computeMultiRootExternal(
        uint256[] memory positions,
        bytes32[] memory leaves,
        uint256 width,
        bytes32[] calldata siblings
    ) external pure returns (bool, bytes32) {
        return SubstrateMerkleProof.computeMultiRoot(positions, leaves, width, siblings);
    }

    function referenceMultiRootExternal(
        uint256[] memory positions,
        bytes32[] memory leaves,
        uint256 width,
        bytes32[] calldata siblings
    ) external pure returns (bool, bytes32) {
        return ReferenceMultiRoot.computeMultiRoot(positions, leaves, width, siblings);
    }

    function assertMatchesReference(
        uint256[] memory positions,
        bytes32[] memory leaves,
        uint256 width,
        bytes32[] memory siblings
    ) internal view returns (bool valid, bytes32 root) {
        (bool refValid, bytes32 refRoot) = this.referenceMultiRootExternal(
            positions, leaves, width, siblings
        );
        (valid, root) = this.computeMultiRootExternal(positions, leaves, width, siblings);
        assertEq(valid, refValid, "valid differs from reference");
        assertEq(root, refRoot, "root differs from reference");
        if (!valid) assertEq(root, bytes32(0), "invalid result must carry a zero root");
    }

    // ---- matches reference -------------------------------------------------------------

    function testFuzz_matchesReferenceOnArbitraryInput(
        uint256[] memory positions,
        bytes32[] memory leaves,
        uint256 width,
        bytes32[] memory siblings
    ) public view {
        assertMatchesReference(positions, leaves, width, siblings);
    }

    function testFuzz_matchesReferenceOnRealTrees(uint256 wSeed, uint256 shape, uint256 mutation)
        public
        view
    {
        (
            uint256[] memory positions,
            bytes32[] memory leaves,
            uint256 width,
            bytes32[] memory siblings,
            bytes32 fullRoot
        ) = realCase(wSeed, shape, mutation);
        (bool valid, bytes32 root) = assertMatchesReference(positions, leaves, width, siblings);
        if (mutation % 8 == 0) {
            assertTrue(valid && root == fullRoot, "honest multiproof rejected");
        }
    }

    function testFuzz_matchesReferenceAtAnyWidth(
        uint256 width,
        uint256 seed,
        uint8 nSeed,
        uint8 sibSeed
    ) public view {
        width = bound(width, 1, MAX);
        uint256 n = bound(nSeed, 1, 8);
        uint256[] memory positions = new uint256[](n);
        bytes32[] memory leaves = new bytes32[](n);
        for (uint256 i = 0; i < n; i++) {
            uint256 r = uint256(keccak256(abi.encode(seed, i)));
            // 1 in 4 near the top of the range.
            positions[i] = r % 4 == 0 ? width - 1 - (r >> 8) % 4 % width : r % width;
            leaves[i] = keccak256(abi.encode(seed, "leaf", i));
        }
        MerkleLibSubstrate.sort(positions);
        assertMatchesReference(positions, leaves, width, junk(bytes32(seed), sibSeed));
    }

    // ---- type(uint256).max --------------------------------------------------------------

    // `position + 1` wraps to 0 at MAX.
    function testMaxPositionIsRejected() public view {
        bytes32[] memory none = new bytes32[](0);
        uint256[] memory widths = new uint256[](3);
        (widths[0], widths[1], widths[2]) = (MAX, MAX - 1, 600);

        for (uint256 w = 0; w < widths.length; w++) {
            assertRejected(one(MAX), widths[w], none, "max position alone");
            assertRejected(two(MAX, 0), widths[w], none, "max position first");
            assertRejected(two(0, MAX), widths[w], none, "max position last");
            assertRejected(two(MAX, MAX), widths[w], none, "max position repeated");
        }
    }

    function testTopEdgeAtMaxWidth() public view {
        bytes32 leaf = keccak256("leaf");
        // MAX is odd, so `MAX - 1` is promoted.
        uint256[3] memory tops = [MAX - 1, MAX - 2, MAX - 3];
        bytes32[] memory leaves = leafArray(leaf, 1);
        for (uint256 t = 0; t < tops.length; t++) {
            bytes32[] memory path = junk(leaf, pathLength(tops[t], MAX));
            (bool valid, bytes32 root) = assertMatchesReference(one(tops[t]), leaves, MAX, path);
            assertTrue(valid, "canonical path rejected at max width");
            (, bytes32 single) = this.computeRootExternal(leaves[0], tops[t], MAX, path);
            assertEq(root, single, "multiproof != computeRoot at max width");
        }
        // `MAX - 3` pairs with `MAX - 2`; `MAX - 1` is promoted.
        uint256[] memory three = new uint256[](3);
        (three[0], three[1], three[2]) = (MAX - 3, MAX - 2, MAX - 1);
        for (uint256 len = 250; len <= 260; len++) {
            assertMatchesReference(three, leafArray(leaf, 3), MAX, junk(leaf, len));
        }
    }

    // ---- memory safety -----------------------------------------------------------------

    function testFuzz_leavesSurroundingMemoryIntact(uint256 wSeed, uint256 shape, uint256 mutation)
        public
        view
    {
        (
            uint256[] memory positions,
            bytes32[] memory leaves,
            uint256 width,
            bytes32[] memory siblings,
        ) = realCase(wSeed, shape, mutation);
        this.computeMultiRootGuarded(positions, leaves, width, siblings);
    }

    function testFuzz_leavesSurroundingMemoryIntactOnArbitraryInput(
        uint256[] memory positions,
        bytes32[] memory leaves,
        uint256 width,
        bytes32[] memory siblings
    ) public view {
        this.computeMultiRootGuarded(positions, leaves, width, siblings);
    }

    /// Runs on copies laid out as `guard | positions | guard | leaves | guard`.
    function computeMultiRootGuarded(
        uint256[] memory positionsIn,
        bytes32[] memory leavesIn,
        uint256 width,
        bytes32[] calldata siblings
    ) external pure {
        bytes32[] memory before = guard(1);
        uint256[] memory positions = new uint256[](positionsIn.length);
        bytes32[] memory middle = guard(2);
        bytes32[] memory leaves = new bytes32[](leavesIn.length);
        bytes32[] memory after_ = guard(3);
        for (uint256 i = 0; i < positions.length; i++) {
            positions[i] = positionsIn[i];
        }
        for (uint256 i = 0; i < leaves.length; i++) {
            leaves[i] = leavesIn[i];
        }

        uint256 fmp;
        uint256 zeroSlot;
        assembly {
            fmp := mload(0x40)
        }

        SubstrateMerkleProof.computeMultiRoot(positions, leaves, width, siblings);

        uint256 fmpAfter;
        assembly {
            fmpAfter := mload(0x40)
            zeroSlot := mload(0x60)
        }
        assertEq(fmpAfter, fmp, "free memory pointer moved");
        assertEq(zeroSlot, 0, "zero slot written");
        assertEq(positions.length, positionsIn.length, "positions length word written");
        assertEq(leaves.length, leavesIn.length, "leaves length word written");
        assertGuard(before, 1);
        assertGuard(middle, 2);
        assertGuard(after_, 3);
    }

    function guard(uint256 tag) internal pure returns (bytes32[] memory g) {
        g = new bytes32[](GUARD_WORDS);
        for (uint256 i = 0; i < GUARD_WORDS; i++) {
            g[i] = keccak256(abi.encode("guard", tag, i));
        }
    }

    function assertGuard(bytes32[] memory g, uint256 tag) internal pure {
        assertEq(g.length, GUARD_WORDS, "guard length word written");
        for (uint256 i = 0; i < GUARD_WORDS; i++) {
            assertEq(g[i], keccak256(abi.encode("guard", tag, i)), "guard word written");
        }
    }

    // ---- helpers ------------------------------------------------------------------------

    function computeRootExternal(
        bytes32 leaf,
        uint256 position,
        uint256 width,
        bytes32[] calldata proof
    ) external pure returns (bool, bytes32) {
        return SubstrateMerkleProof.computeRoot(leaf, position, width, proof);
    }

    /// Multiproof over a real tree of width 1..700. Unless `mutation % 8 == 0`, one mutation
    /// makes it invalid.
    function realCase(uint256 wSeed, uint256 shape, uint256 mutation)
        internal
        pure
        returns (
            uint256[] memory positions,
            bytes32[] memory leaves,
            uint256 width,
            bytes32[] memory siblings,
            bytes32 fullRoot
        )
    {
        width = bound(wSeed, 1, 700);
        bytes32[][] memory L;
        (L, fullRoot) = MerkleLibSubstrate.buildLevels(MerkleLibSubstrate.genLeaves(width));
        positions = sample(width, shape);
        leaves = new bytes32[](positions.length);
        for (uint256 i = 0; i < positions.length; i++) {
            leaves[i] = L[0][positions[i]];
        }
        siblings = siblingsFor(L, positions);

        uint256 n = positions.length;
        uint256 k = mutation >> 8;
        uint256 kind = mutation % 8;
        if (kind == 1 && siblings.length > 0) {
            assembly {
                mstore(siblings, sub(mload(siblings), 1))
            }
        } else if (kind == 2) {
            siblings = append(siblings, keccak256(abi.encode(mutation)));
        } else if (kind == 3 && siblings.length > 0) {
            siblings[k % siblings.length] = keccak256(abi.encode(mutation));
        } else if (kind == 4 && n > 1) {
            uint256 i = k % (n - 1);
            positions[i + 1] = positions[i];
        } else if (kind == 5) {
            positions[n - 1] = width + k % 3;
        } else if (kind == 6 && n > 1) {
            uint256 i = k % n;
            uint256 j = (i + 1 + (k >> 32) % (n - 1)) % n;
            (leaves[i], leaves[j]) = (leaves[j], leaves[i]);
        } else if (kind == 7) {
            positions[k % n] = MAX - k % 2;
        }
    }

    function sample(uint256 width, uint256 shape) internal pure returns (uint256[] memory out) {
        out = new uint256[](width);
        uint256 omit = (shape >> 2) % width;
        uint256 n;
        for (uint256 p = 0; p < width; p++) {
            bool keep;
            if (shape % 3 == 1) keep = true;
            else if (shape % 3 == 2) keep = p != omit || width == 1;
            else keep = uint256(keccak256(abi.encode(shape, p))) % 5 == 0;
            if (keep) out[n++] = p;
        }
        if (n == 0) out[n++] = shape % width;
        assembly {
            mstore(out, n)
        }
    }

    /// Per layer, each known node's sibling unless that sibling is known too.
    function siblingsFor(bytes32[][] memory L, uint256[] memory positions)
        internal
        pure
        returns (bytes32[] memory out)
    {
        out = new bytes32[](positions.length * L.length);
        uint256 k;
        bool[] memory known = new bool[](L[0].length);
        for (uint256 i = 0; i < positions.length; i++) {
            known[positions[i]] = true;
        }
        for (uint256 l = 0; l + 1 < L.length; l++) {
            uint256 w = L[l].length;
            bool[] memory up = new bool[](L[l + 1].length);
            for (uint256 p = 0; p < w; p++) {
                if (!known[p]) continue;
                up[p / 2] = true;
                if (p == w - 1 && w % 2 == 1) continue;
                if (!known[p ^ 1]) out[k++] = L[l][p ^ 1];
            }
            known = up;
        }
        assembly {
            mstore(out, k)
        }
    }

    function pathLength(uint256 position, uint256 width) internal pure returns (uint256 len) {
        while (width > 1) {
            if (!(position + 1 == width && width & 1 == 1)) len++;
            position >>= 1;
            width = ((width - 1) >> 1) + 1;
        }
    }

    function assertRejected(
        uint256[] memory positions,
        uint256 width,
        bytes32[] memory siblings,
        string memory reason
    ) internal view {
        (bool valid,) = assertMatchesReference(
            positions, leafArray(keccak256("leaf"), positions.length), width, siblings
        );
        assertFalse(valid, reason);
    }

    function junk(bytes32 seed, uint256 n) internal pure returns (bytes32[] memory out) {
        out = new bytes32[](n);
        for (uint256 i = 0; i < n; i++) {
            out[i] = keccak256(abi.encode(seed, "sibling", i));
        }
    }

    function append(bytes32[] memory a, bytes32 x) internal pure returns (bytes32[] memory b) {
        b = new bytes32[](a.length + 1);
        for (uint256 i = 0; i < a.length; i++) {
            b[i] = a[i];
        }
        b[a.length] = x;
    }

    function leafArray(bytes32 leaf, uint256 n) internal pure returns (bytes32[] memory out) {
        out = new bytes32[](n);
        for (uint256 i = 0; i < n; i++) {
            out[i] = keccak256(abi.encode(leaf, i));
        }
    }

    function one(uint256 a) internal pure returns (uint256[] memory out) {
        out = new uint256[](1);
        out[0] = a;
    }

    function two(uint256 a, uint256 b) internal pure returns (uint256[] memory out) {
        out = new uint256[](2);
        (out[0], out[1]) = (a, b);
    }
}
