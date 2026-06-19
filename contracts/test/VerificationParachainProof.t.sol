// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.34;

// Regression tests covering the SECOND consumer of SubstrateMerkleProof.computeRoot: the parachain
// heads merkle proof in Verification.verifyCommitment (src/Verification.sol). The validator-set
// route (BeefyClient.isValidatorInSet) is covered in SubstrateMerkleProofAliasing.t.sol; this file
// asserts the same guarantee on the parachain route.
//
// Two properties, on the path that actually runs in production (Verification.verifyCommitment):
//   (1) LIVENESS  — a canonical parachain-heads proof (the exact shape the relayer's promotion-aware
//       generator emits, including the SHORT proof of a lone-promoted leaf) still verifies, i.e. the
//       new exact-consumption check does not reject honest proofs.
//   (2) SECURITY  — a too-short or zero-padded parachain-heads proof is rejected (verifyCommitment
//       returns false at the `!valid` guard), never silently producing a wrong parachainHeadsRoot.
//
// The MMR step is satisfied with a single-leaf MMR (empty leafProof, latestMMRRoot == leafHash) so
// the test isolates the parachain-heads proof behaviour.
//
// Run: forge test --match-path test/VerificationParachainProof.t.sol -vv

import {Test} from "forge-std/Test.sol";
import {Verification} from "../src/Verification.sol";
import {BeefyClient} from "../src/BeefyClient.sol";
import {BeefyClientMock} from "./mocks/BeefyClientMock.sol";
import {VerificationWrapper} from "./mocks/VerificationWrapper.sol";
import {ScaleCodec} from "../src/utils/ScaleCodec.sol";
import {MerkleLibSubstrate} from "./utils/MerkleLib.sol";

