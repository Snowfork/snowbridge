// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.34;

// Production verification for the SubstrateMerkleProof aliasing fix.
//
// This file merges the former ProdParity + ProdReplay suites. For each real (submitInitial,
// submitFinal) pair captured from mainnet it forks once and runs TWO complementary checks:
//
//   1. PARITY (library layer): decode the validator merkle proofs from the real calldata and run
//      each through BOTH the inlined ORIGINAL (pre-fix) verify and the PATCHED verify, asserting
//      they behave IDENTICALLY (both true). This proves the fix does not regress valid proofs. The
//      genuine validator-set root/length are read from the live contract on the fork (selected by
//      the commitment's validatorSetID), so no constants need to be maintained per pair.
//
//   2. REPLAY (contract layer): replace the live BeefyClient's CODE with the PATCHED build via
//      vm.etch (real storage — validator set, tickets, prevRandao — is preserved) and replay the
//      exact production calldata end-to-end, asserting it still succeeds (and that submitFinal
//      advances latestMMRRoot).
//
//   BeefyClient 0x7cfc5C8b341991993080Af67D940B6aD19a010E1; all pairs from relayer 0xBa9b...Ed49.
//
// Pairs (beefy block -> mainnet tx):
//   25236612/25236743 (legacy seed pair, validatorSetID 5011)
//   31600743: 0x5916adcf... / 0x0471a218...
//   31598364: 0xd04c15dc... / 0xf41aaee0...
//   31593578: 0xb1951dee... / 0xbca8dd30...
//   31591200: 0x7d8092ca... / 0xdc38e280...
//   31588809: 0xdaac6076... / 0x64812b93...
//
// Needs an archive RPC for the fork. Run:
//   forge test --match-path test/SubstrateMerkleProofProd.t.sol -vv

import {Test} from "forge-std/Test.sol";
import {SubstrateMerkleProof} from "../src/utils/SubstrateMerkleProof.sol";
import {BeefyClient} from "../src/BeefyClient.sol";

/// Inlined copy of the ORIGINAL (pre-fix) verify/computeRoot, to diff against the patched library.
library OldSubstrateMerkleProof {
    function verify(bytes32 root, bytes32 leaf, uint256 position, uint256 width, bytes32[] memory proof)
        internal
        pure
        returns (bool)
    {
        if (position >= width) {
            return false;
        }
        return root == computeRoot(leaf, position, width, proof);
    }

    function computeRoot(bytes32 leaf, uint256 position, uint256 width, bytes32[] memory proof)
        internal
        pure
        returns (bytes32)
    {
        bytes32 node = leaf;
        unchecked {
            for (uint256 i = 0; i < proof.length; i++) {
                if (position & 1 == 1 || position + 1 == width) {
                    node = efficientHash(proof[i], node);
                } else {
                    node = efficientHash(node, proof[i]);
                }
                position = position >> 1;
                width = ((width - 1) >> 1) + 1;
            }
            return node;
        }
    }

    function efficientHash(bytes32 a, bytes32 b) internal pure returns (bytes32 value) {
        assembly {
            mstore(0x00, a)
            mstore(0x20, b)
            value := keccak256(0x00, 0x40)
        }
    }
}

/// memory->calldata bridge so we exercise the REAL patched on-chain library code.
contract NewVerifyHarness {
    function verify(bytes32 root, bytes32 leaf, uint256 pos, uint256 width, bytes32[] calldata proof)
        external
        pure
        returns (bool)
    {
        return SubstrateMerkleProof.verify(root, leaf, pos, width, proof);
    }
}

