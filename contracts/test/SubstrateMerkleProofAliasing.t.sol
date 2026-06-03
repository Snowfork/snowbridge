// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.34;

// Regression tests for the lone-promoted-leaf merkle-proof aliasing fix in
// src/utils/SubstrateMerkleProof.sol.
//
// Before the fix, `computeRoot` folded exactly `proof.length` siblings without binding the proof
// to a canonical leaf position. A leaf promoted up an odd-width tree has a SHORT proof, and that
// one short proof verified at every index sharing its leading decision bits (e.g. in an n=513
// tree the trailing leaf's length-1 proof verified at all 256 odd indices), letting one validator
// answer many quorum slots. After the fix, `computeRoot` walks the tree by geometry (promoting
// lone trailing nodes, consuming a proof element only where a real sibling exists) and requires
// the proof length to match the canonical path exactly -- a short proof verifies at its own index
// ONLY.
//
// Two parts, sharing the promote-tree/aliasing helpers in MerkleLibSubstrate (test/utils/MerkleLib.sol):
//   (A) SubstrateMerkleProofAliasingTest — minimal library-level regression: canonical proofs
//       still verify; a lone-promoted leaf no longer aliases; wrong-length proofs are rejected;
//       computeRoot reverts on a structurally invalid proof.
//   (B) BeefyClientForgeRejectionTest / FiatShamirForgeRejectionTest — end-to-end forge-attempt
//       scenarios (interactive + Fiat-Shamir, at the live validator count), converted to assert
//       the FIXED behavior: the forge constructions are now REJECTED with InvalidValidatorProof.
//
// Tree construction matches substrate's binary-merkle-tree (lone odd node PROMOTED unchanged); see
// the references in src/utils/SubstrateMerkleProof.sol.
//
// Run: forge test --match-path test/SubstrateMerkleProofAliasing.t.sol -vv

import {Test} from "forge-std/Test.sol";
import {SubstrateMerkleProof} from "../src/utils/SubstrateMerkleProof.sol";
import {BeefyClient} from "../src/BeefyClient.sol";
import {BeefyClientMock} from "./mocks/BeefyClientMock.sol";
import {MerkleLibSubstrate} from "./utils/MerkleLib.sol";

/// memory->calldata bridge so we exercise the REAL on-chain library code
contract SubMerkleHarness {
    function verify(
        bytes32 root,
        bytes32 leaf,
        uint256 pos,
        uint256 width,
        bytes32[] calldata proof
    ) external pure returns (bool) {
        return SubstrateMerkleProof.verify(root, leaf, pos, width, proof);
    }
}

contract ComputeRootHarness {
    function computeRoot(bytes32 leaf, uint256 pos, uint256 width, bytes32[] calldata proof)
        external
        pure
        returns (bytes32)
    {
        return SubstrateMerkleProof.computeRoot(leaf, pos, width, proof);
    }
}

