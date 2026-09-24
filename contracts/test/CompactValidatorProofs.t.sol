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
// one directly. It also covers the two new framing invariants that replace the per-proof
// length field: every leaf gets exactly its canonical path, and the flat sibling array must be
// consumed exactly.

import {Test} from "forge-std/Test.sol";
import {stdJson} from "forge-std/StdJson.sol";
import {BeefyClient} from "../src/BeefyClient.sol";
import {BeefyClientMock} from "./mocks/BeefyClientMock.sol";
import {Bitfield} from "../src/utils/Bitfield.sol";
import {SubstrateMerkleProof} from "../src/utils/SubstrateMerkleProof.sol";
import {CompactProofLib} from "./utils/CompactProofLib.sol";

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
        _submit(c, CompactProofLib.toCompact(finalValidatorProofs));
        assertEq(beefyClient.latestBeefyBlock(), blockNumber);
    }

    // ---- structural guarantee 3: the signer is recovered, never supplied -----------------

    function testRejectsSignatureFromAValidatorTheSampleDidNotSelect() public {
        BeefyClient.Commitment memory c = _reachSubmitFinal();
        vm.expectRevert(BeefyClient.InvalidValidatorProof.selector);
        _submit(c, CompactProofLib.withSubstitutedSigner(finalValidatorProofs, 0, 1));
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
        _submit(c, CompactProofLib.toCompact(ps));
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
        _submit(c, CompactProofLib.toCompact(rev));
    }

    // ---- framing: the sibling array must be consumed exactly -----------------------------

    function testRejectsTrailingSiblings() public {
        BeefyClient.Commitment memory c = _reachSubmitFinal();
        BeefyClient.CompactValidatorProofs memory p =
            CompactProofLib.toCompact(finalValidatorProofs);
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
            CompactProofLib.toCompact(finalValidatorProofs);
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
            CompactProofLib.toCompact(finalValidatorProofs);
        p.signatures = bytes.concat(p.signatures, hex"00");
        vm.expectRevert(BeefyClient.InvalidValidatorProofLength.selector);
        _submit(c, p);
    }

    // ---- computeRootAt must consume exactly the canonical path, same root as computeRoot ----

    function testFuzz_computeRootAtMatchesComputeRoot(
        uint256 wSeed,
        uint256 pSeed,
        bytes32 leaf,
        uint256 extraSeed
    ) public view {
        uint256 width = bound(wSeed, 1, 1024);
        uint256 position = bound(pSeed, 0, width - 1);
        uint256 expected = canonicalPathLength(position, width);
        // A flat array with trailing siblings that belong to the next leaf.
        bytes32[] memory flat = siblings(leaf, expected + bound(extraSeed, 0, 5));

        (bool valid, bytes32 root, uint256 next) =
            this.computeRootAtExternal(leaf, position, width, flat, 0);
        assertTrue(valid, "canonical path rejected");
        assertEq(next, expected, "computeRootAt consumed a non-canonical number of siblings");
        assertEq(
            root, referenceRoot(leaf, position, width, expected), "disagrees with computeRoot"
        );
    }

    function referenceRoot(bytes32 leaf, uint256 position, uint256 width, uint256 len)
        internal
        view
        returns (bytes32)
    {
        (bool valid, bytes32 root) =
            this.computeRootExternal(leaf, position, width, siblings(leaf, len));
        assertTrue(valid, "computeRoot rejected the canonical path");
        return root;
    }

    function siblings(bytes32 seed, uint256 n) internal pure returns (bytes32[] memory out) {
        out = new bytes32[](n);
        for (uint256 i = 0; i < n; i++) {
            out[i] = keccak256(abi.encode(seed, i));
        }
    }

    function canonicalPathLength(uint256 p, uint256 w) internal pure returns (uint256 n) {
        while (w > 1) {
            if (!(p + 1 == w && w & 1 == 1)) {
                n++;
            }
            p >>= 1;
            w = ((w - 1) >> 1) + 1;
        }
    }

    function testComputeRootAtRejectsOutOfRangePosition() public view {
        (bool valid,,) =
            this.computeRootAtExternal(bytes32(uint256(1)), 600, 600, new bytes32[](0), 0);
        assertFalse(valid);
        (valid,,) = this.computeRootAtExternal(bytes32(uint256(1)), 601, 600, new bytes32[](0), 0);
        assertFalse(valid);
    }

    function testComputeRootAtRejectsTruncatedPath() public view {
        (bool valid,,) =
            this.computeRootAtExternal(bytes32(uint256(1)), 0, 600, new bytes32[](3), 0);
        assertFalse(valid);
    }

    function computeRootAtExternal(
        bytes32 leaf,
        uint256 position,
        uint256 width,
        bytes32[] calldata siblings,
        uint256 offset
    ) external pure returns (bool, bytes32, uint256) {
        return SubstrateMerkleProof.computeRootAt(leaf, position, width, siblings, offset);
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
