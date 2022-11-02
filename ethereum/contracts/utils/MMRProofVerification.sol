// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

/**
 * @dev The MMRProof is used to verify inclusion of a leaf in an MMR
 * @param items an array of hashes
 * @param order a bitfield describing the order of each item (left vs right)
*/
struct MMRProof {
    bytes32[] items;
    uint64 order;
}

library MMRProofVerification {
    function verifyLeafProof(
        bytes32 root,
        bytes32 leafHash,
        MMRProof calldata proof
    ) external pure returns (bool) {
        require(proof.items.length < 64, "proof to large");
        return root == calculateMerkleRoot(leafHash, proof.items, proof.order);
    }

    // Get the value of the bit at the given 'index' in 'self'.
    // index should be validated beforehand to make sure it is less than 64
    function bit(uint64 self, uint256 index) internal pure returns (bool) {
        if (uint8((self >> index) & 1) == 1) {
            return true;
        } else {
            return false;
        }
    }

    function calculateMerkleRoot(
        bytes32 leafHash,
        bytes32[] calldata items,
        uint64 order
    ) internal pure returns (bytes32) {
        bytes32 currentHash = leafHash;

        for (uint256 currentPosition = 0; currentPosition < items.length; currentPosition++) {
            bool isSiblingLeft = bit(order, currentPosition);
            bytes32 sibling = items[currentPosition];

            if (isSiblingLeft) {
                currentHash = keccak256(bytes.concat(sibling, currentHash));
            } else {
                currentHash = keccak256(bytes.concat(currentHash, sibling));
            }
        }
        return currentHash;
    }
}
