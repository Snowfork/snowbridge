// "SPDX-License-Identifier: MIT"
pragma solidity ^0.8.5;

library SparseMerkleMultiProof {

    function hash_node(bytes32 left, bytes32 right)
        internal
        pure
        returns (bytes32 hash)
    {
        assembly {
            mstore(0x00, left)
            mstore(0x20, right)
            hash := keccak256(0x00, 0x40)
        }
        return hash;
    }

    // Indices are required to be sorted highest to lowest.
    function verify(
        bytes32 root,
        uint256 depth,
        uint256[] memory indices,
        bytes32[] memory leaves,
        bytes32[] memory decommitments
    )
        internal
        pure
        returns (bool)
    {
        require(indices.length == leaves.length, "LENGTH_MISMATCH");
        uint256 n = indices.length;

        // Dynamically allocate index and hash queue
        uint256[] memory tree_indices = new uint256[](n + 1);
        bytes32[] memory hashes = new bytes32[](n + 1);
        uint256 head = 0;
        uint256 tail = 0;
        uint256 di = 0;

        // Queue the leafs
        for(; tail < n; ++tail) {
            tree_indices[tail] = 2**depth + indices[tail];
            hashes[tail] = leaves[tail];
        }

        // Itterate the queue until we hit the root
        while (true) {
            uint256 index = tree_indices[head];
            bytes32 hash = hashes[head];
            head = (head + 1) % (n + 1);

            // Merkle root
            if (index == 1) {
                return hash == root;
            // Even node, take sibbling from decommitments
            } else if (index & 1 == 0) {
                hash = hash_node(hash, decommitments[di++]);
            // Odd node with sibbling in the queue
            } else if (head != tail && tree_indices[head] == index - 1) {
                hash = hash_node(hashes[head], hash);
                head = (head + 1) % (n + 1);
            // Odd node with sibbling from decommitments
            } else {
                hash = hash_node(decommitments[di++], hash);
            }
            tree_indices[tail] = index / 2;
            hashes[tail] = hash;
            tail = (tail + 1) % (n + 1);
        }

        // Fix warning
        return false;
    }
}
