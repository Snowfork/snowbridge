// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.5;

struct MMRProof {
    bytes32[] items;
    uint64 order;
}

library MMRProofVerification {
    function verifyLeafProof(
        bytes32 root,
        bytes32 leafNodeHash,
        MMRProof memory proof
    ) public pure returns (bool) {
        require(proof.items.length < 64);

        return root == calculateMerkleRoot(leafNodeHash, proof.items, proof.order);
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
        bytes32 leafNodeHash,
        bytes32[] memory items,
        uint64 order
    ) internal pure returns (bytes32) {
        bytes32 currentHash = leafNodeHash;

        for (uint256 currentPosition = 0; currentPosition < items.length; currentPosition++) {
            bool isSiblingLeft = bit(order, currentPosition);
            bytes32 sibling = items[currentPosition];

            if (isSiblingLeft) {
                currentHash = keccak256(abi.encodePacked(sibling, currentHash));
            } else {
                currentHash = keccak256(abi.encodePacked(currentHash, sibling));
            }
        }
        return currentHash;
    }
}
