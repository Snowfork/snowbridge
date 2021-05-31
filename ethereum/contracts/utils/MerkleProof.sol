// "SPDX-License-Identifier: Apache-2.0"
pragma solidity ^0.7.0;

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
        bytes32 computedHash = leaf;

        if (pos + 1 > width) {
            return false;
        }

        uint256 i = 0;
        for (uint256 height = 0; width > 1; height++) {
            bool computedHashLeft = pos % 2 == 0;

            // check if at rightmost branch and whether the computedHash is left
            if (pos + 1 == width && computedHashLeft) {
                // there is no sibling and also no element in proofs, so we just go up one layer in the tree
                pos /= 2;
                width = ((width - 1) / 2) + 1;
                continue;
            }

            if (i >= proof.length) {
                // need another element from the proof we don't have
                return false;
            }

            bytes32 proofElement = proof[i];

            if (computedHashLeft) {
                computedHash = keccak256(
                    abi.encodePacked(computedHash, proofElement)
                );
            } else {
                computedHash = keccak256(
                    abi.encodePacked(proofElement, computedHash)
                );
            }

            pos /= 2;
            width = ((width - 1) / 2) + 1;
            i++;
        }

        return computedHash == root;
    }
}
