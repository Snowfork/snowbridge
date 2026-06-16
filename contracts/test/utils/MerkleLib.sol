// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.34;

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
        uint256 width = (n + 1) / 2; // ceil(n/2)
        bytes32[] memory next = new bytes32[](width);
        for (uint256 j = 0; j < n / 2; j++) {
            next[j] = hashPair(level[2 * j], level[2 * j + 1]);
        }
        // Lone trailing node in an odd-width level is PROMOTED unchanged (matches substrate's
        // binary-merkle-tree and the Snowbridge relayer). Do NOT hash it with itself.
        if (n % 2 == 1) {
            next[width - 1] = level[n - 1];
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
        // First pass: count the canonical proof length, skipping promoted (lone trailing) nodes.
        uint256 k = 0;
        {
            uint256 pos = index;
            bytes32[] memory level = leaves;
            while (level.length > 1) {
                uint256 L = level.length;
                if (!(pos == L - 1 && L % 2 == 1)) {
                    k++;
                }
                level = nextLevel(level);
                pos = pos / 2;
            }
        }

        // Second pass: collect siblings, skipping promoted levels (which carry no sibling).
        proof = new bytes32[](k);
        uint256 j = 0;
        bytes32[] memory lvl = leaves;
        uint256 p = index;
        while (lvl.length > 1) {
            uint256 L = lvl.length;
            if (p == L - 1 && L % 2 == 1) {
                // promoted: no sibling at this level
            } else if (p % 2 == 1) {
                proof[j++] = lvl[p - 1];
            } else {
                proof[j++] = lvl[p + 1];
            }
            lvl = nextLevel(lvl);
            p = p / 2;
        }
    }

    function rootFromLeaves(bytes32[] memory leaves) internal pure returns (bytes32) {
        bytes32[] memory level = leaves;
        while (level.length > 1) {
            level = nextLevel(level);
        }
        return level[0];
    }

    /// Build every layer of the tree once (leaf layer at index 0, root layer last), returning the
    /// layers and the root. Pair with `proofFromLevels` to extract many proofs without rebuilding
    /// the tree per leaf (cheaper than calling `genProof` repeatedly).
    function buildLevels(bytes32[] memory leaves)
        internal
        pure
        returns (bytes32[][] memory L, bytes32 root)
    {
        uint256 nl = 1;
        {
            uint256 w = leaves.length;
            while (w > 1) {
                w = (w + 1) / 2;
                nl++;
            }
        }
        L = new bytes32[][](nl);
        L[0] = leaves;
        for (uint256 i = 1; i < nl; i++) {
            L[i] = nextLevel(L[i - 1]);
        }
        root = L[nl - 1][0];
    }

    /// Canonical proof for position `p` from precomputed layers (see `buildLevels`). Skips promoted
    /// (lone trailing) nodes, which carry no sibling.
    function proofFromLevels(bytes32[][] memory L, uint256 p)
        internal
        pure
        returns (bytes32[] memory pr)
    {
        uint256 c;
        {
            uint256 q = p;
            for (uint256 i = 0; i < L.length - 1; i++) {
                uint256 w = L[i].length;
                if (!(q == w - 1 && w % 2 == 1)) c++;
                q /= 2;
            }
        }
        pr = new bytes32[](c);
        uint256 k;
        uint256 pos = p;
        for (uint256 i = 0; i < L.length - 1; i++) {
            uint256 w = L[i].length;
            if (pos == w - 1 && w % 2 == 1) {
                // promoted: no sibling at this level
            } else if (pos & 1 == 1) {
                pr[k++] = L[i][pos - 1];
            } else {
                pr[k++] = L[i][pos + 1];
            }
            pos /= 2;
        }
    }

    /// Whether position `p`'s length-`kp` proof verifies at index `X` in a width-`n` tree, i.e. the
    /// first `kp` per-level direction bits match. Mirrors `SubstrateMerkleProof.computeRoot`'s
    /// branch logic; pre-fix the library accepted these aliases, post-fix it does not.
    function aliases(uint256 p, uint256 X, uint256 kp, uint256 n) internal pure returns (bool) {
        uint256 pp = p;
        uint256 xx = X;
        uint256 w = n;
        for (uint256 i = 0; i < kp; i++) {
            bool dp = (pp & 1 == 1) || (pp + 1 == w);
            bool dx = (xx & 1 == 1) || (xx + 1 == w);
            if (dp != dx) return false;
            pp >>= 1;
            xx >>= 1;
            w = ((w - 1) >> 1) + 1;
        }
        return true;
    }

    function buildBinaryMerkleTree(bytes32[] memory leaves)
        internal
        pure
        returns (bytes32 root, bytes32[][] memory outProofs)
    {
        uint256 n = leaves.length;
        require(n > 0, "no leaves");

        outProofs = new bytes32[][](n);

        // for each leaf independently, compute its proof by walking up levels
        for (uint256 leafIdx = 0; leafIdx < n; leafIdx++) {
            // First, count this leaf's canonical proof length (promoted levels carry no sibling).
            uint256 proofLen = 0;
            {
                uint256 p = leafIdx;
                uint256 w = n;
                while (w > 1) {
                    if (!(p + 1 == w && w & 1 == 1)) {
                        proofLen++;
                    }
                    p >>= 1;
                    w = (w + 1) >> 1;
                }
            }
            outProofs[leafIdx] = new bytes32[](proofLen);

            uint256 pos = leafIdx;
            uint256 width = n;
            bytes32[] memory layer = new bytes32[](width);
            for (uint256 i = 0; i < width; i++) {
                layer[i] = leaves[i];
            }

            uint256 step = 0;
            while (width > 1) {
                if (pos + 1 == width && width & 1 == 1) {
                    // Lone trailing node: promoted unchanged, no sibling recorded.
                } else if (pos & 1 == 1) {
                    // right child -> sibling is left (pos-1)
                    outProofs[leafIdx][step++] = layer[pos - 1];
                } else {
                    // left child with right sibling
                    outProofs[leafIdx][step++] = layer[pos + 1];
                }

                // next layer: PROMOTE the lone trailing node when odd (do not duplicate-hash it).
                uint256 nextW = (width + 1) >> 1;
                bytes32[] memory nextLayer = new bytes32[](nextW);
                for (uint256 i = 0; i + 1 < width; i += 2) {
                    nextLayer[i >> 1] = keccak256(abi.encodePacked(layer[i], layer[i + 1]));
                }
                if (width & 1 == 1) {
                    nextLayer[nextW - 1] = layer[width - 1];
                }

                // move up one level
                pos >>= 1;
                width = nextW;
                layer = nextLayer;
            }

            if (leafIdx == 0) {
                root = layer[0];
            }
        }
    }
}
