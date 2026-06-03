// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.34;

// Production-parity check for the SubstrateMerkleProof aliasing fix.
//
// Replays the validator merkle proofs from two real mainnet transactions against BOTH the
// original (pre-fix) verify and the patched verify, and asserts they behave IDENTICALLY:
//   submitInitial: 0xaf3a30dfa2ae6acb0217e50fedfbd864401101c747d5df7521ca9a4b3b946c2b
//   submitFinal:   0xe2976fbbb5ae2e8f449ebf2b37576165f32a833e39befd190038690c05154e22
// (BeefyClient 0x7cfc5C8b341991993080Af67D940B6aD19a010E1).
//
// The genuine validator-set root/length used by both txs (read from forked mainnet state at the
// relevant blocks via `cast call --block`): currentValidatorSet id=5010 and nextValidatorSet
// id=5011 both have root=0xaa35b5...64a7 and length=600 (a handover with no membership change).
// The commitments carry validatorSetID 5011. We pin those as constants and confirm each real proof
// recomputes exactly that root, so the production proofs verify true under BOTH implementations.
//
// The fix only changes the FAILURE mode (malformed proofs -> false / InvalidMerkleProof); for the
// canonical proofs these production txs carry, old and new compute the same root and both return
// true -- i.e. no regression for valid proofs.
//
// Run: forge test --match-path test/SubstrateMerkleProofProdParity.t.sol -vv

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

contract SubstrateMerkleProofProdParityTest is Test {
    // Genuine on-chain validator set used by both txs (validatorSetID 5011), read from mainnet.
    bytes32 constant ROOT = 0xaa35b595056b5e391b82c64a3d06a230eb32970fb39e798200e568fc67ff64a7;
    uint256 constant WIDTH = 600;

    NewVerifyHarness newH;

    function setUp() public {
        newH = new NewVerifyHarness();
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

    function _assertProofParity(bytes32 leaf, uint256 index, bytes32[] memory proof) internal {
        bool oldR = OldSubstrateMerkleProof.verify(ROOT, leaf, index, WIDTH, proof);
        bool newR = newH.verify(ROOT, leaf, index, WIDTH, proof);
        // Verifies true under the original impl => decode + (root,width) are correct and it is a
        // genuine production proof.
        assertTrue(oldR, "production proof must verify under the ORIGINAL impl");
        // The fix must not change the result for this valid proof.
        assertEq(oldR, newR, "patched verify must match original for this real proof");
    }

    function test_submitInitial_prodParity() public {
        bytes memory cd = vm.parseBytes(vm.readFile("test/data/prod_submitInitial.hex"));
        (,, BeefyClient.ValidatorProof memory p) = this.decodeInitial(cd);

        _assertProofParity(keccak256(abi.encodePacked(p.account)), p.index, p.proof);
        emit log_named_uint("submitInitial: proofs checked (old==new==true)", 1);
    }

    function test_submitFinal_prodParity() public {
        bytes memory cd = vm.parseBytes(vm.readFile("test/data/prod_submitFinal.hex"));
        (,, BeefyClient.ValidatorProof[] memory ps) = this.decodeFinal(cd);
        assertGt(ps.length, 0, "expected validator proofs");

        for (uint256 k = 0; k < ps.length; k++) {
            _assertProofParity(keccak256(abi.encodePacked(ps[k].account)), ps[k].index, ps[k].proof);
        }
        emit log_named_uint("submitFinal: proofs checked (old==new==true)", ps.length);
    }
}
