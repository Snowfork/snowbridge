// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.34;

/// @dev Changes to an honest multiproof that must never reproduce the tree root.
library MultiproofAttacks {
    /// A second leaf at `positions[i]`, right after it.
    function duplicateLeaf(
        uint256[] memory positions,
        bytes32[] memory leaves,
        uint256 i,
        bytes32 leaf
    ) internal pure returns (uint256[] memory p, bytes32[] memory l) {
        uint256 n = positions.length;
        p = new uint256[](n + 1);
        l = new bytes32[](n + 1);
        for (uint256 k = 0; k <= n; k++) {
            uint256 from = k <= i ? k : k - 1;
            p[k] = positions[from];
            l[k] = k == i + 1 ? leaf : leaves[from];
        }
    }

    /// `pad` after every sibling, so a forged duplicate's climb finds a sibling at every layer.
    /// It defeats a verifier that stops once the first node reaches the root.
    function padSiblings(bytes32[] memory siblings, bytes32 pad)
        internal
        pure
        returns (bytes32[] memory s)
    {
        s = new bytes32[](2 * siblings.length);
        for (uint256 k = 0; k < siblings.length; k++) {
            s[2 * k] = siblings[k];
            s[2 * k + 1] = pad;
        }
    }

    /// Fisher-Yates shuffle driven by `seed`.
    function shuffle(bytes32[] memory siblings, uint256 seed)
        internal
        pure
        returns (bytes32[] memory s)
    {
        s = new bytes32[](siblings.length);
        for (uint256 k = 0; k < s.length; k++) {
            s[k] = siblings[k];
        }
        for (uint256 k = s.length; k > 1; k--) {
            uint256 j = uint256(keccak256(abi.encode(seed, k))) % k;
            (s[k - 1], s[j]) = (s[j], s[k - 1]);
        }
    }

    function sameOrder(bytes32[] memory a, bytes32[] memory b) internal pure returns (bool) {
        for (uint256 k = 0; k < a.length; k++) {
            if (a[k] != b[k]) return false;
        }
        return true;
    }

    /// Adds a leaf at `x`, which must not be in `positions`, keeping positions ascending.
    function insertLeaf(
        uint256[] memory positions,
        bytes32[] memory leaves,
        uint256 x,
        bytes32 leaf
    ) internal pure returns (uint256[] memory p, bytes32[] memory l) {
        uint256 n = positions.length;
        p = new uint256[](n + 1);
        l = new bytes32[](n + 1);
        uint256 k;
        for (uint256 j = 0; j <= n; j++) {
            if (k == j && (j == n || positions[j] > x)) {
                p[k] = x;
                l[k] = leaf;
                k++;
            }
            if (j < n) {
                p[k] = positions[j];
                l[k] = leaves[j];
                k++;
            }
        }
    }

    function removeLeaf(uint256[] memory positions, bytes32[] memory leaves, uint256 i)
        internal
        pure
        returns (uint256[] memory p, bytes32[] memory l)
    {
        uint256 n = positions.length;
        p = new uint256[](n - 1);
        l = new bytes32[](n - 1);
        for (uint256 k = 0; k + 1 < n; k++) {
            uint256 from = k < i ? k : k + 1;
            p[k] = positions[from];
            l[k] = leaves[from];
        }
    }

    /// The lowest position below `width` that is not in `positions`, searching from `start`.
    /// Returns `width` when every position is taken.
    function unsampled(uint256[] memory positions, uint256 width, uint256 start)
        internal
        pure
        returns (uint256)
    {
        start %= width;
        for (uint256 d = 0; d < width; d++) {
            uint256 x = (start + d) % width;
            bool taken;
            for (uint256 k = 0; k < positions.length; k++) {
                if (positions[k] == x) {
                    taken = true;
                    break;
                }
            }
            if (!taken) return x;
        }
        return width;
    }
}
