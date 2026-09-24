// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.34;

// The compact proof format (`BeefyClient.CompactValidatorProofs`) REMOVES three checks the
// per-proof format performed explicitly, on the grounds that each is now structural:
//
//   1. "is this validator in the sample?"  -- the positions ARE the sample (Bitfield.toIndices)
//   2. "has this slot been answered already?" -- toIndices yields each position exactly once
//   3. "does the signature match the claimed account?" -- the account is recovered, not supplied
//
// A structural argument is only as good as the thing enforcing it, so this file attacks each
// one directly. It also covers the multiproof that replaces the per-proof paths: it must
// reproduce the root from the sampled positions, and its sibling array must be consumed exactly.

import {Test} from "forge-std/Test.sol";
import {stdJson} from "forge-std/StdJson.sol";
import {BeefyClient} from "../src/BeefyClient.sol";
import {BeefyClientMock} from "./mocks/BeefyClientMock.sol";
import {Bitfield} from "../src/utils/Bitfield.sol";
import {SubstrateMerkleProof} from "../src/utils/SubstrateMerkleProof.sol";
import {CompactProofLib} from "./utils/CompactProofLib.sol";
import {MerkleLibSubstrate} from "./utils/MerkleLib.sol";

contract CompactValidatorProofsTest is Test {
    using stdJson for string;

    BeefyClientMock beefyClient;
    uint8 randaoCommitDelay = 3;
    uint256 minNumRequiredSignatures;
    uint256 requiredSignatures;
    uint256 fiatShamirRequiredSignatures = 111;
    uint32 blockNumber;
    uint32 setId;
    uint32 setSize;
    uint32 prevRandao = 377;
    bytes32 commitHash;
    bytes32 root;
    bytes32 mmrRoot;
    uint256[] bitSetArray;
    uint256[] bitfield;
    BeefyClient.ValidatorProof[] finalValidatorProofs;
    BeefyClient.MMRLeaf emptyLeaf;
    bytes32[] emptyLeafProofs;
    // forge-lint: disable-next-line(unsafe-typecast)
    bytes2 mmrRootID = bytes2("mh");

    struct LegacyValidatorProof {
        uint8 v;
        bytes32 r;
        bytes32 s;
        uint256 index;
        address account;
        bytes32[] proof;
    }

    function setUp() public {
        minNumRequiredSignatures = vm.envOr("MINIMUM_REQUIRED_SIGNATURES", uint256(17));
        string memory c =
            vm.readFile(string.concat(vm.projectRoot(), "/test/data/beefy-commitment.json"));
        blockNumber = uint32(c.readUint(".params.commitment.blockNumber"));
        setId = uint32(c.readUint(".params.commitment.validatorSetID"));
        commitHash = c.readBytes32(".commitmentHash");
        mmrRoot = c.readBytes32(".params.commitment.payload[0].data");

        string memory vs =
            vm.readFile(string.concat(vm.projectRoot(), "/test/data/beefy-validator-set.json"));
        setSize = uint32(vs.readUint(".validatorSetSize"));
        root = vs.readBytes32(".validatorRoot");
        bitSetArray = vs.readUintArray(".participants");

        beefyClient = new BeefyClientMock(
            randaoCommitDelay,
            8,
            minNumRequiredSignatures,
            fiatShamirRequiredSignatures,
            0,
            BeefyClient.ValidatorSet(0, 0, 0x0),
            BeefyClient.ValidatorSet(1, 0, 0x0)
        );
        bitfield = beefyClient.createInitialBitfield(bitSetArray, setSize);
        requiredSignatures = beefyClient.computeNumRequiredSignatures_public(
            setSize, 0, minNumRequiredSignatures
        );

        string memory pr =
            vm.readFile(string.concat(vm.projectRoot(), "/test/data/beefy-final-proof.json"));
        LegacyValidatorProof[] memory ps =
            abi.decode(pr.readBytes(".finalValidatorsProofRaw"), (LegacyValidatorProof[]));
        for (uint256 i = 0; i < ps.length; i++) {
            finalValidatorProofs.push(
                BeefyClient.ValidatorProof(
                    ps[i].v, ps[i].r, ps[i].s, ps[i].index, ps[i].account, ps[i].proof
                )
            );
        }
    }

    function _reachSubmitFinal() internal returns (BeefyClient.Commitment memory commitment) {
        BeefyClient.ValidatorSet memory vset = BeefyClient.ValidatorSet(setId, setSize, root);
        BeefyClient.ValidatorSet memory nextvset =
            BeefyClient.ValidatorSet(setId + 1, setSize, root);
        beefyClient.initialize_public(0, vset, nextvset);

        BeefyClient.PayloadItem[] memory payload = new BeefyClient.PayloadItem[](1);
        payload[0] = BeefyClient.PayloadItem(mmrRootID, abi.encodePacked(mmrRoot));
        commitment = BeefyClient.Commitment(blockNumber, setId, payload);

        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[0]);
        vm.roll(block.number + randaoCommitDelay);
        vm.prevrandao(bytes32(uint256(prevRandao)));
        beefyClient.commitPrevRandao(commitHash);
    }

    function _submit(
        BeefyClient.Commitment memory commitment,
        BeefyClient.CompactValidatorProofs memory proofs
    ) internal {
        beefyClient.submitFinal(commitment, bitfield, proofs, emptyLeaf, emptyLeafProofs, 0);
    }

    // ---- baseline ----------------------------------------------------------------------

    function testCompactHappyPath() public {
        BeefyClient.Commitment memory c = _reachSubmitFinal();
        _submit(c, CompactProofLib.toCompact(finalValidatorProofs, setSize));
        assertEq(beefyClient.latestBeefyBlock(), blockNumber);
    }

    // ---- structural guarantee 3: the signer is recovered, never supplied -----------------

    function testRejectsSignatureFromAValidatorTheSampleDidNotSelect() public {
        BeefyClient.Commitment memory c = _reachSubmitFinal();
        vm.expectRevert(BeefyClient.InvalidValidatorProof.selector);
        _submit(c, CompactProofLib.withSubstitutedSigner(finalValidatorProofs, setSize, 0, 1));
    }

    // ---- structural guarantee 2: a sampled slot cannot be answered twice -----------------

    function testRejectsTheSameValidatorAnsweringTwoSlots() public {
        BeefyClient.Commitment memory c = _reachSubmitFinal();
        // Copy slot 0's signature AND its Merkle path over slot 1. Both slots now answer as
        // validator 0, which is exactly what the removed `Bitfield.unset` used to prevent.
        BeefyClient.ValidatorProof[] memory ps = _copy();
        ps[1].v = ps[0].v;
        ps[1].r = ps[0].r;
        ps[1].s = ps[0].s;
        ps[1].proof = ps[0].proof;
        vm.expectRevert();
        _submit(c, CompactProofLib.toCompact(ps, setSize));
    }

    // ---- structural guarantee 1: ordering is fixed by the sample -------------------------

    function testRejectsSignaturesOutOfAscendingOrder() public {
        BeefyClient.Commitment memory c = _reachSubmitFinal();
        // Reverse every signature and path. Each individual proof is genuine; only the order
        // no longer matches the ascending index list the contract derived.
        BeefyClient.ValidatorProof[] memory ps = _copy();
        uint256 n = ps.length;
        BeefyClient.ValidatorProof[] memory rev = new BeefyClient.ValidatorProof[](n);
        for (uint256 i = 0; i < n; i++) {
            rev[i] = ps[n - 1 - i];
        }
        vm.expectRevert();
        _submit(c, CompactProofLib.toCompact(rev, setSize));
    }

    // ---- framing: the sibling array must be consumed exactly -----------------------------

    function testRejectsTrailingSiblings() public {
        BeefyClient.Commitment memory c = _reachSubmitFinal();
        BeefyClient.CompactValidatorProofs memory p =
            CompactProofLib.toCompact(finalValidatorProofs, setSize);
        bytes32[] memory padded = new bytes32[](p.siblings.length + 1);
        for (uint256 i = 0; i < p.siblings.length; i++) {
            padded[i] = p.siblings[i];
        }
        p.siblings = padded;
        vm.expectRevert(BeefyClient.InvalidValidatorProofLength.selector);
        _submit(c, p);
    }

    function testRejectsTruncatedSiblings() public {
        BeefyClient.Commitment memory c = _reachSubmitFinal();
        BeefyClient.CompactValidatorProofs memory p =
            CompactProofLib.toCompact(finalValidatorProofs, setSize);
        bytes32[] memory short_ = new bytes32[](p.siblings.length - 1);
        for (uint256 i = 0; i < short_.length; i++) {
            short_[i] = p.siblings[i];
        }
        p.siblings = short_;
        vm.expectRevert();
        _submit(c, p);
    }

    function testRejectsWrongSignatureCount() public {
        BeefyClient.Commitment memory c = _reachSubmitFinal();
        BeefyClient.CompactValidatorProofs memory p =
            CompactProofLib.toCompact(finalValidatorProofs, setSize);
        p.signatures = bytes.concat(p.signatures, hex"00");
        vm.expectRevert(BeefyClient.InvalidValidatorProofLength.selector);
        _submit(c, p);
    }

    // ---- computeMultiRoot: same root as the full tree, siblings consumed exactly --------

    function testFuzz_computeMultiRootMatchesFullTree(uint256 wSeed, uint256 subsetSeed)
        public
        view
    {
        uint256 width = bound(wSeed, 1, 700);
        _assertMultiRootForPositions(width, randomSubset(width, subsetSeed));
    }

    function testFuzz_computeMultiRootRejectsBadFraming(uint256 wSeed, uint256 subsetSeed)
        public
        view
    {
        uint256 width = bound(wSeed, 2, 700);
        (bytes32[][] memory L,) =
            MerkleLibSubstrate.buildLevels(MerkleLibSubstrate.genLeaves(width));
        uint256[] memory positions = randomSubset(width, subsetSeed);
        bytes32[] memory leaves = leavesAt(L, positions);
        bytes32[] memory sibs = referenceSiblings(L, positions);

        bytes32[] memory longer = new bytes32[](sibs.length + 1);
        for (uint256 i = 0; i < sibs.length; i++) {
            longer[i] = sibs[i];
        }
        (bool valid,) = this.computeMultiRootExternal(positions, leaves, width, longer);
        assertFalse(valid, "trailing sibling accepted");

        if (sibs.length > 0) {
            bytes32[] memory shorter = new bytes32[](sibs.length - 1);
            for (uint256 i = 0; i < shorter.length; i++) {
                shorter[i] = sibs[i];
            }
            (valid,) = this.computeMultiRootExternal(positions, leaves, width, shorter);
            assertFalse(valid, "truncated siblings accepted");
        }
    }

    function testFuzz_computeMultiRootRejectsTampering(
        uint256 wSeed,
        uint256 shape,
        uint256 pick,
        bytes32 junk
    ) public view {
        uint256 width = bound(wSeed, 2, 700);
        (bytes32[][] memory L, bytes32 fullRoot) =
            MerkleLibSubstrate.buildLevels(MerkleLibSubstrate.genLeaves(width));
        uint256[] memory positions = subsetOfShape(width, shape);
        bytes32[] memory leaves = leavesAt(L, positions);
        bytes32[] memory sibs = referenceSiblings(L, positions);

        uint256 k = pick >> 1;
        if (pick & 1 == 1 && sibs.length >= 2) {
            // Swap two genuine siblings: right values, wrong order.
            uint256 i = k % sibs.length;
            uint256 j = (i + 1 + (k >> 128) % (sibs.length - 1)) % sibs.length;
            vm.assume(sibs[i] != sibs[j]);
            (sibs[i], sibs[j]) = (sibs[j], sibs[i]);
        } else {
            // Replace one leaf or one sibling with a different value.
            k %= leaves.length + sibs.length;
            if (k < leaves.length) {
                vm.assume(junk != leaves[k]);
                leaves[k] = junk;
            } else {
                vm.assume(junk != sibs[k - leaves.length]);
                sibs[k - leaves.length] = junk;
            }
        }

        (bool valid, bytes32 got) =
            this.computeMultiRootExternal(copy(positions), leaves, width, sibs);
        assertFalse(valid && got == fullRoot, "tampered multiproof reproduced the root");
    }

    /// Random, full or near-full subset, so tampering also runs where nearly every node pairs.
    function subsetOfShape(uint256 width, uint256 shape) internal pure returns (uint256[] memory) {
        if (shape % 3 == 1) return fullRange(width);
        if (shape % 3 == 2) return omitOne(width, (shape >> 2) % width);
        return randomSubset(width, shape);
    }

    function copy(uint256[] memory a) internal pure returns (uint256[] memory b) {
        b = new uint256[](a.length);
        for (uint256 i = 0; i < a.length; i++) {
            b[i] = a[i];
        }
    }

    // Polytope's multiproof verifier was forged (solidity-merkle-trees#57) with a leaf count
    // above 2^255: leaves started on different layers, the walk stopped at the root, and a
    // forged subtree was never folded in. Here every node stays on one layer and the loop is
    // bounded by `width`, so even at the largest widths the call must terminate cleanly.

    /// Arbitrary input at any width: returns instead of panicking (a revert fails the test).
    function testFuzz_computeMultiRootHugeWidthTerminates(
        uint256 width,
        uint256 seed,
        uint8 nSeed,
        uint8 sibSeed
    ) public view {
        width = bound(width, 2, type(uint256).max);
        uint256 n = bound(nSeed, 1, 8);
        uint256[] memory positions = new uint256[](n);
        bytes32[] memory leaves = new bytes32[](n);
        for (uint256 i = 0; i < n; i++) {
            // Spread positions over the whole range, including the top edge.
            positions[i] = uint256(keccak256(abi.encode(seed, i))) % width;
            leaves[i] = keccak256(abi.encode(seed, "leaf", i));
        }
        MerkleLibSubstrate.sort(positions);
        bytes32[] memory sibs = siblings(bytes32(seed), sibSeed);

        this.computeMultiRootExternal(positions, leaves, width, sibs);
    }

    /// One leaf at any width, including the top edge: same result as `computeRoot`.
    function testFuzz_computeMultiRootHugeWidthMatchesComputeRoot(
        uint256 width,
        uint256 pSeed,
        bytes32 leaf
    ) public view {
        width = bound(width, 1, type(uint256).max);
        uint256 position = pSeed % 4 == 0 ? width - 1 : bound(pSeed, 0, width - 1);
        bytes32[] memory path = siblings(leaf, pathLength(position, width));

        uint256[] memory positions = new uint256[](1);
        positions[0] = position;
        bytes32[] memory leaves = new bytes32[](1);
        leaves[0] = leaf;

        (bool multiOk, bytes32 multiRoot) =
            this.computeMultiRootExternal(positions, leaves, width, path);
        (bool singleOk, bytes32 singleRoot) = this.computeRootExternal(leaf, position, width, path);
        assertTrue(multiOk && singleOk, "canonical path rejected");
        assertEq(multiRoot, singleRoot, "multiproof != computeRoot");
    }

    function siblings(bytes32 seed, uint256 n) internal pure returns (bytes32[] memory out) {
        out = new bytes32[](n);
        for (uint256 i = 0; i < n; i++) {
            out[i] = keccak256(abi.encode(seed, "sibling", i));
        }
    }

    /// Canonical path length for `position`: one sibling per layer, except where it is the lone
    /// trailing node of an odd-width layer.
    function pathLength(uint256 position, uint256 width) internal pure returns (uint256 n) {
        while (width > 1) {
            if (!(position + 1 == width && width & 1 == 1)) n++;
            position >>= 1;
            width = ((width - 1) >> 1) + 1;
        }
    }

    function testComputeMultiRootRejectsBadPositions() public view {
        bytes32[] memory two = new bytes32[](2);
        bytes32[] memory none = new bytes32[](0);
        uint256[] memory p = new uint256[](2);

        (p[0], p[1]) = (0, 600);
        (bool valid,) = this.computeMultiRootExternal(p, two, 600, none);
        assertFalse(valid, "out-of-range position");

        (p[0], p[1]) = (5, 5);
        (valid,) = this.computeMultiRootExternal(p, two, 600, none);
        assertFalse(valid, "duplicate position");

        (p[0], p[1]) = (6, 5);
        (valid,) = this.computeMultiRootExternal(p, two, 600, none);
        assertFalse(valid, "descending positions");

        (valid,) = this.computeMultiRootExternal(new uint256[](0), none, 600, none);
        assertFalse(valid, "no leaves");

        (p[0], p[1]) = (0, 1);
        (valid,) = this.computeMultiRootExternal(p, new bytes32[](1), 600, none);
        assertFalse(valid, "positions and leaves differ in length");
    }

    // Dense subsets are almost never hit by `randomSubset` (~1/5 keep rate). Full set means an
    // empty sibling array and a pure in-tree fold; near-full maximizes pairing. Both must still
    // reproduce the root and agree with the independent reference sibling oracle.
    function testFuzz_computeMultiRootFullSet(uint256 wSeed) public view {
        uint256 width = bound(wSeed, 1, 700);
        _assertMultiRootForPositions(width, fullRange(width));
    }

    function testFuzz_computeMultiRootNearFullSet(uint256 wSeed, uint256 omitSeed) public view {
        uint256 width = bound(wSeed, 2, 700);
        _assertMultiRootForPositions(width, omitOne(width, omitSeed % width));
    }

    // A one-leaf multiproof must be byte-equivalent to `computeRoot`: same siblings consumed,
    // same root. Odd widths exercise the promotion rule on both paths.
    function testFuzz_computeMultiRootMatchesComputeRoot(uint256 wSeed, uint256 pSeed)
        public
        view
    {
        uint256 width = bound(wSeed, 1, 700);
        uint256 position = bound(pSeed, 0, width - 1);
        (bytes32[][] memory L, bytes32 fullRoot) =
            MerkleLibSubstrate.buildLevels(MerkleLibSubstrate.genLeaves(width));

        uint256[] memory positions = new uint256[](1);
        positions[0] = position;
        bytes32[] memory leaves = leavesAt(L, positions);
        bytes32[] memory sibs = referenceSiblings(L, positions);
        bytes32[] memory singlePath = MerkleLibSubstrate.proofFromLevels(L, position);

        assertEq(sibs, singlePath, "single-leaf multiproof siblings != canonical path");

        (bool multiOk, bytes32 multiRoot) =
            this.computeMultiRootExternal(positions, leaves, width, sibs);
        (bool singleOk, bytes32 singleRoot) =
            this.computeRootExternal(leaves[0], position, width, singlePath);

        assertTrue(multiOk && singleOk, "single-leaf proof rejected");
        assertEq(multiRoot, fullRoot, "multiproof root");
        assertEq(singleRoot, fullRoot, "computeRoot root");
        assertEq(multiRoot, singleRoot, "multiproof != computeRoot");
    }

    function _assertMultiRootForPositions(uint256 width, uint256[] memory positions)
        internal
        view
    {
        (bytes32[][] memory L, bytes32 fullRoot) =
            MerkleLibSubstrate.buildLevels(MerkleLibSubstrate.genLeaves(width));
        bytes32[] memory expected = referenceSiblings(L, positions);

        BeefyClient.ValidatorProof[] memory ps = new BeefyClient.ValidatorProof[](positions.length);
        for (uint256 i = 0; i < positions.length; i++) {
            ps[i].index = positions[i];
            ps[i].proof = MerkleLibSubstrate.proofFromLevels(L, positions[i]);
        }
        assertEq(CompactProofLib.multiproof(ps, width), expected, "builder disagrees");

        (bool valid, bytes32 got) =
            this.computeMultiRootExternal(positions, leavesAt(L, positions), width, expected);
        assertTrue(valid, "multiproof rejected");
        assertEq(got, fullRoot, "multiproof root disagrees with full tree");
        if (positions.length == width) {
            assertEq(expected.length, 0, "full set must need no siblings");
        }
    }

    function fullRange(uint256 width) internal pure returns (uint256[] memory out) {
        out = new uint256[](width);
        for (uint256 i = 0; i < width; i++) {
            out[i] = i;
        }
    }

    function omitOne(uint256 width, uint256 omit) internal pure returns (uint256[] memory out) {
        out = new uint256[](width - 1);
        uint256 k;
        for (uint256 i = 0; i < width; i++) {
            if (i == omit) continue;
            out[k++] = i;
        }
    }

    /// Siblings a multiproof needs, from the full tree: per layer, each known node's sibling
    /// unless that sibling is known too.
    function referenceSiblings(bytes32[][] memory L, uint256[] memory positions)
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

    function randomSubset(uint256 width, uint256 seed)
        internal
        pure
        returns (uint256[] memory out)
    {
        out = new uint256[](width);
        uint256 n;
        for (uint256 p = 0; p < width; p++) {
            if (uint256(keccak256(abi.encode(seed, p))) % 5 == 0) out[n++] = p;
        }
        if (n == 0) out[n++] = seed % width;
        assembly {
            mstore(out, n)
        }
    }

    function leavesAt(bytes32[][] memory L, uint256[] memory positions)
        internal
        pure
        returns (bytes32[] memory leaves)
    {
        leaves = new bytes32[](positions.length);
        for (uint256 i = 0; i < positions.length; i++) {
            leaves[i] = L[0][positions[i]];
        }
    }

    function computeMultiRootExternal(
        uint256[] memory positions,
        bytes32[] memory leaves,
        uint256 width,
        bytes32[] calldata siblings
    ) external pure returns (bool, bytes32) {
        return SubstrateMerkleProof.computeMultiRoot(positions, leaves, width, siblings);
    }

    function computeRootExternal(
        bytes32 leaf,
        uint256 position,
        uint256 width,
        bytes32[] calldata proof
    ) external pure returns (bool, bytes32) {
        return SubstrateMerkleProof.computeRoot(leaf, position, width, proof);
    }

    // ---- toIndices ----------------------------------------------------------------------

    function testToIndicesIsAscendingExactAndComplete() public view {
        uint256[] memory sample =
            Bitfield.subsample(prevRandao, bitfield, setSize, requiredSignatures);
        uint256[] memory idx = Bitfield.toIndices(sample, requiredSignatures);
        assertEq(idx.length, requiredSignatures);
        for (uint256 i = 0; i < idx.length; i++) {
            assertTrue(Bitfield.isSet(sample, idx[i]), "index not in sample");
            assertLt(idx[i], setSize, "index out of range");
            if (i > 0) {
                assertGt(idx[i], idx[i - 1], "not strictly ascending");
            }
        }
    }

    function testFuzz_toIndicesMatchesNaiveScan(uint256 a, uint256 b, uint256 c) public pure {
        uint256[] memory bf = new uint256[](3);
        bf[0] = a;
        bf[1] = b;
        bf[2] = c;

        uint256 n;
        for (uint256 i = 0; i < 768; i++) {
            if (Bitfield.isSet(bf, i)) n++;
        }
        vm.assume(n > 0);

        uint256[] memory idx = Bitfield.toIndices(bf, n);
        assertEq(idx.length, n);

        uint256 k;
        for (uint256 i = 0; i < 768; i++) {
            if (Bitfield.isSet(bf, i)) {
                assertEq(idx[k], i, "toIndices disagrees with naive scan");
                k++;
            }
        }
    }

    function testToIndicesRejectsCountMismatch() public {
        uint256[] memory sample =
            Bitfield.subsample(prevRandao, bitfield, setSize, requiredSignatures);
        vm.expectRevert(Bitfield.InvalidSamplingParams.selector);
        this.callToIndices(sample, requiredSignatures - 1);
        vm.expectRevert(Bitfield.InvalidSamplingParams.selector);
        this.callToIndices(sample, requiredSignatures + 1);
    }

    function callToIndices(uint256[] memory b, uint256 n) external pure {
        Bitfield.toIndices(b, n);
    }

    function _copy() internal view returns (BeefyClient.ValidatorProof[] memory ps) {
        ps = new BeefyClient.ValidatorProof[](finalValidatorProofs.length);
        for (uint256 i = 0; i < ps.length; i++) {
            ps[i] = finalValidatorProofs[i];
        }
    }
}