// ============================================================================================
// (A) Minimal library-level regression.
// ============================================================================================
contract SubstrateMerkleProofAliasingTest is Test {
    // n = 2^9 + 1: the trailing leaf (index = width - 1) is promoted all the way to a length-1
    // proof, which pre-fix aliased onto all 256 odd indices.
    uint256 constant PROMO_WIDTH = 513;
    uint256 constant PROMO_LEAF = PROMO_WIDTH - 1; // trailing leaf = last index

    SubMerkleHarness h;

    function setUp() public {
        h = new SubMerkleHarness();
    }

    // 1. Every canonical proof still verifies at its own index, across power-of-two and odd
    //    widths (no false negatives introduced by the fix).
    function test_canonicalProofsStillVerify() public view {
        uint256[5] memory widths = [uint256(16), 15, 17, 513, 600];
        for (uint256 wi = 0; wi < widths.length; wi++) {
            uint256 n = widths[wi];
            bytes32[] memory leaves = MerkleLibSubstrate.genLeaves(n);
            (bytes32[][] memory L, bytes32 root) = MerkleLibSubstrate.buildLevels(leaves);
            for (uint256 p = 0; p < n; p++) {
                bytes32[] memory pr = MerkleLibSubstrate.proofFromLevels(L, p);
                assertTrue(h.verify(root, leaves[p], p, n, pr), "canonical proof must verify");
            }
        }
    }

    // 2. THE FIX. A lone-promoted leaf's short proof must verify at its own index ONLY -- never
    //    at the odd indices it used to alias onto.
    function test_promotedLeafNoLongerAliases() public view {
        uint256 n = PROMO_WIDTH;
        bytes32[] memory leaves = MerkleLibSubstrate.genLeaves(n);
        (bytes32[][] memory L, bytes32 root) = MerkleLibSubstrate.buildLevels(leaves);

        bytes32[] memory pr = MerkleLibSubstrate.proofFromLevels(L, PROMO_LEAF);
        assertEq(pr.length, 1, "trailing leaf has a length-1 proof");

        // Own index: verifies.
        assertTrue(
            h.verify(root, leaves[PROMO_LEAF], PROMO_LEAF, n, pr), "must verify at its true index"
        );

        // Every other index (previously 256 odd-index aliases): must NOT verify.
        uint256 aliasHits;
        for (uint256 X = 0; X < n; X++) {
            if (X == PROMO_LEAF) continue;
            if (h.verify(root, leaves[PROMO_LEAF], X, n, pr)) aliasHits++;
        }
        assertEq(aliasHits, 0, "short proof must not alias onto any other index");
    }

    // 3. Hand-checkable n=3 micro-case: leaves [m0,m1,m2]; (m0,m1)->h01; m2 promoted;
    //    root = hash(h01, m2); m2's proof = [h01]. Pre-fix this also verified at odd index 1.
    function test_microCase_n3_noAlias() public view {
        bytes32 m0 = keccak256("m0");
        bytes32 m1 = keccak256("m1");
        bytes32 m2 = keccak256("m2");
        bytes32 h01 = MerkleLibSubstrate.hashPair(m0, m1);
        bytes32 root = MerkleLibSubstrate.hashPair(h01, m2);
        bytes32[] memory pr = new bytes32[](1);
        pr[0] = h01;

        assertTrue(h.verify(root, m2, 2, 3, pr), "m2 verifies at its own index 2");
        assertFalse(h.verify(root, m2, 1, 3, pr), "must NOT alias onto index 1 (the old bug)");
        assertFalse(h.verify(root, m2, 0, 3, pr), "must not verify at index 0");
    }

    // 4. A proof of the wrong length for its position does not verify (verify returns false,
    //    preserving its bool contract -- it must never revert).
    function test_wrongLengthProofRejected() public view {
        uint256 n = 600;
        bytes32[] memory leaves = MerkleLibSubstrate.genLeaves(n);
        (bytes32[][] memory L, bytes32 root) = MerkleLibSubstrate.buildLevels(leaves);

        bytes32[] memory longProof = MerkleLibSubstrate.proofFromLevels(L, 0); // full-depth proof
        bytes32[] memory shortProof = MerkleLibSubstrate.proofFromLevels(L, 599); // promoted-path, shorter
        assertGt(longProof.length, shortProof.length, "interior proof is longer than promoted one");

        // Too long for the promoted index, and too short for an interior index: both rejected.
        assertFalse(h.verify(root, leaves[599], 599, n, longProof), "too-long proof rejected");
        assertFalse(h.verify(root, leaves[0], 0, n, shortProof), "too-short proof rejected");
    }

    // 5. The public computeRoot wrapper reverts on a structurally invalid proof (this is the
    //    entry point used by Verification.sol for parachain-head proofs).
    function test_computeRoot_revertsOnInvalid() public {
        uint256 n = 600;
        bytes32[] memory leaves = MerkleLibSubstrate.genLeaves(n);
        (bytes32[][] memory L,) = MerkleLibSubstrate.buildLevels(leaves);
        bytes32[] memory shortProof = MerkleLibSubstrate.proofFromLevels(L, 599);

        ComputeRootHarness c = new ComputeRootHarness();
        vm.expectRevert(SubstrateMerkleProof.InvalidMerkleProof.selector);
        c.computeRoot(leaves[0], 0, n, shortProof);
    }
}

