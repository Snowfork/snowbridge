// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

library MerkleProof {
    /**
     * @notice Verify that a specific leaf element is part of the Merkle Tree at a specific position in the tree
     *
     * @param root the root of the merkle tree
     * @param leaf the leaf which needs to be proven
     * @param pos the position of the leaf, index starting with 0
     * @param width the width or number of leaves in the tree
     * @param proof the array of proofs to help verify the leaf's membership, ordered from leaf to root
     * @return a boolean value representing the success or failure of the operation
     */
    function verifyMerkleLeafAtPosition(
        bytes32 root,
        bytes32 leaf,
        uint256 pos,
        uint256 width,
        bytes32[] calldata proof
    ) public pure returns (bool) {
        bytes32 computedHash = computeRootFromProofAtPosition(leaf, pos, width, proof);

        return computedHash == root;
    }

    /**
     * @notice Compute the root of a MMR from a leaf and proof
     *
     * @param leaf the leaf we want to prove
     * @param proof an array of nodes to be hashed in order that they should be hashed
     * @param side an array of booleans signalling whether the corresponding proof hash should be hashed on the left side (true) or
     * the right side (false) of the current node hash
     */
    function computeRootFromProofAndSide(
        bytes32 leaf,
        bytes32[] calldata proof,
        bool[] calldata side
    ) public pure returns (bytes32) {
        bytes32 node = leaf;
        for (uint256 i = 0; i < proof.length; i++) {
            if (side[i]) {
                node = keccak256(abi.encodePacked(proof[i], node));
            } else {
                node = keccak256(abi.encodePacked(node, proof[i]));
            }
        }
        return node;
    }

    function computeRootFromProofAtPosition(
        bytes32 leaf,
        uint256 pos,
        uint256 width,
        bytes32[] calldata proof
    ) public pure returns (bytes32) {
        bytes32 computedHash = leaf;

        require(pos < width, "Merkle position is too high");

        unchecked {
            uint256 i = 0;
            for (uint256 height = 0; width > 1; height++) {
                bool computedHashLeft = pos & 1 == 0;

                // check if at rightmost branch and whether the computedHash is left
                if (pos + 1 == width && computedHashLeft) {
                    // there is no sibling and also no element in proofs, so we just go up one layer in the tree
                    pos = pos >> 1;
                    width = ((width - 1) >> 1) + 1;
                    continue;
                }

                if (computedHashLeft) {
                    computedHash = _efficientHash(computedHash, proof[i]);
                } else {
                    computedHash = _efficientHash(proof[i], computedHash);
                }

                pos = pos >> 1;
                width = ((width - 1) >> 1) + 1;
                i++;
            }

            return computedHash;
        }
    }

    function _efficientHash(bytes32 a, bytes32 b) private pure returns (bytes32 value) {
        /// @solidity memory-safe-assembly
        assembly {
            mstore(0x00, a)
            mstore(0x20, b)
            value := keccak256(0x00, 0x40)
        }
    }
}
