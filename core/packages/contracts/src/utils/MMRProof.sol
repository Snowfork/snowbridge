// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.20;

library MMRProof {
    /**
     * @dev Verify inclusion of a leaf in an MMR
     * @param root MMR root hash
     * @param leafHash leaf hash
     * @param proof an array of hashes
     * @param proofOrder a bitfield describing the order of each item (left vs right)
     */
    function verifyLeafProof(bytes32 root, bytes32 leafHash, bytes32[] calldata proof, uint256 proofOrder)
        internal
        pure
        returns (bool)
    {
        bytes32 acc = leafHash;
        for (uint256 i = 0; i < proof.length;) {
            acc = hashPairs(acc, proof[i], (proofOrder >> i) & 1);
            unchecked {
                i++;
            }
        }
        return root == acc;
    }

    function hashPairs(bytes32 x, bytes32 y, uint256 order) internal pure returns (bytes32 value) {
        assembly {
            switch order
            case 0 {
                mstore(0x00, x)
                mstore(0x20, y)
            }
            default {
                mstore(0x00, y)
                mstore(0x20, x)
            }
            value := keccak256(0x0, 0x40)
        }
    }
}
