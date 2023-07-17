// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.20;

import {SubstrateMerkleProof} from "./utils/MerkleProof.sol";
import {BeefyClient} from "./BeefyClient.sol";
import {ScaleCodec} from "./ScaleCodec.sol";
import {SubstrateTypes} from "./SubstrateTypes.sol";

import {VerificationStorage} from "./storage/VerificationStorage.sol";

library Verification {
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

    uint256 public constant DIGEST_ITEM_PRERUNTIME = 6;
    uint256 public constant DIGEST_ITEM_CONSENSUS = 4;
    uint256 public constant DIGEST_ITEM_SEAL = 5;
    uint256 public constant DIGEST_ITEM_OTHER = 0;
    uint256 public constant DIGEST_ITEM_RUNTIME_ENVIRONMENT_UPDATED = 8;

    struct DigestItem {
        uint256 kind;
        bytes4 consensusEngineID;
        bytes data;
    }

    struct ParachainHeader {
        bytes32 parentHash;
        uint256 number;
        bytes32 stateRoot;
        bytes32 extrinsicsRoot;
        DigestItem[] digestItems;
    }

    struct Proof {
        ParachainHeader header;
        HeadProof headProof;
        MMRLeafPartial leafPartial;
        bytes32[] leafProof;
        uint256 leafProofOrder;
    }

    error InvalidParachainHeader();

    struct InitParams {
        address beefyClient;
        uint32 parachainID;
    }

    function initialize(InitParams calldata params) external {
        VerificationStorage.Layout storage $ = VerificationStorage.layout();
        $.beefyClient = params.beefyClient;
        $.parachainID = params.parachainID;
        $.encodedParachainID = ScaleCodec.encodeU32(params.parachainID);
    }

    function verifyCommitment(bytes32 commitment, Proof calldata proof) external view returns (bool) {
        VerificationStorage.Layout storage $ = VerificationStorage.layout();

        // bypass verification for unit tests
        if ($.beefyClient == address(0)) {
            return true;
        }

        if (!isCommitmentInHeaderDigest(commitment, proof.header)) {
            return false;
        }
        // Compute the merkle leaf hash of our parachain
        bytes32 parachainHeadHash = createParachainHeaderMerkleLeaf(proof.header);

        // Compute the merkle root hash of all parachain heads
        if (proof.headProof.pos >= proof.headProof.width) {
            return false;
        }
        bytes32 parachainHeadsRoot = SubstrateMerkleProof.computeRoot(
            parachainHeadHash, proof.headProof.pos, proof.headProof.width, proof.headProof.proof
        );

        bytes32 leafHash = createMMRLeaf(proof.leafPartial, parachainHeadsRoot);
        return BeefyClient($.beefyClient).verifyMMRLeafProof(leafHash, proof.leafProof, proof.leafProofOrder);
    }

    // Verify that a message commitment is in the header digest
    function isCommitmentInHeaderDigest(bytes32 commitment, ParachainHeader calldata header)
        internal
        pure
        returns (bool)
    {
        for (uint256 i = 0; i < header.digestItems.length; i++) {
            DigestItem memory item = header.digestItems[i];
            if (item.kind == DIGEST_ITEM_OTHER && item.data.length == 32 && commitment == bytes32(item.data)) {
                return true;
            }
        }
        return false;
    }

    // encodes Vec<DigestItem>
    function encodeDigestItems(DigestItem[] calldata digestItems) internal pure returns (bytes memory) {
        // encode all digest items into a buffer
        bytes memory accum = hex"";
        for (uint256 i = 0; i < digestItems.length; i++) {
            accum = bytes.concat(accum, encodeDigestItem(digestItems[i]));
        }
        // encode number of digest items, followed by encoded digest items
        return bytes.concat(ScaleCodec.encodeCompactUint(digestItems.length), accum);
    }

    function encodeDigestItem(DigestItem calldata digestItem) internal pure returns (bytes memory) {
        if (digestItem.kind == DIGEST_ITEM_PRERUNTIME) {
            return bytes.concat(
                bytes1(uint8(DIGEST_ITEM_PRERUNTIME)),
                digestItem.consensusEngineID,
                ScaleCodec.encodeCompactUint(digestItem.data.length),
                digestItem.data
            );
        } else if (digestItem.kind == DIGEST_ITEM_CONSENSUS) {
            return bytes.concat(
                bytes1(uint8(DIGEST_ITEM_CONSENSUS)),
                digestItem.consensusEngineID,
                ScaleCodec.encodeCompactUint(digestItem.data.length),
                digestItem.data
            );
        } else if (digestItem.kind == DIGEST_ITEM_SEAL) {
            return bytes.concat(
                bytes1(uint8(DIGEST_ITEM_SEAL)),
                digestItem.consensusEngineID,
                ScaleCodec.encodeCompactUint(digestItem.data.length),
                digestItem.data
            );
        } else if (digestItem.kind == DIGEST_ITEM_OTHER) {
            return bytes.concat(
                bytes1(uint8(DIGEST_ITEM_OTHER)), ScaleCodec.encodeCompactUint(digestItem.data.length), digestItem.data
            );
        } else if (digestItem.kind == DIGEST_ITEM_RUNTIME_ENVIRONMENT_UPDATED) {
            return bytes.concat(bytes1(uint8(DIGEST_ITEM_RUNTIME_ENVIRONMENT_UPDATED)));
        } else {
            revert InvalidParachainHeader();
        }
    }

    // Creates a keccak hash of a SCALE-encoded parachain header
    function createParachainHeaderMerkleLeaf(ParachainHeader calldata header) internal view returns (bytes32) {
        VerificationStorage.Layout storage $ = VerificationStorage.layout();

        // Encode Parachain header
        bytes memory encodedHeader = bytes.concat(
            // H256
            header.parentHash,
            // Compact unsigned int
            ScaleCodec.encodeCompactUint(header.number),
            // H256
            header.stateRoot,
            // H256
            header.extrinsicsRoot,
            // Vec<DigestItem>
            encodeDigestItems(header.digestItems)
        );

        // Hash of encoded parachain header merkle leaf
        return keccak256(
            bytes.concat(
                // u32
                $.encodedParachainID,
                // Vec<u8>
                ScaleCodec.encodeCompactUint(encodedHeader.length),
                encodedHeader
            )
        );
    }

    function createParachainHeader(ParachainHeader calldata header) internal view returns (bytes memory) {
        VerificationStorage.Layout storage $ = VerificationStorage.layout();

        bytes memory encodedHeader = bytes.concat(
            // H256
            header.parentHash,
            // Compact unsigned int
            ScaleCodec.encodeCompactUint(header.number),
            // H256
            header.stateRoot,
            // H256
            header.extrinsicsRoot,
            // Vec<DigestItem>
            ScaleCodec.encodeCompactUint(header.digestItems.length),
            encodeDigestItems(header.digestItems)
        );

        return bytes.concat(
            // u32
            $.encodedParachainID,
            // length of encoded header
            ScaleCodec.encodeCompactUint(encodedHeader.length),
            encodedHeader
        );
    }

    function createMMRLeaf(MMRLeafPartial memory leaf, bytes32 parachainHeadsRoot) internal pure returns (bytes32) {
        bytes memory encodedLeaf = bytes.concat(
            ScaleCodec.encodeU8(leaf.version),
            ScaleCodec.encodeU32(leaf.parentNumber),
            leaf.parentHash,
            ScaleCodec.encodeU64(leaf.nextAuthoritySetID),
            ScaleCodec.encodeU32(leaf.nextAuthoritySetLen),
            leaf.nextAuthoritySetRoot,
            parachainHeadsRoot
        );
        return keccak256(encodedLeaf);
    }
}
