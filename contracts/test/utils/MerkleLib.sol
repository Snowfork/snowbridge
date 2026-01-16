// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.33;

library MerkleLib {
    // Build a power-of-two Merkle tree with `leafCount` leaves, place `targetLeaf` at
    // `targetIndex`, return the root and the inclusion proof (items + order) for that target.
    function buildMerkleWithTargetLeaf(uint256 leafCount, uint256 targetIndex, bytes32 targetLeaf)
        internal
        pure
        returns (bytes32 root, bytes32[] memory items, uint256 order)
    {
        require(
            leafCount > 0 && (leafCount & (leafCount - 1)) == 0, "leafCount must be power of two"
        );
        require(targetIndex < leafCount, "targetIndex OOB");

        bytes32[] memory leaves = new bytes32[](leafCount);
        for (uint256 i = 0; i < leafCount; i++) {
            if (i == targetIndex) {
                leaves[i] = targetLeaf;
            } else {
                leaves[i] = keccak256(abi.encodePacked("leaf:", i));
            }
        }

        // compute root by reducing levels
        bytes32[] memory current = leaves;
        while (current.length > 1) {
            uint256 pairs = current.length / 2;
            bytes32[] memory next = new bytes32[](pairs);
            for (uint256 j = 0; j < pairs; j++) {
                next[j] = keccak256(abi.encodePacked(current[2 * j], current[2 * j + 1]));
            }
            current = next;
        }
        root = current[0];

        // build proof for targetIndex
        // compute depth
        uint256 depth = 0;
        uint256 tmp = leafCount;
        while (tmp > 1) {
            tmp = tmp / 2;
            depth++;
        }

        bytes32[] memory proof = new bytes32[](depth);
        uint256 orderBits = 0;
        bytes32[] memory level = leaves;
        uint256 index = targetIndex;
        uint256 proofLen = 0;
        while (level.length > 1) {
            uint256 sibling = index ^ 1;
            proof[proofLen] = level[sibling];
            if ((index & 1) == 1) {
                orderBits |= (uint256(1) << proofLen);
            }
            proofLen++;

            // build next level
            uint256 pairs = level.length / 2;
            bytes32[] memory next = new bytes32[](pairs);
            for (uint256 j = 0; j < pairs; j++) {
                next[j] = keccak256(abi.encodePacked(level[2 * j], level[2 * j + 1]));
            }
            index = index / 2;
            level = next;
        }

        // trim proof
        items = new bytes32[](proofLen);
        for (uint256 k = 0; k < proofLen; k++) {
            items[k] = proof[k];
        }
        order = orderBits;
    }
}

library MerkleLibSubstrate {
    function hashPair(bytes32 a, bytes32 b) internal pure returns (bytes32 value) {
        assembly {
            mstore(0x00, a)
            mstore(0x20, b)
            value := keccak256(0x00, 0x40)
        }
    }

    function nextLevel(bytes32[] memory level) internal pure returns (bytes32[] memory) {
        uint256 n = level.length;
        uint256 pairs = (n + 1) / 2; // ceil(n/2)
        bytes32[] memory next = new bytes32[](pairs);
        for (uint256 j = 0; j < pairs; j++) {
            bytes32 left = level[2 * j];
            bytes32 right = (2 * j + 1 < n) ? level[2 * j + 1] : left;
            next[j] = hashPair(left, right);
        }
        return next;
    }

    function depth(uint256 width) internal pure returns (uint256 d) {
        d = 0;
        while (width > 1) {
            width = (width + 1) / 2;
            d++;
        }
    }

    function genLeaves(uint256 width) internal pure returns (bytes32[] memory leaves) {
        leaves = new bytes32[](width);
        for (uint256 i = 0; i < width; i++) {
            leaves[i] = keccak256(abi.encodePacked("leaf:", i));
        }
    }

    function genProof(bytes32[] memory leaves, uint256 index)
        internal
        pure
        returns (bytes32[] memory proof)
    {
        uint256 width = leaves.length;
        uint256 d = depth(width);
        proof = new bytes32[](d);

        bytes32[] memory level = leaves;
        uint256 pos = index;
        uint256 k = 0;
        while (level.length > 1) {
            uint256 L = level.length;
            uint256 siblingIndex = (pos ^ 1) < L ? (pos ^ 1) : pos;
            proof[k++] = level[siblingIndex];

            level = nextLevel(level);
            pos = pos / 2;
        }

        return proof;
    }

    function rootFromLeaves(bytes32[] memory leaves) internal pure returns (bytes32) {
        bytes32[] memory level = leaves;
        while (level.length > 1) {
            level = nextLevel(level);
        }
        return level[0];
    }

    function buildBinaryMerkleTree(bytes32[] memory leaves)
        internal
        pure
        returns (bytes32 root, bytes32[][] memory outProofs)
    {
        uint256 n = leaves.length;
        require(n > 0, "no leaves");

        // number of levels (excluding leaf level)
        uint256 levels = 0;
        for (uint256 w = n; w > 1; w = (w + 1) >> 1) {
            levels++;
        }

        outProofs = new bytes32[][](n);
        for (uint256 i = 0; i < n; i++) {
            outProofs[i] = new bytes32[](levels);
        }

        // for each leaf independently, compute its proof by walking up levels
        for (uint256 leafIdx = 0; leafIdx < n; leafIdx++) {
            uint256 pos = leafIdx;
            uint256 width = n;
            bytes32[] memory layer = new bytes32[](width);
            for (uint256 i = 0; i < width; i++) {
                layer[i] = leaves[i];
            }

            uint256 step = 0;
            while (width > 1) {
                // proof sibling at this level
                bytes32 sibling;
                if (pos & 1 == 1) {
                    // right child -> sibling is left (pos-1)
                    sibling = layer[pos - 1];
                } else if (pos + 1 == width) {
                    // last element with no right sibling -> duplicate self
                    sibling = layer[pos];
                } else {
                    // left child with right sibling
                    sibling = layer[pos + 1];
                }
                outProofs[leafIdx][step] = sibling;

                // next layer with duplication of last when odd
                uint256 nextW = (width + 1) >> 1;
                bytes32[] memory nextLayer = new bytes32[](nextW);
                for (uint256 i = 0; i < width; i += 2) {
                    bytes32 left = layer[i];
                    bytes32 right = (i + 1 < width) ? layer[i + 1] : layer[i];
                    nextLayer[i >> 1] = keccak256(abi.encodePacked(left, right));
                }

                // move up one level
                pos >>= 1;
                width = nextW;
                layer = nextLayer;
                step++;
            }

            if (leafIdx == 0) {
                root = layer[0];
            }
        }
    }
}
