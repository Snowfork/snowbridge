// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.34;

import {Test} from "forge-std/Test.sol";
import {BeefyClient} from "../src/BeefyClient.sol";
import {BeefyClientMock} from "./mocks/BeefyClientMock.sol";
import {Bitfield} from "../src/utils/Bitfield.sol";
import {MerkleLibSubstrate} from "./utils/MerkleLib.sol";
import {CompactProofLib} from "./utils/CompactProofLib.sol";

/// @dev Attacks on the compact proof format, with validator keys the test controls: genuine
/// signatures reused under a forged payload, a misconfigured Fiat-Shamir signature count, and
/// small validator sets where `computeMaxRequiredSignatures` caps the Fiat-Shamir sample.
contract BeefyClientCompactAttackTest is Test {
    // forge-lint: disable-next-line(unsafe-typecast)
    bytes2 constant MMR_ROOT_ID = bytes2("mh");
    bytes32 constant HONEST_ROOT = keccak256("honest MMR root");
    bytes32 constant FORGED_ROOT = keccak256("forged MMR root");
    uint64 constant SET_ID = 1;
    uint256 constant RANDAO_DELAY = 128;
    uint256 constant FS_SIGNATURES = 111;

    BeefyClientMock client;
    uint256 n;
    uint256[] keys;
    address[] signers;
    bytes32[][] paths;
    bytes32 vsetRoot;
    BeefyClient.MMRLeaf emptyLeaf;

    function deploy(uint256 setSize, uint256 fiatShamirSignatures) internal {
        n = setSize;
        delete keys;
        delete signers;
        bytes32[] memory leaves = new bytes32[](setSize);
        for (uint256 i = 0; i < setSize; i++) {
            keys.push(uint256(keccak256(abi.encode("validator", i))));
            signers.push(vm.addr(keys[i]));
            leaves[i] = keccak256(abi.encodePacked(signers[i]));
        }
        (vsetRoot, paths) = MerkleLibSubstrate.buildBinaryMerkleTree(leaves);
        client = new BeefyClientMock(
            RANDAO_DELAY,
            24,
            17,
            fiatShamirSignatures,
            0,
            // forge-lint: disable-next-line(unsafe-typecast)
            BeefyClient.ValidatorSet(SET_ID, uint128(setSize), vsetRoot),
            // forge-lint: disable-next-line(unsafe-typecast)
            BeefyClient.ValidatorSet(SET_ID + 1, uint128(setSize), vsetRoot)
        );
    }

    function commitmentFor(uint32 blockNumber, bytes32 mmrRoot)
        internal
        view
        returns (BeefyClient.Commitment memory c, bytes32 hash)
    {
        BeefyClient.PayloadItem[] memory payload = new BeefyClient.PayloadItem[](1);
        payload[0] = BeefyClient.PayloadItem(MMR_ROOT_ID, bytes.concat(mmrRoot));
        c = BeefyClient.Commitment(blockNumber, SET_ID, payload);
        hash = keccak256(client.encodeCommitment_public(c));
    }

    /// The minimum quorum: validators 0..quorum-1 claim to have signed.
    function claimQuorum() internal view returns (uint256[] memory bitfield) {
        bitfield = new uint256[](Bitfield.containerLength(n));
        uint256 quorum = client.computeQuorum_public(n);
        for (uint256 i = 0; i < quorum; i++) {
            Bitfield.set(bitfield, i);
        }
    }

    /// One signature over `signedHash` from each validator in `sample`, as compact proofs.
    function proofsFor(uint256[] memory sample, bytes32 signedHash)
        internal
        view
        returns (BeefyClient.CompactValidatorProofs memory)
    {
        BeefyClient.ValidatorProof[] memory ps =
            new BeefyClient.ValidatorProof[](Bitfield.countSetBits(sample));
        uint256 j;
        for (uint256 i = 0; i < n; i++) {
            if (!Bitfield.isSet(sample, i)) continue;
            (uint8 v, bytes32 r, bytes32 s) = vm.sign(keys[i], signedHash);
            ps[j++] = BeefyClient.ValidatorProof(v, r, s, i, signers[i], paths[i]);
        }
        return CompactProofLib.toCompact(ps, n);
    }

    function submitFiatShamir(
        BeefyClient.Commitment memory c,
        uint256[] memory bitfield,
        BeefyClient.CompactValidatorProofs memory proofs
    ) internal {
        client.submitFiatShamir(c, bitfield, proofs, emptyLeaf, new bytes32[](0), 0);
    }

    // ---- genuine signatures under a forged payload -------------------------------------

    /// The honest commitment verifies, so the forged cases below fail for the forgery alone.
    function testFiatShamirAcceptsTheHonestCommitment() public {
        deploy(600, FS_SIGNATURES);
        (BeefyClient.Commitment memory c, bytes32 h) = commitmentFor(1, HONEST_ROOT);
        uint256[] memory bitfield = claimQuorum();
        submitFiatShamir(
            c, bitfield, proofsFor(client.createFiatShamirFinalBitfield(c, bitfield), h)
        );
        assertEq(client.latestMMRRoot(), HONEST_ROOT);
    }

    /// Every validator signed the honest commitment. The attacker swaps in a forged MMR root (or
    /// block number), takes the sample the forged commitment selects, and answers it with the
    /// honest signatures of exactly those validators.
    /// forge-config: default.fuzz.runs = 16
    function testFuzz_fiatShamirRejectsGenuineSignaturesUnderAForgedCommitment(
        bytes32 forgedRoot,
        uint32 forgedBlock
    ) public {
        deploy(600, FS_SIGNATURES);
        (, bytes32 honestHash) = commitmentFor(1, HONEST_ROOT);
        forgedBlock = uint32(bound(forgedBlock, 1, type(uint32).max));
        vm.assume(forgedRoot != HONEST_ROOT || forgedBlock != 1);
        (BeefyClient.Commitment memory forged,) = commitmentFor(forgedBlock, forgedRoot);
        uint256[] memory bitfield = claimQuorum();

        BeefyClient.CompactValidatorProofs memory proofs =
            proofsFor(client.createFiatShamirFinalBitfield(forged, bitfield), honestHash);
        vm.expectRevert(BeefyClient.InvalidValidatorProof.selector);
        submitFiatShamir(forged, bitfield, proofs);
        assertEq(client.latestMMRRoot(), bytes32(0));
    }

    /// The same attack on the honest commitment's own sample: positions no longer match.
    function testFiatShamirRejectsTheHonestSampleUnderAForgedRoot() public {
        deploy(600, FS_SIGNATURES);
        (BeefyClient.Commitment memory honest, bytes32 honestHash) = commitmentFor(1, HONEST_ROOT);
        (BeefyClient.Commitment memory forged,) = commitmentFor(1, FORGED_ROOT);
        uint256[] memory bitfield = claimQuorum();

        BeefyClient.CompactValidatorProofs memory proofs =
            proofsFor(client.createFiatShamirFinalBitfield(honest, bitfield), honestHash);
        vm.expectRevert();
        submitFiatShamir(forged, bitfield, proofs);
        assertEq(client.latestMMRRoot(), bytes32(0));
    }

    /// Interactive path: a ticket opened for the honest commitment cannot finalise a forged one,
    /// and an honest signature cannot open a ticket for the forged one.
    function testInteractiveRejectsGenuineSignaturesUnderAForgedCommitment() public {
        deploy(600, FS_SIGNATURES);
        (BeefyClient.Commitment memory honest, bytes32 honestHash) = commitmentFor(1, HONEST_ROOT);
        (BeefyClient.Commitment memory forged,) = commitmentFor(1, FORGED_ROOT);
        uint256[] memory bitfield = claimQuorum();
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(keys[0], honestHash);
        BeefyClient.ValidatorProof memory initial =
            BeefyClient.ValidatorProof(v, r, s, 0, signers[0], paths[0]);

        vm.expectRevert(BeefyClient.InvalidSignature.selector);
        client.submitInitial(forged, bitfield, initial);

        client.submitInitial(honest, bitfield, initial);
        vm.roll(block.number + RANDAO_DELAY);
        client.commitPrevRandao(honestHash);
        BeefyClient.CompactValidatorProofs memory proofs =
            proofsFor(client.createFinalBitfield(honestHash, bitfield), honestHash);

        vm.expectRevert(BeefyClient.InvalidTicket.selector);
        client.submitFinal(forged, bitfield, proofs, emptyLeaf, new bytes32[](0), 0);
        assertEq(client.latestMMRRoot(), bytes32(0));
    }

    // ---- Fiat-Shamir signature count -----------------------------------------------------

    /// A zero count fails closed: the sample is empty, and `computeMultiRoot` rejects it.
    function testFiatShamirWithZeroRequiredSignaturesFailsClosed() public {
        deploy(64, 0);
        (BeefyClient.Commitment memory c,) = commitmentFor(1, HONEST_ROOT);
        uint256[] memory bitfield = claimQuorum();
        assertEq(Bitfield.countSetBits(client.createFiatShamirFinalBitfield(c, bitfield)), 0);

        vm.expectRevert(BeefyClient.InvalidValidatorProofLength.selector);
        submitFiatShamir(c, bitfield, BeefyClient.CompactValidatorProofs("", new bytes32[](0)));
    }

    /// Why the redeploy needs a constructor floor: with a count of 1, one signature from the
    /// sampled validator finalises a commitment. An attacker holding a third of the set wins
    /// about one in three tries, and each try is an offline hash.
    function testFiatShamirWithOneRequiredSignatureAcceptsASingleSigner() public {
        deploy(64, 1);
        (BeefyClient.Commitment memory c, bytes32 h) = commitmentFor(1, FORGED_ROOT);
        uint256[] memory bitfield = claimQuorum();
        uint256[] memory sample = client.createFiatShamirFinalBitfield(c, bitfield);
        assertEq(Bitfield.countSetBits(sample), 1);

        submitFiatShamir(c, bitfield, proofsFor(sample, h));
        assertEq(client.latestMMRRoot(), FORGED_ROOT);
    }

    // ---- small validator sets --------------------------------------------------------------

    /// Westend-sized set: `computeMaxRequiredSignatures` caps the sample below 111.
    function testFiatShamirOnATwentyValidatorSetUsesTheCap() public {
        checkCappedSample(20);
    }

    /// forge-config: default.fuzz.runs = 16
    function testFuzz_fiatShamirOnSmallSetsUsesTheCap(uint256 setSize) public {
        checkCappedSample(bound(setSize, 1, 200));
    }

    function checkCappedSample(uint256 setSize) internal {
        deploy(setSize, FS_SIGNATURES);
        uint256 expected = FS_SIGNATURES < client.computeMaxRequiredSignatures_public(setSize)
            ? FS_SIGNATURES
            : client.computeMaxRequiredSignatures_public(setSize);
        (BeefyClient.Commitment memory c, bytes32 h) = commitmentFor(1, HONEST_ROOT);
        uint256[] memory bitfield = claimQuorum();
        uint256[] memory sample = client.createFiatShamirFinalBitfield(c, bitfield);
        assertEq(Bitfield.countSetBits(sample), expected, "sample size");

        BeefyClient.CompactValidatorProofs memory proofs = proofsFor(sample, h);
        if (expected > 0) {
            // One signature short is rejected.
            BeefyClient.CompactValidatorProofs memory short = BeefyClient.CompactValidatorProofs(
                slice(proofs.signatures, (expected - 1) * 65), proofs.siblings
            );
            vm.expectRevert(BeefyClient.InvalidValidatorProofLength.selector);
            submitFiatShamir(c, bitfield, short);
        }
        submitFiatShamir(c, bitfield, proofs);
        assertEq(client.latestMMRRoot(), HONEST_ROOT);
    }

    function slice(bytes memory b, uint256 len) internal pure returns (bytes memory out) {
        out = new bytes(len);
        for (uint256 i = 0; i < len; i++) {
            out[i] = b[i];
        }
    }
}
