// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.28;

import {BeefyClient} from "./BeefyClient.sol";
import {ScaleCodec} from "./utils/ScaleCodec.sol";

library BeefyVerification {
    /// @dev An MMRLeaf without the `leaf_extra` field.
    /// Reference: https://github.com/paritytech/substrate/blob/14e0a0b628f9154c5a2c870062c3aac7df8983ed/primitives/consensus/beefy/src/mmr.rs#L52
    struct MMRLeafPartial {
        uint8 version;
        uint32 parentNumber;
        bytes32 parentHash;
        uint64 nextAuthoritySetID;
        uint32 nextAuthoritySetLen;
        bytes32 nextAuthoritySetRoot;
    }

    /// @dev A proof that a given MMR leaf is part of the MMR maintained by the BEEFY light client
    struct Proof {
        // The MMR leaf to be proven
        MMRLeafPartial leafPartial;
        // The MMR leaf proof
        bytes32[] leafProof;
        // The order in which proof items should be combined
        uint256 leafProofOrder;
    }

    /// @dev Verify that a given MMR leaf is part of the MMR maintained by the BEEFY light client
    ///
    /// @param beefyClient The address of the BEEFY light client
    /// @param leafExtra The extra data to be included in the MMR leaf
    /// @param proof The MMR leaf proof containing the leaf data and proof elements
    function verifyBeefyMMRLeaf(address beefyClient, bytes32 leafExtra, Proof calldata proof)
        external
        view
        returns (bool)
    {
        // Create the MMR leaf by adding the leafExtra field and SCALE-encoding the full leaf
        bytes32 leafHash = createMMRLeaf(proof.leafPartial, leafExtra);

        // Verify that the MMR leaf is part of the MMR maintained by the BEEFY light client
        return BeefyClient(beefyClient).verifyMMRLeafProof(leafHash, proof.leafProof, proof.leafProofOrder);
    }

    // SCALE-encode: MMRLeaf
    // Reference: https://github.com/paritytech/substrate/blob/14e0a0b628f9154c5a2c870062c3aac7df8983ed/primitives/consensus/beefy/src/mmr.rs#L52
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
