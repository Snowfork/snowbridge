// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

library MMRProofVerification {
    /**
     * @dev Verify inclusion of a leaf in an MMR
     * @param proof an array of hashes
     * @param proofOrder a bitfield describing the order of each item (left vs right)
     */
    function verifyLeafProof(
        bytes32 root,
        bytes32 leafHash,
        bytes32[] calldata proof,
        uint256 proofOrder
    ) external pure returns (bool) {
        bytes32 computedHash = leafHash;
        unchecked {
            for (uint256 i = 0; i < proof.length; i++) {
                computedHash = hashPairs(computedHash, proof[i], (proofOrder >> i) & 1);
            }
        }
        return root == computedHash;
    }

    function hashPairs(bytes32 left, bytes32 right, uint256 order) internal pure returns (bytes32 _hash) {
        assembly {
            switch order
            case 0 {
                mstore(0x00, left)
                mstore(0x20, right)
            }
            default {
                mstore(0x00, right)
                mstore(0x20, left)
            }
            _hash := keccak256(0x0, 0x40)
        }
    }
}
