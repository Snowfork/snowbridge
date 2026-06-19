// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.34;

// Adversarial property/fuzz tests for the geometry-walking SubstrateMerkleProof verifier
// (src/utils/SubstrateMerkleProof.sol). These generalize the fixed-input regression cases in
// SubstrateMerkleProofAliasing.t.sol to random widths, positions and proof mutations, and pin the
// safety properties that matter for a merkle verifier on a security boundary:
//
//   1. Round-trip          — every canonical leaf verifies and reconstructs the reference root.
//   2. Position binding     — a canonical proof for index p verifies at claimed index X iff X == p
//                             (this is THE anti-aliasing property the fix introduces).
//   3. Mutation rejection   — flipping a sibling, or dropping/appending an element, fails to verify.
//   4. Totality + bounded   — verify/computeRoot never revert on arbitrary input and do bounded
//                             work (the loop provably terminates: width strictly decreases, so the
//                             "unchecked arithmetic / infinite loop" concern is covered as a test).
//   5. Boundaries           — position >= width, single-leaf, and empty-proof edge cases.
//
// The reference tree (substrate's lone-trailing-node PROMOTION rule) is built by MerkleLibSubstrate
// (test/utils/MerkleLib.sol), the same generator the regression suite uses.
//
// Run: forge test --match-path test/SubstrateMerkleProofFuzz.t.sol -vv
//   (bump iterations with e.g. FOUNDRY_FUZZ_RUNS=5000)

import {Test} from "forge-std/Test.sol";
import {SubstrateMerkleProof} from "../src/utils/SubstrateMerkleProof.sol";
import {MerkleLibSubstrate} from "./utils/MerkleLib.sol";

/// memory->calldata bridge so the fuzzer exercises the REAL on-chain library code.
contract Harness {
    function verify(bytes32 root, bytes32 leaf, uint256 pos, uint256 width, bytes32[] calldata proof)
        external
        pure
        returns (bool)
    {
        return SubstrateMerkleProof.verify(root, leaf, pos, width, proof);
    }

    function computeRoot(bytes32 leaf, uint256 pos, uint256 width, bytes32[] calldata proof)
        external
        pure
        returns (bool valid, bytes32 root)
    {
        return SubstrateMerkleProof.computeRoot(leaf, pos, width, proof);
    }
}