contract VerificationParachainProofTest is Test {
    BeefyClientMock public beefyClient;
    VerificationWrapper public wrapper;

    uint32 public constant BRIDGE_HUB_PARA_ID = 1013;
    bytes4 public encodedParachainID;

    function setUp() public {
        beefyClient = new BeefyClientMock(
            3, 8, 16, 101, 0, BeefyClient.ValidatorSet(0, 0, 0x0), BeefyClient.ValidatorSet(1, 0, 0x0)
        );
        encodedParachainID = ScaleCodec.encodeU32(BRIDGE_HUB_PARA_ID);
        wrapper = new VerificationWrapper();
    }

    // Build a full, valid Verification.Proof whose parachain-heads tree has `width` leaves and proves
    // BridgeHub's header at leaf index `pos`. Wires up the single-leaf MMR so verifyCommitment passes
    // end-to-end when the headProof is canonical.
    function _buildProof(uint256 width, uint256 pos, bytes32 commitment)
        internal
        returns (Verification.Proof memory proof)
    {
        // Header carrying the commitment as a v1 Snowbridge DIGEST_ITEM_OTHER (0x00 ++ commitment).
        Verification.DigestItem[] memory digestItems = new Verification.DigestItem[](1);
        digestItems[0] = Verification.DigestItem({
            kind: 0,
            consensusEngineID: 0x00000000,
            data: bytes.concat(hex"00", commitment)
        });
        Verification.ParachainHeader memory header = Verification.ParachainHeader({
            parentHash: keccak256("parent"),
            number: 866_538,
            stateRoot: keccak256("state"),
            extrinsicsRoot: keccak256("extrinsics"),
            digestItems: digestItems
        });

        // The proven leaf is BridgeHub's parachain-head hash; the rest are arbitrary sibling heads.
        bytes32 headHash = wrapper.createParachainHeaderMerkleLeaf(encodedParachainID, header);
        bytes32[] memory leaves = new bytes32[](width);
        for (uint256 i = 0; i < width; i++) {
            leaves[i] = keccak256(abi.encodePacked("parahead:", i));
        }
        leaves[pos] = headHash;

        bytes32 parachainHeadsRoot = MerkleLibSubstrate.rootFromLeaves(leaves);
        bytes32[] memory headProof = MerkleLibSubstrate.genProof(leaves, pos);

        Verification.MMRLeafPartial memory leafPartial = Verification.MMRLeafPartial({
            version: 0,
            parentNumber: 1,
            parentHash: keccak256("mmrparent"),
            nextAuthoritySetID: 0,
            nextAuthoritySetLen: 0,
            nextAuthoritySetRoot: keccak256("authset")
        });

        // Single-leaf MMR: empty leafProof, so verifyLeafProof reduces to latestMMRRoot == leafHash.
        bytes32 leafHash = wrapper.createMMRLeaf(leafPartial, parachainHeadsRoot);
        beefyClient.setLatestMMRRoot(leafHash);

        proof = Verification.Proof({
            header: header,
            headProof: Verification.HeadProof({pos: pos, width: width, proof: headProof}),
            leafPartial: leafPartial,
            leafProof: new bytes32[](0),
            leafProofOrder: 0
        });
    }

    function _verify(Verification.Proof memory proof, bytes32 commitment) internal view returns (bool) {
        return Verification.verifyCommitment(
            address(beefyClient), encodedParachainID, commitment, proof, false
        );
    }

    // (1) LIVENESS: canonical parachain-heads proofs verify end-to-end, including a lone-promoted
    // (short) proof and an interior full-length proof.
    function testCanonicalParachainProofVerifies() public {
        bytes32 commitment = keccak256("commitment");

        // width 15, pos 14: the trailing leaf is promoted up the odd layers -> SHORT proof.
        Verification.Proof memory promoted = _buildProof(15, 14, commitment);
        assertTrue(_verify(promoted, commitment), "lone-promoted canonical proof must verify");

        // width 16, pos 5: an interior leaf with a full-length proof.
        Verification.Proof memory interior = _buildProof(16, 5, commitment);
        assertTrue(_verify(interior, commitment), "interior canonical proof must verify");
    }

    // (2) SECURITY: a too-short parachain-heads proof is rejected, not silently accepted.
    function testTooShortParachainProofRejected() public {
        bytes32 commitment = keccak256("commitment");
        Verification.Proof memory proof = _buildProof(16, 5, commitment);

        bytes32[] memory full = proof.headProof.proof;
        bytes32[] memory short = new bytes32[](full.length - 1);
        for (uint256 i = 0; i < short.length; i++) {
            short[i] = full[i];
        }
        proof.headProof.proof = short;

        assertFalse(_verify(proof, commitment), "too-short parachain proof must be rejected");
    }

    // (2) SECURITY: a zero-padded (too-long) parachain-heads proof is rejected. Pre-fix this extra
    // element was simply ignored; the exact-consumption check now rejects it.
    function testPaddedParachainProofRejected() public {
        bytes32 commitment = keccak256("commitment");
        Verification.Proof memory proof = _buildProof(16, 5, commitment);

        bytes32[] memory full = proof.headProof.proof;
        bytes32[] memory padded = new bytes32[](full.length + 1);
        for (uint256 i = 0; i < full.length; i++) {
            padded[i] = full[i];
        }
        padded[full.length] = bytes32(0);
        proof.headProof.proof = padded;

        assertFalse(_verify(proof, commitment), "zero-padded parachain proof must be rejected");
    }

    // (2) SECURITY: padding the SHORT proof of a lone-promoted leaf is the exact aliasing shape from
    // the validator-set route; assert it is rejected on the parachain route too.
    function testPaddedPromotedParachainProofRejected() public {
        bytes32 commitment = keccak256("commitment");
        Verification.Proof memory proof = _buildProof(15, 14, commitment);

        bytes32[] memory full = proof.headProof.proof;
        bytes32[] memory padded = new bytes32[](full.length + 1);
        for (uint256 i = 0; i < full.length; i++) {
            padded[i] = full[i];
        }
        padded[full.length] = bytes32(0);
        proof.headProof.proof = padded;

        assertFalse(_verify(proof, commitment), "padded lone-promoted parachain proof must be rejected");
    }
}
