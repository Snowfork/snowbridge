// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.5;

import "./BeefyLightClient.sol";
import "./utils/MerkleProof.sol";
import "./ScaleCodec.sol";
import "./SimplifiedMMRVerification.sol";

library ParachainLightClient {
    struct OwnParachainHead {
        bytes32 parentHash;
        uint32 number;
        bytes32 stateRoot;
        bytes32 extrinsicsRoot;
        bytes32 commitment;
    }

    struct ParachainHeadProof {
        uint256 pos;
        uint256 width;
        bytes32[] proof;
    }

    struct BeefyMMRLeafPartial {
        uint8 version;
        uint32 parentNumber;
        bytes32 parentHash;
        uint64 nextAuthoritySetId;
        uint32 nextAuthoritySetLen;
        bytes32 nextAuthoritySetRoot;
    }

    bytes4 public constant PARACHAIN_ID_SCALE = 0xe8030000;

    struct ParachainVerifyInput {
        bytes ownParachainHeadPrefixBytes;
        bytes ownParachainHeadSuffixBytes;
        ParachainHeadProof parachainHeadProof;
    }

    function verifyCommitmentInParachain(
        bytes32 commitment,
        ParachainVerifyInput calldata _parachainVerifyInput,
        BeefyMMRLeafPartial calldata _beefyMMRLeafPartial,
        SimplifiedMMRProof calldata proof,
        BeefyLightClient beefyLightClient
    ) internal view {
        // 1. Compute our parachains merkle leaf by combining the parachain id, commitment data
        // and other misc bytes provided for the parachain header and hashing them.
        bytes32 ownParachainHeadHash = createParachainMerkleLeaf(
            _parachainVerifyInput.ownParachainHeadPrefixBytes,
            commitment,
            _parachainVerifyInput.ownParachainHeadSuffixBytes
        );

        // 2. Compute `parachainHeadsRoot` by verifying the merkle proof using `ownParachainHeadHash` and
        // `_parachainHeadsProof`
        bytes32 parachainHeadsRoot = MerkleProof.computeRootFromProofAtPosition(
            ownParachainHeadHash,
            _parachainVerifyInput.parachainHeadProof.pos,
            _parachainVerifyInput.parachainHeadProof.width,
            _parachainVerifyInput.parachainHeadProof.proof
        );

        // 3. Compute the `beefyMMRLeaf` using `parachainHeadsRoot` and `_beefyMMRLeafPartial`
        bytes32 beefyMMRLeaf = createMMRLeafHash(
            _beefyMMRLeafPartial,
            parachainHeadsRoot
        );

        // 4. Verify inclusion of the beefy MMR leaf in the beefy MMR root using that `beefyMMRLeaf` as well as
        // `_beefyMMRLeafIndex`, `_beefyMMRLeafCount` and `_beefyMMRLeafProof`
        require(
            beefyLightClient.verifyBeefyMerkleLeaf(
                beefyMMRLeaf,
                proof
            ),
            "Invalid proof"
        );
    }

    function createParachainMerkleLeaf(
        bytes calldata _ownParachainHeadPrefixBytes,
        bytes32 commitment,
        bytes calldata _ownParachainHeadSuffixBytes
    ) public pure returns (bytes32) {
        bytes memory scaleEncodedParachainHead = bytes.concat(
            PARACHAIN_ID_SCALE,
            _ownParachainHeadPrefixBytes,
            commitment,
            _ownParachainHeadSuffixBytes
        );

        return keccak256(scaleEncodedParachainHead);
    }

    function createMMRLeafHash(
        BeefyMMRLeafPartial calldata leaf,
        bytes32 parachainHeadsRoot
    ) public pure returns (bytes32) {
        bytes memory scaleEncodedMMRLeaf = abi.encodePacked(
            ScaleCodec.encode8(leaf.version),
            ScaleCodec.encode32(leaf.parentNumber),
            leaf.parentHash,
            ScaleCodec.encode64(leaf.nextAuthoritySetId),
            ScaleCodec.encode32(leaf.nextAuthoritySetLen),
            leaf.nextAuthoritySetRoot,
            parachainHeadsRoot
        );

        return keccak256(scaleEncodedMMRLeaf);
    }
}
