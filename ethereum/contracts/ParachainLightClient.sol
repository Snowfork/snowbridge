// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.5;

import "./BeefyLightClient.sol";
import "./utils/MerkleProof.sol";
import "./ScaleCodec.sol";

library ParachainLightClient {
    struct OwnParachainHead {
        bytes32 parentHash;
        uint32 number;
        bytes32 stateRoot;
        bytes32 extrinsicsRoot;
        bytes32 commitment; // TODO check type and position of this element
    }

    struct ParachainHeadProof {
        uint256 pos;
        uint256 width;
        bytes32[] proof;
    }

    struct OwnParachainHeadPartial {
        bytes32 parentHash;
        uint32 number;
        bytes32 stateRoot;
        bytes32 extrinsicsRoot;
    }

    struct BeefyMMRLeafPartial {
        uint8 version;
        uint32 parentNumber;
        bytes32 parentHash;
        uint64 nextAuthoritySetId;
        uint32 nextAuthoritySetLen;
        bytes32 nextAuthoritySetRoot;
    }

    function verifyCommitmentInParachain(
        bytes32 commitment,
        OwnParachainHeadPartial calldata _ownParachainHeadPartial,
        ParachainHeadProof calldata _parachainHeadProof,
        BeefyMMRLeafPartial calldata _beefyMMRLeafPartial,
        uint256 _beefyMMRLeafIndex,
        uint256 _beefyMMRLeafCount,
        bytes32[] calldata _beefyMMRLeafProof,
        BeefyLightClient beefyLightClient
    ) internal {
        // Must verify the parachain id to ensure msg comes from our parachain
        // TODO - maybe can be done at application level rather than here tho
        // - for example, application can register itself with channel to get
        // the parachain id as part of the calldata to it, then we prepend
        // it to the calldata

        // 2. Compute `ownParachainHead` by hashing the data of the `commitment` together with the contents of
        // `_ownParachainHeadPartial`
        bytes32 ownParachainHeadHash = createParachainHeadHash(
            _ownParachainHeadPartial,
            commitment
        );

        // 3. Compute `parachainHeadsRoot` by verifying the merkle proof using `ownParachainHeadHash` and
        // `_parachainHeadsProof`
        bytes32 parachainHeadsRoot = MerkleProof.computeRootFromProofAtPosition(
            ownParachainHeadHash,
            _parachainHeadProof.pos,
            _parachainHeadProof.width,
            _parachainHeadProof.proof
        );

        // 4. Compute the `beefyMMRLeaf` using `parachainHeadsRoot` and `_beefyMMRLeafPartial`
        bytes32 beefyMMRLeaf = createMMRLeafHash(
            _beefyMMRLeafPartial,
            parachainHeadsRoot
        );

        // 5. Verify inclusion of the beefy MMR leaf in the beefy MMR root using that `beefyMMRLeaf` as well as
        // `_beefyMMRLeafIndex`, `_beefyMMRLeafCount` and `_beefyMMRLeafProof`
        require(
            beefyLightClient.verifyBeefyMerkleLeaf(
                beefyMMRLeaf,
                _beefyMMRLeafIndex,
                _beefyMMRLeafCount,
                _beefyMMRLeafProof
            ),
            "Invalid proof"
        );
    }

    function createParachainHeadHash(
        ParachainLightClient.OwnParachainHeadPartial
            calldata _ownParachainHeadPartial,
        bytes32 commitment
    ) public pure returns (bytes32) {
        bytes memory scaleEncodedParachainHead = abi.encodePacked(
            _ownParachainHeadPartial.parentHash,
            ScaleCodec.encode32(_ownParachainHeadPartial.number),
            _ownParachainHeadPartial.stateRoot,
            _ownParachainHeadPartial.extrinsicsRoot,
            commitment
        );

        return keccak256(scaleEncodedParachainHead);
    }

    // To scale encode the byte array, we need to prefix it
    // with it's length. This is the expected current length of a leaf.
    bytes2 public constant MMR_LEAF_LENGTH_SCALE_ENCODED =
        bytes2(uint16(0xc501));

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

        return
            keccak256(
                bytes.concat(MMR_LEAF_LENGTH_SCALE_ENCODED, scaleEncodedMMRLeaf)
            );
    }
}