// ============================================================================================
// (B) End-to-end forge-attempt scenarios, converted to assert the FIXED behavior. The attack
// construction (the CTRL index sets, the aliasing/grind logic) is preserved so we prove the fix
// defeats the real exploit inputs -- they are now REJECTED rather than forging latestMMRRoot.
//   (1) test_fix_proofNoLongerAliases            — the short proof verifies at its true index ONLY.
//   (2) test_fix_interactiveForgeRejected         — submitFinal reverts InvalidValidatorProof.
//   (3) test_fix_fiatShamirForgeRejected          — submitFiatShamir reverts InvalidValidatorProof.
//   (4) test_fix_forgeRejectedAtLiveValidatorCount — forks mainnet to read the REAL validator
//       count/set-id, then shows a PATCHED BeefyClient at that production scale rejects the forge.
//       NOTE: the contract currently deployed at LIVE predates this fix; it carries the fix only
//       once upgraded.
// ============================================================================================
contract BeefyClientForgeRejectionTest is Test {
    address constant LIVE = 0x6eD05bAa904df3DE117EcFa638d4CB84e1B8A00C;

    uint256 constant N = 600; // validator set size
    // Interactive path: prevRandao isn't grindable, so the random subsample can hit any claimed
    // bit -> the answerable union must reach QUORUM directly (no padding). That needs the full
    // 153-index set (~25.5%). (Contrast the Fiat-Shamir path, which pads + grinds with fewer.)
    uint256 constant CONTROLLED = 153;
    uint256 constant QUORUM = 401; // computeQuorum(600) = N - (N-1)/3
    // interactive subsample size = computeNumRequiredSignatures(600, freshUsage, MIN_SIGS)
    //                            = MIN_SIGS + ceil(log2(600)) + 1 = 17 + 10 + 1
    uint256 constant REQUIRED_SIGS = 28;
    uint256 constant PREVRANDAO_SEED = 0xBEEF;
    uint64 constant SET_ID = 77;

    // BeefyClientMock(randaoCommitDelay, randaoCommitExpiration, minNumRequiredSignatures,
    //                 fiatShamirRequiredSignatures, initialBeefyBlock, ...)
    uint256 constant RANDAO_DELAY = 128;
    uint256 constant RANDAO_EXPIRY = 24;
    uint256 constant MIN_SIGS = 17;
    uint256 constant FS_SIGS = 111;

    // High-reach index set for n=600: low fillers [0,55] U [64,80] plus the trailing high-reach
    // block [512,567] U [576,599]. CONTROLLED indices whose aliasing union reaches QUORUM.
    uint16[] CTRL;

    address[CONTROLLED] cAddr;
    uint256[CONTROLLED] cPk;
    bytes32[][CONTROLLED] cProof;
    bytes32 vroot;
    bytes32 forged;
    bytes32 ch;
    uint128 setId;
    uint64 blockNum;

    function setUp() public {
        _appendRange(0, 55);
        _appendRange(64, 80);
        _appendRange(512, 567);
        _appendRange(576, 599);
        require(CTRL.length == CONTROLLED, "CTRL length != CONTROLLED");
    }

    function _appendRange(uint16 lo, uint16 hi) internal {
        for (uint16 i = lo; i <= hi; i++) {
            CTRL.push(i);
        }
    }

    // ============================================================================================
    // (1) FIX: a lone-promoted leaf's short proof verifies at its TRUE index only — no aliasing.
    // ============================================================================================
    function test_fix_proofNoLongerAliases() public {
        SubMerkleHarness h = new SubMerkleHarness();

        // Hand-checkable micro-case (n=3): leaves [m0,m1,m2]; (m0,m1)->h01; m2 lone-promoted;
        // root = hash(h01, m2); m2's canonical proof is [h01]. Pre-fix this also verified at the
        // odd index 1; post-fix it verifies at its own index 2 only.
        {
            bytes32 m0 = keccak256("m0");
            bytes32 m1 = keccak256("m1");
            bytes32 m2 = keccak256("attacker-m2");
            bytes32 h01 = keccak256(abi.encodePacked(m0, m1));
            bytes32 root3 = keccak256(abi.encodePacked(h01, m2));
            bytes32[] memory hp = new bytes32[](1);
            hp[0] = h01;
            assertTrue(h.verify(root3, m2, 2, 3, hp), "verifies at its own index 2");
            assertFalse(h.verify(root3, m2, 1, 3, hp), "FIX: no longer aliases onto index 1");
            assertFalse(h.verify(root3, m2, 0, 3, hp), "does not verify at index 0");
        }

        uint256 n = 513; // 2^9 + 1: the trailing leaf is lone-promoted -> length-1 proof
        uint256 trailing = 512;
        (address attacker,) = makeAddrAndKey("attacker-A");
        bytes32[] memory leaves = new bytes32[](n);
        for (uint256 i = 0; i < n; i++) {
            leaves[i] = keccak256(abi.encodePacked("v:", i));
        }
        leaves[trailing] = keccak256(abi.encodePacked(attacker));
        (bytes32[][] memory L, bytes32 root) = MerkleLibSubstrate.buildLevels(leaves);
        bytes32[] memory aProof = MerkleLibSubstrate.proofFromLevels(L, trailing);

        assertEq(aProof.length, 1, "lone-promoted trailing leaf has a length-1 proof");
        assertTrue(
            h.verify(root, leaves[trailing], trailing, n, aProof), "verifies at its true index 512"
        );
        // Pre-fix this proof verified at all 256 odd indices; post-fix it verifies nowhere else.
        uint256 spurious;
        for (uint256 X = 0; X < n; X++) {
            if (X == trailing) continue;
            if (h.verify(root, leaves[trailing], X, n, aProof)) spurious++;
        }
        assertEq(spurious, 0, "FIX: short proof aliases onto no other index");
    }

    // ============================================================================================
    // (2)/(4) FIX: interactive forge (submitInitial -> commitPrevRandao -> submitFinal) rejected.
    // ============================================================================================
    function _commit() internal view returns (BeefyClient.Commitment memory c) {
        BeefyClient.PayloadItem[] memory pl = new BeefyClient.PayloadItem[](1);
        pl[0] = BeefyClient.PayloadItem({payloadID: bytes2("mh"), data: abi.encodePacked(forged)});
        c = BeefyClient.Commitment({
            blockNumber: uint32(blockNum), validatorSetID: uint64(setId), payload: pl
        });
    }

    // Builds an aliased proof: claims controlled validator i sits at index X.
    function _answer(uint256 X) internal view returns (BeefyClient.ValidatorProof memory) {
        for (uint256 i = 0; i < CONTROLLED; i++) {
            if (MerkleLibSubstrate.aliases(CTRL[i], X, cProof[i].length, N)) {
                (uint8 v, bytes32 r, bytes32 s) = vm.sign(cPk[i], ch);
                return BeefyClient.ValidatorProof({
                    v: v, r: r, s: s, index: X, account: cAddr[i], proof: cProof[i]
                });
            }
        }
        revert("unanswerable");
    }

    // Shared setup: build the attacker set + claimed bitfield, deploy a patched mock, run the
    // interactive path up to (but not including) submitFinal, returning the prepared proofs.
    function _setupInteractive(uint64 _setId)
        internal
        returns (
            BeefyClientMock bc,
            uint256[] memory bf,
            BeefyClient.ValidatorProof[] memory proofs
        )
    {
        setId = _setId;
        bytes32[] memory leaves = new bytes32[](N);
        for (uint256 p = 0; p < N; p++) {
            leaves[p] = keccak256(abi.encodePacked("v:", p));
        }
        for (uint256 i = 0; i < CONTROLLED; i++) {
            (address a, uint256 pk) = makeAddrAndKey(string.concat("c-", vm.toString(i)));
            cAddr[i] = a;
            cPk[i] = pk;
            leaves[CTRL[i]] = keccak256(abi.encodePacked(a));
        }
        (bytes32[][] memory L, bytes32 r) = MerkleLibSubstrate.buildLevels(leaves);
        vroot = r;
        for (uint256 i = 0; i < CONTROLLED; i++) {
            cProof[i] = MerkleLibSubstrate.proofFromLevels(L, CTRL[i]);
        }

        bf = new uint256[]((N + 255) / 256);
        uint256 bits;
        for (uint256 i = 0; i < CONTROLLED; i++) {
            for (uint256 X = 0; X < N; X++) {
                if (MerkleLibSubstrate.aliases(CTRL[i], X, cProof[i].length, N)) {
                    if ((bf[X >> 8] >> (X & 0xff)) & 1 == 0) bits++;
                    bf[X >> 8] |= (uint256(1) << (X & 0xff));
                }
            }
        }
        assertGe(bits, QUORUM, "answerable union reaches computeQuorum(600) (quorum gate passes)");

        bc = new BeefyClientMock(
            RANDAO_DELAY,
            RANDAO_EXPIRY,
            MIN_SIGS,
            FS_SIGS,
            0,
            BeefyClient.ValidatorSet({id: setId, length: uint128(N), root: vroot}),
            BeefyClient.ValidatorSet({id: setId + 1, length: uint128(N), root: vroot})
        );

        forged = keccak256("ATTACKER-FORGED-MMR-ROOT");
        blockNum = bc.latestBeefyBlock() + 1;
        ch = keccak256(bc.encodeCommitment_public(_commit()));

        {
            (uint8 v, bytes32 rr, bytes32 s) = vm.sign(cPk[0], ch);
            bc.submitInitial(
                _commit(),
                bf,
                BeefyClient.ValidatorProof({
                    v: v, r: rr, s: s, index: CTRL[0], account: cAddr[0], proof: cProof[0]
                })
            );
        }
        vm.roll(block.number + bc.randaoCommitDelay() + 2);
        vm.prevrandao(bytes32(uint256(PREVRANDAO_SEED)));
        bc.commitPrevRandao(ch);

        uint256[] memory sampled = _subsample(PREVRANDAO_SEED, bf, REQUIRED_SIGS);
        proofs = new BeefyClient.ValidatorProof[](REQUIRED_SIGS);
        for (uint256 j = 0; j < REQUIRED_SIGS; j++) {
            proofs[j] = _answer(sampled[j]);
        }
    }

    function test_fix_interactiveForgeRejected() public {
        (BeefyClientMock bc, uint256[] memory bf, BeefyClient.ValidatorProof[] memory proofs) =
            _setupInteractive(SET_ID);

        BeefyClient.MMRLeaf memory leaf;
        bytes32[] memory empty = new bytes32[](0);
        // FIX: the aliased proofs no longer pass isValidatorInSet -> submitFinal reverts.
        vm.expectRevert(BeefyClient.InvalidValidatorProof.selector);
        bc.submitFinal(_commit(), bf, proofs, leaf, empty, 0);

        assertTrue(bc.latestMMRRoot() != forged, "root of trust NOT forged");
        assertEq(bc.latestMMRRoot(), bytes32(0), "latestMMRRoot unchanged (forge rejected)");
    }

    function test_fix_forgeRejectedAtLiveValidatorCount() public {
        vm.createSelectFork("https://ethereum-rpc.publicnode.com");
        (uint128 id, uint128 len,,) = BeefyClient(LIVE).currentValidatorSet();
        assertEq(len, N, "live validator count read from mainnet is 600");

        (BeefyClientMock bc, uint256[] memory bf, BeefyClient.ValidatorProof[] memory proofs) =
            _setupInteractive(uint64(id));

        BeefyClient.MMRLeaf memory leaf;
        bytes32[] memory empty = new bytes32[](0);
        vm.expectRevert(BeefyClient.InvalidValidatorProof.selector);
        bc.submitFinal(_commit(), bf, proofs, leaf, empty, 0);

        assertTrue(
            bc.latestMMRRoot() != forged, "patched contract at live count rejects the forge"
        );
    }

    function _subsample(uint256 seed, uint256[] memory prior, uint256 cnt)
        internal
        pure
        returns (uint256[] memory out)
    {
        out = new uint256[](cnt);
        uint256[] memory seen = new uint256[](prior.length);
        uint256 f;
        for (uint256 i = 0; f < cnt;) {
            uint256 X = uint256(keccak256(abi.encodePacked(seed, i))) % N;
            if ((prior[X >> 8] >> (X & 0xff)) & 1 == 0 || (seen[X >> 8] >> (X & 0xff)) & 1 == 1) {
                unchecked {
                    i++;
                }
                continue;
            }
            seen[X >> 8] |= (uint256(1) << (X & 0xff));
            out[f] = X;
            unchecked {
                f++;
                i++;
            }
        }
    }
}

