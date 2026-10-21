// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.34;

import {Test} from "forge-std/Test.sol";
import {BeefyClient} from "../src/BeefyClient.sol";
import {Bitfield} from "../src/utils/Bitfield.sol";
import {BeefyClientMock} from "./mocks/BeefyClientMock.sol";
import {MerkleLibSubstrate} from "./utils/MerkleLib.sol";
import {CompactProofLib} from "./utils/CompactProofLib.sol";

/// @dev Statistical checks of the sampling the security analysis assumes: an attacker below the
/// cap wins with exactly the hypergeometric probability, wherever its keys sit, and the sampler
/// picks every claimed validator equally often. Seeds are fixed, so results are deterministic;
/// the bounds are 4 to 5 standard deviations wide.
contract BeefyClientSamplingStatsTest is Test {
    // forge-lint: disable-next-line(unsafe-typecast)
    bytes2 constant MMR_ROOT_ID = bytes2("mh");
    uint64 constant SET_ID = 1;

    // Toy Fiat-Shamir setting: 16 validators, the attacker holds 5 (the most BFT allows), claims
    // the quorum of 11, and the sample is 2. One try wins with C(5,2) / C(11,2) = 10/55.
    uint256 constant N = 16;
    uint256 constant ATTACKER = 5;
    uint256 constant QUORUM = 11;
    uint256 constant SAMPLE = 2;
    uint256 constant TRIES = 2000;

    BeefyClientMock client;
    uint256[] keys;
    address[] signers;
    bytes32[][] paths;

    function setUp() public {
        bytes32[] memory leaves = new bytes32[](N);
        for (uint256 i = 0; i < N; i++) {
            keys.push(uint256(keccak256(abi.encode("validator", i))));
            signers.push(vm.addr(keys[i]));
            leaves[i] = keccak256(abi.encodePacked(signers[i]));
        }
        bytes32 root;
        (root, paths) = MerkleLibSubstrate.buildBinaryMerkleTree(leaves);
        client = new BeefyClientMock(
            3,
            8,
            4,
            SAMPLE,
            0,
            // forge-lint: disable-next-line(unsafe-typecast)
            BeefyClient.ValidatorSet(SET_ID, uint128(N), root),
            // forge-lint: disable-next-line(unsafe-typecast)
            BeefyClient.ValidatorSet(SET_ID + 1, uint128(N), root)
        );
    }

    // ---- Fiat-Shamir: grinding success rate ------------------------------------------------

    /// The attacker grinds forged commitments offline. Its win rate matches 10/55 whether its
    /// keys are the lowest, the highest or scattered indices.
    function testFiatShamirGrindingMatchesTheHypergeometricRate() public view {
        uint256[3] memory layouts = [uint256(0), 1, 2];
        for (uint256 k = 0; k < layouts.length; k++) {
            (uint256[] memory attacker, uint256[] memory claim) = layout(layouts[k]);
            uint256 wins;
            for (uint256 salt = 0; salt < TRIES; salt++) {
                (BeefyClient.Commitment memory c,) = forged(layouts[k], salt);
                if (isSubset(client.createFiatShamirFinalBitfield(c, claim), attacker)) wins++;
            }
            // Mean 363.6, standard deviation 17.2: 4 sigma is about 69.
            assertApproxEqAbs(wins * 55, TRIES * 10, 69 * 55, "win rate is not 10/55");
        }
    }

    /// The contract accepts a forgery exactly when the sample lands only on attacker keys.
    function testFiatShamirAcceptsExactlyTheLuckySamples() public {
        (uint256[] memory attacker, uint256[] memory claim) = layout(2);
        bool sawLucky;
        bool sawUnlucky;
        for (uint256 salt = 0; salt < TRIES && !(sawLucky && sawUnlucky); salt++) {
            (BeefyClient.Commitment memory c, bytes32 h) = forged(2, salt);
            uint256[] memory sample = client.createFiatShamirFinalBitfield(c, claim);
            bool lucky = isSubset(sample, attacker);
            if (lucky ? sawLucky : sawUnlucky) continue;

            BeefyClient.CompactValidatorProofs memory proofs = attackerProofs(sample, attacker, h);
            uint256 snapshot = vm.snapshotState();
            if (!lucky) {
                vm.expectRevert(BeefyClient.InvalidValidatorProof.selector);
            }
            client.submitFiatShamir(c, claim, proofs, noLeaf(), new bytes32[](0), 0);
            if (lucky) {
                assertEq(client.latestMMRRoot(), bytes32(c.payload[0].data), "lucky forgery");
                sawLucky = true;
            } else {
                sawUnlucky = true;
            }
            vm.revertToState(snapshot);
        }
        assertTrue(sawLucky && sawUnlucky, "grinding never found both kinds of sample");
    }

    // ---- uniformity of the sampler ------------------------------------------------------

    /// Over many seeds, every claimed validator is picked equally often: chi-square against
    /// uniform, 43 claimed of 64, 7 per sample, 3,000 samples. Unclaimed validators are never
    /// picked.
    function testSubsamplePicksEveryClaimedValidatorEquallyOften() public pure {
        uint256 size = 64;
        uint256 perSample = 7;
        uint256 samples = 3000;
        uint256[] memory claim = new uint256[](1);
        uint256 claimed;
        for (uint256 i = 0; i < size; i++) {
            if (i % 3 != 2) {
                claim[0] |= uint256(1) << i;
                claimed++;
            }
        }

        uint256[] memory picks = new uint256[](size);
        for (uint256 s = 0; s < samples; s++) {
            uint256 word = Bitfield.subsample(
                uint256(keccak256(abi.encode("seed", s))), claim, size, perSample
            )[0];
            for (uint256 i = 0; i < size; i++) {
                if (word >> i & 1 == 1) picks[i]++;
            }
        }

        // chi2 = sum((O - E)^2 / E) with E = total / claimed, in integers:
        // sum((claimed * O - total)^2) / (claimed * total).
        uint256 total = samples * perSample;
        uint256 numerator;
        for (uint256 i = 0; i < size; i++) {
            if (claim[0] >> i & 1 == 0) {
                assertEq(picks[i], 0, "unclaimed validator picked");
                continue;
            }
            int256 d = int256(claimed * picks[i]) - int256(total);
            numerator += uint256(d * d);
        }
        uint256 chi2Milli = numerator * 1000 / (claimed * total);
        // 42 degrees of freedom: mean 42; the 0.9999 quantile is about 85.
        assertLt(chi2Milli, 85_000, "picks are not uniform");
    }

    // ---- helpers ----------------------------------------------------------------------

    /// Attacker keys and the claimed bitfield: 0 = lowest indices, 1 = highest, 2 = scattered.
    function layout(uint256 kind)
        internal
        pure
        returns (uint256[] memory attacker, uint256[] memory claim)
    {
        attacker = new uint256[](ATTACKER);
        claim = new uint256[](1);
        for (uint256 i = 0; i < ATTACKER; i++) {
            attacker[i] = kind == 0 ? i : kind == 1 ? N - ATTACKER + i : 3 * i + 1;
        }
        uint256 count;
        for (uint256 i = 0; i < ATTACKER; i++) {
            claim[0] |= uint256(1) << attacker[i];
            count++;
        }
        // Fill the quorum with honest validators that did not sign the forgery.
        for (uint256 i = 0; count < QUORUM; i++) {
            if (claim[0] >> i & 1 == 0) {
                claim[0] |= uint256(1) << i;
                count++;
            }
        }
    }

    function forged(uint256 kind, uint256 salt)
        internal
        view
        returns (BeefyClient.Commitment memory c, bytes32 h)
    {
        BeefyClient.PayloadItem[] memory payload = new BeefyClient.PayloadItem[](1);
        payload[0] =
            BeefyClient.PayloadItem(MMR_ROOT_ID, abi.encode(keccak256(abi.encode(kind, salt))));
        c = BeefyClient.Commitment(1, SET_ID, payload);
        h = keccak256(client.encodeCommitment_public(c));
    }

    function isSubset(uint256[] memory sample, uint256[] memory attacker)
        internal
        pure
        returns (bool)
    {
        uint256 mask;
        for (uint256 i = 0; i < attacker.length; i++) {
            mask |= uint256(1) << attacker[i];
        }
        return sample[0] & ~mask == 0;
    }

    /// Attacker keys sign `h`; an honest slot gets a genuine signature over another message.
    function attackerProofs(uint256[] memory sample, uint256[] memory attacker, bytes32 h)
        internal
        view
        returns (BeefyClient.CompactValidatorProofs memory)
    {
        uint256 mask;
        for (uint256 i = 0; i < attacker.length; i++) {
            mask |= uint256(1) << attacker[i];
        }
        BeefyClient.ValidatorProof[] memory ps = new BeefyClient.ValidatorProof[](SAMPLE);
        uint256 j;
        for (uint256 i = 0; i < N; i++) {
            if (sample[0] >> i & 1 == 0) continue;
            bytes32 signed = mask >> i & 1 == 1 ? h : keccak256("honest message");
            (uint8 v, bytes32 r, bytes32 s) = vm.sign(keys[i], signed);
            ps[j++] = BeefyClient.ValidatorProof(v, r, s, i, signers[i], paths[i]);
        }
        return CompactProofLib.toCompact(ps, N);
    }

    function noLeaf() internal pure returns (BeefyClient.MMRLeaf memory leaf) {}
}
