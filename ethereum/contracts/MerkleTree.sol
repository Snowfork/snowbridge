// SPDX-License-Identifier: Apache-2.0
pragma solidity >=0.7.6;

/**
 * @title MerkleTree
 * @notice This contract represents a Merkle tree data structure
 */
contract MerkleTree {
    bytes32 public root;

    constructor(bytes32 _root) {
        root = _root;
    }

    /**
     * @notice Verify a single leaf element in a Merkle Tree
     * @param leaf the leaf which needs to be proved
     * @param proof the array of proofs to help verify the leafs membership
     * @return a boolean value representing the success or failure of the operation
     */
    function verify(bytes32 leaf, bytes32[] memory proof)
        public
        view
        returns (bool)
    {
        bytes32 computedHash = leaf;

        for (uint256 i = 0; i < proof.length; i++) {
            bytes32 proofElement = proof[i];

            if (computedHash < proofElement) {
                // Hash(current computed hash + current element of the proof)
                computedHash = keccak256(
                    abi.encodePacked(computedHash, proofElement)
                );
            } else {
                // Hash(current element of the proof + current computed hash)
                computedHash = keccak256(
                    abi.encodePacked(proofElement, computedHash)
                );
            }
        }

        // Check if the computed hash (root) is equal to the provided root
        return computedHash == root;
    }

    /**
     * @notice Verify that a specific leaf element is part of the Merkle Tree at a specific position in the tree
     * @return a boolean value representing the success or failure of the operation
     */
    // function verifyAtPosition() {
    //     // TODO
    // }
}
