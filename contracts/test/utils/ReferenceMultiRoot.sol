// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.34;

import {SubstrateMerkleProof} from "../../src/utils/SubstrateMerkleProof.sol";

/// @dev The Solidity `computeMultiRoot` that the assembly in `SubstrateMerkleProof` replaced.
library ReferenceMultiRoot {
    function computeMultiRoot(
        uint256[] memory positions,
        bytes32[] memory leaves,
        uint256 width,
        bytes32[] calldata siblings
    ) internal pure returns (bool valid, bytes32 root) {
        uint256 n = positions.length;
        if (n == 0 || n != leaves.length) {
            return (false, bytes32(0));
        }
        for (uint256 i = 0; i < n; i++) {
            if (positions[i] >= width || (i > 0 && positions[i] <= positions[i - 1])) {
                return (false, bytes32(0));
            }
        }

        uint256 next;
        unchecked {
            while (width > 1) {
                uint256 m;
                for (uint256 i = 0; i < n; i++) {
                    uint256 position = positions[i];
                    bytes32 node = leaves[i];
                    if (position + 1 == width && width & 1 == 1) {
                        // Lone trailing node of an odd-width layer: promoted unchanged.
                    } else if (position & 1 == 0 && i + 1 < n && positions[i + 1] == position + 1)
                    {
                        node = SubstrateMerkleProof.efficientHash(node, leaves[i + 1]);
                        i++;
                    } else {
                        if (next >= siblings.length) {
                            return (false, bytes32(0));
                        }
                        if (position & 1 == 1) {
                            node = SubstrateMerkleProof.efficientHash(siblings[next], node);
                        } else {
                            node = SubstrateMerkleProof.efficientHash(node, siblings[next]);
                        }
                        next++;
                    }
                    positions[m] = position >> 1;
                    leaves[m] = node;
                    m++;
                }
                n = m;
                width = ((width - 1) >> 1) + 1;
            }
        }
        if (n != 1 || next != siblings.length) {
            return (false, bytes32(0));
        }
        return (true, leaves[0]);
    }
}