contract SubstrateMerkleProofFuzzTest is Test {
    Harness h;

    // Cap tree-building widths so each fuzz run stays cheap; full-range widths are exercised
    // (without building a tree) in the totality property below.
    uint256 constant MAX_WIDTH = 600;

    function setUp() public {
        h = new Harness();
    }

    // 1. ROUND-TRIP: every leaf's canonical proof verifies at its own index and reconstructs the
    //    reference root, across arbitrary widths (power-of-two, odd, and promotion-heavy).
    function testFuzz_roundTrip(uint256 nSeed, uint256 pSeed) public view {
        uint256 n = bound(nSeed, 1, MAX_WIDTH);
        uint256 p = bound(pSeed, 0, n - 1);

        bytes32[] memory leaves = MerkleLibSubstrate.genLeaves(n);
        (bytes32[][] memory L, bytes32 root) = MerkleLibSubstrate.buildLevels(leaves);
        bytes32[] memory proof = MerkleLibSubstrate.proofFromLevels(L, p);

        assertTrue(h.verify(root, leaves[p], p, n, proof), "canonical proof must verify at its index");

        (bool valid, bytes32 got) = h.computeRoot(leaves[p], p, n, proof);
        assertTrue(valid, "canonical proof must be structurally valid");
        assertEq(got, root, "computeRoot must reconstruct the reference root");
    }

    // 2. POSITION BINDING (the fix): a canonical proof for index p verifies at claimed index X iff
    //    X == p. This is the anti-aliasing guarantee — a short (lone-promoted) proof can no longer
    //    be replayed at any other index. The leaf is p's leaf, modelling a validator claiming a
    //    different slot, exactly as the BeefyClient forge does.
    function testFuzz_positionBinding(uint256 nSeed, uint256 pSeed, uint256 xSeed) public view {
        uint256 n = bound(nSeed, 1, MAX_WIDTH);
        uint256 p = bound(pSeed, 0, n - 1);
        uint256 x = bound(xSeed, 0, n - 1);

        bytes32[] memory leaves = MerkleLibSubstrate.genLeaves(n);
        (bytes32[][] memory L, bytes32 root) = MerkleLibSubstrate.buildLevels(leaves);
        bytes32[] memory proof = MerkleLibSubstrate.proofFromLevels(L, p);

        assertEq(
            h.verify(root, leaves[p], x, n, proof),
            x == p,
            "proof for p must verify at claimed index X iff X == p"
        );
    }

    // 3a. MUTATION — flipping any single byte of a sibling breaks verification.
    function testFuzz_flippedSiblingRejected(uint256 nSeed, uint256 pSeed, uint256 jSeed, uint256 bitSeed)
        public
        view
    {
        uint256 n = bound(nSeed, 2, MAX_WIDTH);
        uint256 p = bound(pSeed, 0, n - 1);

        bytes32[] memory leaves = MerkleLibSubstrate.genLeaves(n);
        (bytes32[][] memory L, bytes32 root) = MerkleLibSubstrate.buildLevels(leaves);
        bytes32[] memory proof = MerkleLibSubstrate.proofFromLevels(L, p);
        vm.assume(proof.length > 0); // some promotion-heavy indices have empty proofs

        uint256 j = bound(jSeed, 0, proof.length - 1);
        proof[j] = proof[j] ^ bytes32(uint256(1) << (bitSeed & 0xff)); // flip one bit

        assertFalse(h.verify(root, leaves[p], p, n, proof), "tampered sibling must not verify");
    }

    // 3b. MUTATION — wrong-length proofs (one short, one long) are rejected as structurally invalid.
    function testFuzz_wrongLengthRejected(uint256 nSeed, uint256 pSeed, bytes32 extra) public view {
        uint256 n = bound(nSeed, 2, MAX_WIDTH);
        uint256 p = bound(pSeed, 0, n - 1);

        bytes32[] memory leaves = MerkleLibSubstrate.genLeaves(n);
        (bytes32[][] memory L, bytes32 root) = MerkleLibSubstrate.buildLevels(leaves);
        bytes32[] memory proof = MerkleLibSubstrate.proofFromLevels(L, p);
        vm.assume(proof.length > 0);

        // too long: append an element
        bytes32[] memory long = new bytes32[](proof.length + 1);
        for (uint256 k = 0; k < proof.length; k++) {
            long[k] = proof[k];
        }
        long[proof.length] = extra;
        assertFalse(h.verify(root, leaves[p], p, n, long), "over-long proof must be rejected");

        // too short: drop the last element
        bytes32[] memory short = new bytes32[](proof.length - 1);
        for (uint256 k = 0; k < short.length; k++) {
            short[k] = proof[k];
        }
        assertFalse(h.verify(root, leaves[p], p, n, short), "too-short proof must be rejected");
    }

    // 4. TOTALITY + BOUNDED WORK: on fully arbitrary input the verifier never reverts (no panic
    //    from the unchecked block, no out-of-gas from a runaway loop) and does bounded work. Also
    //    pins the verify/computeRoot consistency: verify against the computed root equals `valid`.
    function testFuzz_totalAndBounded(
        bytes32 leaf,
        uint256 position,
        uint256 width,
        bytes32[] calldata proof
    ) public view {
        vm.assume(proof.length <= 1024); // keep gas loop-dominated, not calldata-dominated
        uint256 gasBefore = gasleft();
        (bool valid, bytes32 root) = h.computeRoot(leaf, position, width, proof);
        // The loop runs <= ceil(log2(width)) <= 256 iterations; assert generous bounded work.
        assertLt(gasBefore - gasleft(), 2_000_000, "computeRoot must do bounded work");

        // verify must be consistent with computeRoot: verifying against the computed root == valid.
        assertEq(
            h.verify(root, leaf, position, width, proof),
            valid,
            "verify(computedRoot) must equal computeRoot.valid"
        );

        // verify must never accept an out-of-range position, whatever the proof.
        if (position >= width) {
            assertFalse(h.verify(root, leaf, position, width, proof), "out-of-range position");
            assertFalse(valid, "out-of-range position is invalid");
        }
    }

    // 5. BOUNDARIES — single-leaf tree (root == leaf, empty canonical proof) and empty-proof edges.
    function testFuzz_singleLeaf(bytes32 leaf, uint256 posSeed, bytes32 wrongRoot) public view {
        bytes32[] memory empty = new bytes32[](0);

        // n == 1: the leaf IS the root; empty proof verifies only at index 0.
        (bool valid, bytes32 root) = h.computeRoot(leaf, 0, 1, empty);
        assertTrue(valid, "single-leaf empty proof is valid");
        assertEq(root, leaf, "single-leaf root is the leaf");
        assertTrue(h.verify(leaf, leaf, 0, 1, empty), "single leaf verifies at index 0");

        // any position > 0 is out of range -> false
        uint256 pos = bound(posSeed, 1, type(uint256).max);
        assertFalse(h.verify(leaf, leaf, pos, 1, empty), "single leaf: position > 0 is out of range");

        // a wrong expected root never verifies
        vm.assume(wrongRoot != leaf);
        assertFalse(h.verify(wrongRoot, leaf, 0, 1, empty), "single leaf: wrong root rejected");
    }

    // An empty proof for a multi-leaf tree is too short for any index: every leaf in an n>=2 tree
    // has at least one real sibling (the final width-2 reduction is even), so it is always rejected.
    function testFuzz_emptyProofMultiLeaf(uint256 nSeed, uint256 pSeed) public view {
        uint256 n = bound(nSeed, 2, MAX_WIDTH);
        uint256 p = bound(pSeed, 0, n - 1);

        bytes32[] memory leaves = MerkleLibSubstrate.genLeaves(n);
        (, bytes32 root) = MerkleLibSubstrate.buildLevels(leaves);
        bytes32[] memory empty = new bytes32[](0);

        (bool valid,) = h.computeRoot(leaves[p], p, n, empty);
        assertFalse(valid, "empty proof is structurally invalid for n>=2");
        assertFalse(h.verify(root, leaves[p], p, n, empty), "empty proof must not verify for n>=2");
    }
}