contract SubstrateMerkleProofProdTest is Test {
    string constant RPC = "https://eth-mainnet.public.blastapi.io"; // archive
    address constant BC = 0x7cfc5C8b341991993080Af67D940B6aD19a010E1;
    address constant RELAYER = 0xBa9bC9a8Aa87872f7B990031bde984A00b9CEd49;

    NewVerifyHarness newH;

    struct Pair {
        uint64 beefyBlock;
        uint256 initialBlock; // mainnet block of the real submitInitial tx
        uint256 finalBlock; // mainnet block of the real submitFinal tx
        string initialTx;
        string finalTx;
        string initialFile;
        string finalFile;
    }

    function _pairs() internal pure returns (Pair[] memory pairs) {
        pairs = new Pair[](6);
        // Legacy seed pair kept for continuity.
        pairs[0] = Pair({
            beefyBlock: 0,
            initialBlock: 25236612,
            finalBlock: 25236743,
            initialTx: "0xaf3a30dfa2ae6acb0217e50fedfbd864401101c747d5df7521ca9a4b3b946c2b",
            finalTx: "0xe2976fbbb5ae2e8f449ebf2b37576165f32a833e39befd190038690c05154e22",
            initialFile: "test/data/prod_submitInitial.hex",
            finalFile: "test/data/prod_submitFinal.hex"
        });
        pairs[1] = Pair({
            beefyBlock: 31600743,
            initialBlock: 25278433,
            finalBlock: 25278572,
            initialTx: "0x5916adcf99363468982ff2c918e6501376aca6984a72806344f88899fa931b88",
            finalTx: "0x0471a218d86545e3d5c92f19201ddf4f3e320c23a8b1ca2f0553543ff2e9f1e3",
            initialFile: "test/data/prod_submitInitial_1.hex",
            finalFile: "test/data/prod_submitFinal_1.hex"
        });
        pairs[2] = Pair({
            beefyBlock: 31598364,
            initialBlock: 25277237,
            finalBlock: 25277370,
            initialTx: "0xd04c15dcc3525dca24a3914d0adae9b304c74dd47cea90f4da376a5a1e2a6e8d",
            finalTx: "0xf41aaee0c3b607b6a08e1c2834351f48fe9bfca5913380dd28589ed2dc89134c",
            initialFile: "test/data/prod_submitInitial_2.hex",
            finalFile: "test/data/prod_submitFinal_2.hex"
        });
        pairs[3] = Pair({
            beefyBlock: 31593578,
            initialBlock: 25274841,
            finalBlock: 25274972,
            initialTx: "0xb1951deee439cfbc52969d010160492e972bb6b239f2327443bf19382d4e09a1",
            finalTx: "0xbca8dd30e90f9a15c2dc20e29c36f36c098da587d89d3a7fbd2fc2aa3e1b6995",
            initialFile: "test/data/prod_submitInitial_3.hex",
            finalFile: "test/data/prod_submitFinal_3.hex"
        });
        pairs[4] = Pair({
            beefyBlock: 31591200,
            initialBlock: 25273644,
            finalBlock: 25273775,
            initialTx: "0x7d8092ca60c5e1e0861555ced80b57e5c0116e81825fbb7734ae11124e967929",
            finalTx: "0xdc38e280b428b6ac50252af2e79010de36cff07cbe853f2473a57bde8fd3b175",
            initialFile: "test/data/prod_submitInitial_4.hex",
            finalFile: "test/data/prod_submitFinal_4.hex"
        });
        pairs[5] = Pair({
            beefyBlock: 31588809,
            initialBlock: 25272445,
            finalBlock: 25272582,
            initialTx: "0xdaac6076bfebf3c51219952e7eb86ce8cc7bb49441a03f4cd733fca94146c6c9",
            finalTx: "0x64812b93dce3c2e8b252b427dd503cb5edb1221bb257f67011c0c6062d082f99",
            initialFile: "test/data/prod_submitInitial_5.hex",
            finalFile: "test/data/prod_submitFinal_5.hex"
        });
    }

    function setUp() public {
        newH = new NewVerifyHarness();
        // Forks reset all account state; keep the harness alive across createSelectFork so the
        // parity check can call the real patched library code on every fork.
        vm.makePersistent(address(newH));
    }

    // External decoders: strip the 4-byte selector and abi.decode the rest (calldata slicing).
    function decodeInitial(bytes calldata cd)
        external
        pure
        returns (BeefyClient.Commitment memory c, uint256[] memory bf, BeefyClient.ValidatorProof memory p)
    {
        (c, bf, p) = abi.decode(cd[4:], (BeefyClient.Commitment, uint256[], BeefyClient.ValidatorProof));
    }

    function decodeFinal(bytes calldata cd)
        external
        pure
        returns (BeefyClient.Commitment memory c, uint256[] memory bf, BeefyClient.ValidatorProof[] memory ps)
    {
        BeefyClient.MMRLeaf memory leaf;
        bytes32[] memory leafProof;
        uint256 order;
        (c, bf, ps, leaf, leafProof, order) = abi.decode(
            cd[4:],
            (
                BeefyClient.Commitment,
                uint256[],
                BeefyClient.ValidatorProof[],
                BeefyClient.MMRLeaf,
                bytes32[],
                uint256
            )
        );
    }

    // Replace the live contract's code with a freshly-compiled PATCHED BeefyClient that carries the
    // same immutables (read from the live contract), preserving the live storage layout/state.
    function _etchPatched() internal {
        BeefyClient live = BeefyClient(BC);
        uint256 delay = live.randaoCommitDelay();
        uint256 expiry = live.randaoCommitExpiration();
        uint256 minSigs = live.minNumRequiredSignatures();
        uint256 fsSigs = live.fiatShamirRequiredSignatures();

        // Dummy sets only satisfy the constructor; we keep only the runtime CODE, not its storage.
        BeefyClient.ValidatorSet memory d0 = BeefyClient.ValidatorSet({id: 0, length: 1, root: bytes32(0)});
        BeefyClient.ValidatorSet memory d1 = BeefyClient.ValidatorSet({id: 1, length: 1, root: bytes32(0)});
        BeefyClient patched = new BeefyClient(delay, expiry, minSigs, fsSigs, 0, d0, d1);

        vm.etch(BC, address(patched).code);
        // Sanity: the etched patched code reads the live immutables/state unchanged.
        assertEq(BeefyClient(BC).randaoCommitDelay(), delay, "immutables preserved after etch");
    }

    function _revertReason(bytes memory ret) internal pure returns (string memory) {
        if (ret.length == 0) return "(no return data)";
        return vm.toString(ret);
    }

    // Resolve the genuine validator-set (root, length) that a commitment was verified against, by
    // matching its validatorSetID to the current or next set read from the live contract on the fork.
    function _setForCommitment(uint64 vsetID) internal view returns (bytes32 root, uint256 width) {
        (uint128 cid, uint128 clen, bytes32 croot,) = BeefyClient(BC).currentValidatorSet();
        (uint128 nid, uint128 nlen, bytes32 nroot,) = BeefyClient(BC).nextValidatorSet();
        if (vsetID == cid) return (croot, clen);
        if (vsetID == nid) return (nroot, nlen);
        revert("commitment validatorSetID matches neither current nor next validator set");
    }

    // Library-layer parity: a real production proof must verify true under the ORIGINAL impl, and the
    // PATCHED impl must return the same result (the fix only changes the malformed-proof failure mode).
    function _assertProofParity(bytes32 root, uint256 width, bytes32 leaf, uint256 index, bytes32[] memory proof)
        internal
    {
        bool oldR = OldSubstrateMerkleProof.verify(root, leaf, index, width, proof);
        bool newR = newH.verify(root, leaf, index, width, proof);
        assertTrue(oldR, "production proof must verify under the ORIGINAL impl");
        assertEq(oldR, newR, "patched verify must match original for this real proof");
    }

    function _checkPair_initial(Pair memory p) internal {
        vm.createSelectFork(RPC, p.initialBlock - 1);
        bytes memory cd = vm.parseBytes(vm.readFile(p.initialFile));
        (BeefyClient.Commitment memory c,, BeefyClient.ValidatorProof memory pr) = this.decodeInitial(cd);

        // 1. Parity at the merkle-proof layer (old == new == true) against the genuine on-chain set.
        (bytes32 root, uint256 width) = _setForCommitment(c.validatorSetID);
        _assertProofParity(root, width, keccak256(abi.encodePacked(pr.account)), pr.index, pr.proof);

        // 2. End-to-end replay of the full submitInitial call on the patched code.
        _etchPatched();
        vm.roll(p.initialBlock);
        vm.prank(RELAYER);
        (bool ok, bytes memory ret) = BC.call(cd);
        assertTrue(
            ok,
            string.concat("submitInitial must succeed on patched code (", p.initialTx, "): ", _revertReason(ret))
        );
    }

    function _checkPair_final(Pair memory p) internal {
        // Fork just before submitFinal: the ticket from the real submitInitial and the committed
        // prevRandao are already in storage, so the full flow's state is present.
        vm.createSelectFork(RPC, p.finalBlock - 1);
        bytes memory cd = vm.parseBytes(vm.readFile(p.finalFile));
        (BeefyClient.Commitment memory c,, BeefyClient.ValidatorProof[] memory ps) = this.decodeFinal(cd);
        assertGt(ps.length, 0, "expected validator proofs");

        // 1. Parity at the merkle-proof layer for every validator proof in the call.
        (bytes32 root, uint256 width) = _setForCommitment(c.validatorSetID);
        for (uint256 k = 0; k < ps.length; k++) {
            _assertProofParity(root, width, keccak256(abi.encodePacked(ps[k].account)), ps[k].index, ps[k].proof);
        }

        // 2. End-to-end replay of the full submitFinal call on the patched code.
        _etchPatched();
        vm.roll(p.finalBlock);
        bytes32 rootBefore = BeefyClient(BC).latestMMRRoot();
        vm.prank(RELAYER);
        (bool ok, bytes memory ret) = BC.call(cd);
        assertTrue(
            ok, string.concat("submitFinal must succeed on patched code (", p.finalTx, "): ", _revertReason(ret))
        );

        bytes32 rootAfter = BeefyClient(BC).latestMMRRoot();
        assertTrue(rootAfter != rootBefore, "submitFinal must update latestMMRRoot");
        emit log_named_bytes32(string.concat("latestMMRRoot after ", p.finalTx), rootAfter);
    }

    function test_prod_allInitials() public {
        Pair[] memory pairs = _pairs();
        for (uint256 i = 0; i < pairs.length; i++) {
            _checkPair_initial(pairs[i]);
        }
    }

    function test_prod_allFinals() public {
        Pair[] memory pairs = _pairs();
        for (uint256 i = 0; i < pairs.length; i++) {
            _checkPair_final(pairs[i]);
        }
    }
}