// ============================================================================================
// (3) FIX: the single-tx Fiat-Shamir forge is rejected. The attacker still grinds blockNumber
// offline to bias the subsample (that modelling is unaffected), and the claimed bitfield still
// satisfies the quorum check — but submitFiatShamir now reverts InvalidValidatorProof
// because the aliased proofs fail the (now position-bound) membership check.
// ============================================================================================
contract FiatShamirForgeRejectionTest is Test {
    uint256 constant N = 600; // validator set size
    uint128 constant SET_ID = 99;
    // Fiat-Shamir path: blockNumber grinding biases the subsample, so the claimed bitfield can be
    // padded to QUORUM with non-answerable fillers and fewer validators suffice -> ~23.3%. This
    // lower stake (vs the interactive 25.5%) is the "amplification" the original PoC demonstrated.
    uint256 constant CONTROLLED = 140;
    uint256 constant QUORUM = 401; // computeQuorum(600)
    uint256 constant FS_SIGS = 111; // fiatShamirRequiredSignatures = Fiat-Shamir subsample size
    uint256 constant GRIND_BUDGET = 200_000; // max blockNumber grind iterations before giving up

    // BeefyClientMock(randaoCommitDelay, randaoCommitExpiration, minNumRequiredSignatures, ...)
    uint256 constant RANDAO_DELAY = 128;
    uint256 constant RANDAO_EXPIRY = 24;
    uint256 constant MIN_SIGS = 17;

    // High-reach index set for n=600: low fillers [0,55] U [64,67] plus the trailing high-reach
    // block [512,567] U [576,599]. CONTROLLED indices.
    uint16[] CTRL;

    address[CONTROLLED] cAddr;
    uint256[CONTROLLED] cPk;
    bytes32[][CONTROLLED] cProof;
    bool[N] ans;
    bytes32 vRoot;
    bytes32 forged;
    bytes32 ch;
    uint256[] bitfield;
    BeefyClientMock bc;

    function setUp() public {
        _appendRange(0, 55);
        _appendRange(64, 67);
        _appendRange(512, 567);
        _appendRange(576, 599);
        require(CTRL.length == CONTROLLED, "CTRL length != CONTROLLED");
    }

    function _appendRange(uint16 lo, uint16 hi) internal {
        for (uint16 i = lo; i <= hi; i++) {
            CTRL.push(i);
        }
    }

    function _set(uint256[] memory bf, uint256 i) internal pure {
        bf[i >> 8] |= (uint256(1) << (i & 0xff));
    }

    function _is(uint256[] memory bf, uint256 i) internal pure returns (bool) {
        return (bf[i >> 8] >> (i & 0xff)) & 1 == 1;
    }

    function _sub(uint256 seed, uint256[] memory prior, uint256 cnt)
        internal
        pure
        returns (uint256[] memory out)
    {
        out = new uint256[](cnt);
        uint256[] memory seen = new uint256[](prior.length);
        uint256 f;
        for (uint256 i = 0; f < cnt;) {
            uint256 X = uint256(keccak256(abi.encodePacked(seed, i))) % N;
            if (!_is(prior, X) || _is(seen, X)) {
                unchecked {
                    i++;
                }
                continue;
            }
            _set(seen, X);
            out[f] = X;
            unchecked {
                f++;
                i++;
            }
        }
    }

    function _fsSeed(bytes32 cmh, bytes32 bfh) internal view returns (uint256) {
        bytes32 inner =
            sha256(bytes.concat(cmh, bfh, vRoot, bytes32(uint256(SET_ID)), bytes32(uint256(N))));
        return uint256(sha256(bytes.concat(bc.FIAT_SHAMIR_DOMAIN_ID(), inner)));
    }

    function _commit(uint64 bn) internal view returns (BeefyClient.Commitment memory c) {
        BeefyClient.PayloadItem[] memory pl = new BeefyClient.PayloadItem[](1);
        pl[0] = BeefyClient.PayloadItem({payloadID: bytes2("mh"), data: abi.encodePacked(forged)});
        c = BeefyClient.Commitment({
            blockNumber: uint32(bn), validatorSetID: uint64(SET_ID), payload: pl
        });
    }

    function _answer(uint256 X) internal view returns (BeefyClient.ValidatorProof memory) {
        for (uint256 i = 0; i < CONTROLLED; i++) {
            if (MerkleLibSubstrate.aliases(CTRL[i], X, cProof[i].length, N)) {
                (uint8 v, bytes32 r, bytes32 s) = vm.sign(cPk[i], ch);
                return BeefyClient.ValidatorProof({
                    v: v, r: r, s: s, index: X, account: cAddr[i], proof: cProof[i]
                });
            }
        }
        revert("no");
    }

    function test_fix_fiatShamirForgeRejected() public {
        bytes32[] memory lv = new bytes32[](N);
        for (uint256 p = 0; p < N; p++) {
            lv[p] = keccak256(abi.encodePacked("v:", p));
        }
        for (uint256 i = 0; i < CONTROLLED; i++) {
            (address a, uint256 pk) = makeAddrAndKey(string.concat("f-", vm.toString(i)));
            cAddr[i] = a;
            cPk[i] = pk;
            lv[CTRL[i]] = keccak256(abi.encodePacked(a));
        }
        (bytes32[][] memory L, bytes32 r) = MerkleLibSubstrate.buildLevels(lv);
        vRoot = r;
        for (uint256 i = 0; i < CONTROLLED; i++) {
            cProof[i] = MerkleLibSubstrate.proofFromLevels(L, CTRL[i]);
            for (uint256 X = 0; X < N; X++) {
                if (MerkleLibSubstrate.aliases(CTRL[i], X, cProof[i].length, N)) ans[X] = true;
            }
        }

        // claimed bitfield = answerable union, PADDED to >= QUORUM (passes the quorum gate)
        uint256[] memory bf = new uint256[]((N + 255) / 256);
        uint256 bits;
        for (uint256 X = 0; X < N; X++) {
            if (ans[X]) {
                _set(bf, X);
                bits++;
            }
        }
        for (uint256 X = 0; bits < QUORUM && X < N; X++) {
            if (!ans[X] && !_is(bf, X)) {
                _set(bf, X);
                bits++;
            }
        }
        bitfield = bf;
        assertGe(bits, QUORUM, "claimed bitfield satisfies the quorum check");

        bc = new BeefyClientMock(
            RANDAO_DELAY,
            RANDAO_EXPIRY,
            MIN_SIGS,
            FS_SIGS,
            0,
            BeefyClient.ValidatorSet({id: SET_ID, length: uint128(N), root: vRoot}),
            BeefyClient.ValidatorSet({id: SET_ID + 1, length: uint128(N), root: vRoot})
        );
        forged = keccak256("ATTACKER-FORGED-ROOT-FS");
        bytes32 bfh = keccak256(abi.encodePacked(bitfield));

        // The attacker's offline grind is UNAFFECTED by the fix: bias blockNumber until the
        // Fiat-Shamir subsample (FS_SIGS) lands entirely in the would-be-answerable set.
        uint64 bn;
        uint256 tries;
        uint256[] memory sampled;
        for (bn = 1;; bn++) {
            tries++;
            bytes32 cmh = keccak256(bc.encodeCommitment_public(_commit(bn)));
            sampled = _sub(_fsSeed(cmh, bfh), bitfield, FS_SIGS);
            bool ok = true;
            for (uint256 j = 0; j < FS_SIGS; j++) {
                if (!ans[sampled[j]]) {
                    ok = false;
                    break;
                }
            }
            if (ok) {
                ch = cmh;
                break;
            }
            require(tries < GRIND_BUDGET, "grind budget");
        }

        BeefyClient.ValidatorProof[] memory proofs = new BeefyClient.ValidatorProof[](FS_SIGS);
        for (uint256 j = 0; j < FS_SIGS; j++) {
            proofs[j] = _answer(sampled[j]);
        }
        BeefyClient.MMRLeaf memory leaf;
        bytes32[] memory empty = new bytes32[](0);

        // FIX: even with a winning grind and a quorum-satisfying bitfield, the aliased proofs are
        // rejected -> the single-tx forge reverts.
        vm.expectRevert(BeefyClient.InvalidValidatorProof.selector);
        bc.submitFiatShamir(_commit(bn), bitfield, proofs, leaf, empty, 0);

        assertTrue(bc.latestMMRRoot() != forged, "single-tx FS forge rejected");
        emit log_named_uint("grind found a biasing blockNumber after tries", tries);
    }
}
