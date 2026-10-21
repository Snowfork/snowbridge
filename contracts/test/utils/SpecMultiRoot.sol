// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.34;

/// @dev A second, independent model of `computeMultiRoot`, after the multiproof in the Ethereum
/// consensus specs (ssz/merkle-proofs.md: `get_helper_indices`, `calculate_multi_merkle_root`),
/// with Substrate's promotion rule. It shares no structure with the fold in
/// `SubstrateMerkleProof` and `ReferenceMultiRoot`:
///  - it derives the helper (sibling) set from `positions` and `width` alone, and requires
///    `siblings.length` to equal its size before it hashes anything;
///  - it computes each parent from its two children, looked up by (layer, index);
///  - a node is promoted when its parent's right child falls outside the layer, read from the
///    layer widths rather than from a `position + 1 == width` test.
/// Helpers are ordered layer by layer from the leaves up, then by ascending index. The spec
/// orders them by descending generalized index; ascending within a layer is this format's choice.
/// It also rejects what the spec leaves to the caller: no leaves, duplicate or unsorted
/// positions, and positions outside the tree.
/// Arithmetic is checked, so an overflow panics instead of wrapping.
library SpecMultiRoot {
    function computeMultiRoot(
        uint256[] memory positions,
        bytes32[] memory leaves,
        uint256 width,
        bytes32[] memory siblings
    ) internal pure returns (bool valid, bytes32 root) {
        uint256 n = positions.length;
        if (n == 0 || leaves.length != n) {
            return (false, bytes32(0));
        }
        for (uint256 i = 0; i < n; i++) {
            if (positions[i] >= width || (i > 0 && positions[i] <= positions[i - 1])) {
                return (false, bytes32(0));
            }
        }

        uint256[] memory widths = layerWidths(width);
        uint256[][] memory path = pathNodes(positions, widths.length - 1);
        uint256[][] memory helpers = helperNodes(path, widths);
        bytes32[][] memory helperValues;
        (valid, helperValues) = assignSiblings(helpers, siblings);
        if (!valid) {
            return (false, bytes32(0));
        }
        return (true, fold(leaves, widths, path, helpers, helperValues));
    }

    /// Path nodes per layer: the leaves' ancestors, ascending and distinct.
    function pathNodes(uint256[] memory positions, uint256 depth)
        internal
        pure
        returns (uint256[][] memory path)
    {
        path = new uint256[][](depth + 1);
        path[0] = positions;
        for (uint256 l = 0; l < depth; l++) {
            path[l + 1] = parents(path[l]);
        }
    }

    /// Helpers per layer: each path node's sibling inside the layer that is not itself on the
    /// path.
    function helperNodes(uint256[][] memory path, uint256[] memory widths)
        internal
        pure
        returns (uint256[][] memory helpers)
    {
        helpers = new uint256[][](widths.length - 1);
        for (uint256 l = 0; l < helpers.length; l++) {
            helpers[l] = siblingsOffPath(path[l], widths[l]);
        }
    }

    /// `siblings` must hold exactly one value per helper, in helper order.
    function assignSiblings(uint256[][] memory helpers, bytes32[] memory siblings)
        internal
        pure
        returns (bool valid, bytes32[][] memory helperValues)
    {
        uint256 count;
        for (uint256 l = 0; l < helpers.length; l++) {
            count += helpers[l].length;
        }
        if (siblings.length != count) {
            return (false, helperValues);
        }
        helperValues = new bytes32[][](helpers.length);
        uint256 next;
        for (uint256 l = 0; l < helpers.length; l++) {
            helperValues[l] = new bytes32[](helpers[l].length);
            for (uint256 k = 0; k < helpers[l].length; k++) {
                helperValues[l][k] = siblings[next++];
            }
        }
        return (true, helperValues);
    }

    /// Each parent from its two children; a parent whose right child falls outside the layer
    /// takes its left child unchanged.
    function fold(
        bytes32[] memory leaves,
        uint256[] memory widths,
        uint256[][] memory path,
        uint256[][] memory helpers,
        bytes32[][] memory helperValues
    ) internal pure returns (bytes32) {
        bytes32[] memory values = leaves;
        for (uint256 l = 0; l + 1 < widths.length; l++) {
            uint256[] memory up = path[l + 1];
            bytes32[] memory upValues = new bytes32[](up.length);
            for (uint256 k = 0; k < up.length; k++) {
                uint256 left = 2 * up[k];
                bytes32 leftValue = nodeAt(l, left, path, values, helpers, helperValues);
                if (left + 1 == widths[l]) {
                    upValues[k] = leftValue;
                } else {
                    upValues[k] = keccak256(
                        abi.encodePacked(
                            leftValue, nodeAt(l, left + 1, path, values, helpers, helperValues)
                        )
                    );
                }
            }
            values = upValues;
        }
        return values[0];
    }

    /// Layer widths from the leaves (`width`) up to the root (1): each is the ceiling of half
    /// the one below.
    function layerWidths(uint256 width) internal pure returns (uint256[] memory widths) {
        widths = new uint256[](257);
        uint256 d;
        widths[0] = width;
        while (widths[d] > 1) {
            widths[d + 1] = widths[d] / 2 + widths[d] % 2;
            d++;
        }
        assembly {
            mstore(widths, add(d, 1))
        }
    }

    function parents(uint256[] memory layer) internal pure returns (uint256[] memory up) {
        up = new uint256[](layer.length);
        uint256 m;
        for (uint256 i = 0; i < layer.length; i++) {
            uint256 parent = layer[i] / 2;
            if (m == 0 || up[m - 1] != parent) up[m++] = parent;
        }
        assembly {
            mstore(up, m)
        }
    }

    function siblingsOffPath(uint256[] memory layer, uint256 width)
        internal
        pure
        returns (uint256[] memory out)
    {
        out = new uint256[](layer.length);
        uint256 m;
        for (uint256 i = 0; i < layer.length; i++) {
            uint256 s = layer[i] % 2 == 0 ? layer[i] + 1 : layer[i] - 1;
            if (s < width && !contains(layer, s)) out[m++] = s;
        }
        assembly {
            mstore(out, m)
        }
    }

    function nodeAt(
        uint256 layer,
        uint256 index,
        uint256[][] memory path,
        bytes32[] memory values,
        uint256[][] memory helpers,
        bytes32[][] memory helperValues
    ) internal pure returns (bytes32) {
        (bool onPath, uint256 k) = find(path[layer], index);
        if (onPath) return values[k];
        bool isHelper;
        (isHelper, k) = find(helpers[layer], index);
        // Every child of a path node is on the path, a helper, or outside the layer.
        require(isHelper, "SpecMultiRoot: node is neither on the path nor a helper");
        return helperValues[layer][k];
    }

    function contains(uint256[] memory sorted, uint256 x) internal pure returns (bool found) {
        (found,) = find(sorted, x);
    }

    /// Binary search in an ascending array.
    function find(uint256[] memory sorted, uint256 x) internal pure returns (bool, uint256) {
        uint256 lo = 0;
        uint256 hi = sorted.length;
        while (lo < hi) {
            uint256 mid = lo + (hi - lo) / 2;
            if (sorted[mid] < x) lo = mid + 1;
            else hi = mid;
        }
        return (lo < sorted.length && sorted[lo] == x, lo);
    }
}
