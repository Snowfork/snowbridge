// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.5;

import "./BeefyLightClient.sol";
import "./utils/MerkleProof.sol";
import "./ScaleCodec.sol";
import "./SimplifiedMMRVerification.sol";

contract ParachainLightClient {

    using ScaleCodec for uint32;

    BeefyLightClient public immutable client;
    bytes4 public immutable encodedParachainID;

    struct Head {
        bytes32 parentHash;
        uint32 number;
        bytes32 stateRoot;
        bytes32 extrinsicsRoot;
        bytes32 commitment;
    }

    struct HeadProof {
        uint256 pos;
        uint256 width;
        bytes32[] proof;
    }

    struct MMRLeafPartial {
        uint8 version;
        uint32 parentNumber;
        bytes32 parentHash;
        uint64 nextAuthoritySetId;
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

    constructor(BeefyLightClient _client, uint32 parachainID) {
        client = _client;
        encodedParachainID = ScaleCodec.encode32(parachainID);
    }

    function verifyCommitment(
        bytes32 commitment,
        Proof calldata proof
    ) external view {
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

        // Verify inclusion of the leaf in the MMR
        require(
            client.verifyMMRLeaf(
                leafHash,
                proof.leafProof
            ),
            "Invalid proof"
        );
    }

    function createParachainMerkleLeaf(
        bytes32 commitment,
        bytes calldata headPrefix,
        bytes calldata headSuffix
    )
        internal
        pure
        returns (bytes32)
    {
        bytes memory encodedHead = bytes.concat(
            encodedParachainID,
            headPrefix,
            commitment,
            headSuffix
        );
        return keccak256(encodedHead);
    }

    function createMMRLeaf(
        MMRLeafPartial calldata leaf,
        bytes32 parachainHeadsRoot
    )
        internal
        pure
        returns (bytes32)
    {
        bytes memory encodedLeaf = bytes.concat(
            ScaleCodec.encode8(leaf.version),
            ScaleCodec.encode32(leaf.parentNumber),
            leaf.parentHash,
            ScaleCodec.encode64(leaf.nextAuthoritySetId),
            ScaleCodec.encode32(leaf.nextAuthoritySetLen),
            leaf.nextAuthoritySetRoot,
            parachainHeadsRoot
        );
        return keccak256(encodedLeaf);
    }
}
