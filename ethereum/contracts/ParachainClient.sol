// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "./BeefyClient.sol";
import "./utils/MerkleProof.sol";
import "./ScaleCodec.sol";
import "./utils/MMRProofVerification.sol";

contract ParachainClient {
    BeefyClient public immutable beefyClient;
    uint32 public immutable parachainID;
    bytes4 public immutable encodedParachainID;

    struct HeadProof {
        uint256 pos;
        uint256 width;
        bytes32[] proof;
    }

    struct MMRLeafPartial {
        uint8 version;
        uint32 parentNumber;
        bytes32 parentHash;
        uint64 nextAuthoritySetID;
        uint32 nextAuthoritySetLen;
        bytes32 nextAuthoritySetRoot;
    }

    struct Proof {
        bytes headPrefix;
        bytes headSuffix;
        HeadProof headProof;
        MMRLeafPartial leafPartial;
        MMRProof leafProof;
    }

    constructor(BeefyClient _client, uint32 _parachainID) {
        beefyClient = _client;
        parachainID = _parachainID;
        encodedParachainID = ScaleCodec.encode32(_parachainID);
    }

    function verifyCommitment(bytes32 commitment, bytes calldata opaqueProof)
        external
        view
        returns (bool)
    {
        (Proof memory proof) = abi.decode(opaqueProof, (Proof));

        // Compute the merkle leaf hash of our parachain
        bytes32 parachainHeadHash = createParachainMerkleLeaf(
            commitment,
            proof.headPrefix,
            proof.headSuffix
        );

        // Compute the merkle root hash of all parachain heads
        bytes32 parachainHeadsRoot = MerkleProof.computeRootFromProofAtPosition(
            parachainHeadHash,
            proof.headProof.pos,
            proof.headProof.width,
            proof.headProof.proof
        );

        bytes32 leafHash = createMMRLeaf(proof.leafPartial, parachainHeadsRoot);
        return beefyClient.verifyMMRLeafProof(leafHash, proof.leafProof);
    }

    function createParachainMerkleLeaf(
        bytes32 commitment,
        bytes memory headPrefix,
        bytes memory headSuffix
    ) internal view returns (bytes32) {
        bytes memory encodedHead = bytes.concat(
            encodedParachainID,
            headPrefix,
            commitment,
            headSuffix
        );
        return keccak256(encodedHead);
    }

    function createMMRLeaf(MMRLeafPartial memory leaf, bytes32 parachainHeadsRoot)
        internal
        pure
        returns (bytes32)
    {
        bytes memory encodedLeaf = bytes.concat(
            ScaleCodec.encode8(leaf.version),
            ScaleCodec.encode32(leaf.parentNumber),
            leaf.parentHash,
            ScaleCodec.encode64(leaf.nextAuthoritySetID),
            ScaleCodec.encode32(leaf.nextAuthoritySetLen),
            leaf.nextAuthoritySetRoot,
            parachainHeadsRoot
        );
        return keccak256(encodedLeaf);
    }
}
