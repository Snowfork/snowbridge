pragma solidity ^0.8.5;

struct SimplifiedMMRProof {
    bytes32[] restOfThePeaks;
    bytes32 rightBaggedPeak;
    bytes32[] merkleProofItems;
    uint64 merkleProofOrderBitField;
}

contract  SimplifiedMMRVerification {
    function verifyInclusionProof(
        bytes32 root,
        bytes32 leafNodeHash,
        SimplifiedMMRProof memory proof
    ) public pure returns (bool) {
        require(proof.merkleProofItems.length < 64);

        bool hasRightBaggedPeak = proof.rightBaggedPeak != 0x0;

        uint numberOfPeaks = 1 + proof.restOfThePeaks.length;
        if (hasRightBaggedPeak) {
            numberOfPeaks++;
        }

        bytes32[] memory reversedPeaks = new bytes32[](numberOfPeaks);
        uint peakInsertionPointer = 0;

        if (hasRightBaggedPeak) {
            reversedPeaks[peakInsertionPointer++] = proof.rightBaggedPeak;
        }

        bytes32 merkleRootPeak = calculateMerkleRoot(leafNodeHash, proof.merkleProofItems, proof.merkleProofOrderBitField);
        reversedPeaks[peakInsertionPointer++] = merkleRootPeak;

        if (proof.restOfThePeaks.length > 0) {
            for (uint i = 0; i < proof.restOfThePeaks.length; i++) {
                reversedPeaks[peakInsertionPointer] = proof.restOfThePeaks[proof.restOfThePeaks.length - i - 1];
                peakInsertionPointer++;
            }
        }

        bytes32 mmrRoot = bagPeaks(reversedPeaks);

        return mmrRoot == root;
    }

    function bagPeaks(bytes32[] memory reversedPeaks) internal pure returns (bytes32) {
        require(reversedPeaks.length > 0);
        bytes32 bag = reversedPeaks[0];
        uint currentIndex = 1;

        while (currentIndex < reversedPeaks.length) {
            bag = keccak256(
                abi.encodePacked(bag, reversedPeaks[currentIndex++])
            );
        }

        return bag;
    }

    // Get the value of the bit at the given 'index' in 'self'.
    // index should be validated beforehand to make sure it is less than 64
    function bit(uint64 self, uint index) internal pure returns (bool) {
        if (uint8(self >> index & 1) == 1) {
            return true;
        } else {
            return false;
        }
    }

    function calculateMerkleRoot(
        bytes32 leafNodeHash,
        bytes32[] memory merkleProofItems,
        uint64 merkleProofOrderBitField
    ) internal pure returns (bytes32) {
        bytes32 currentHash = leafNodeHash;

        for (uint currentPosition = 0; currentPosition < merkleProofItems.length; currentPosition++) {
            bool isSiblingLeft = bit(merkleProofOrderBitField, currentPosition);
            bytes32 sibling = merkleProofItems[currentPosition];

            if (isSiblingLeft) {
                currentHash = keccak256(
                    abi.encodePacked(sibling, currentHash)
                );
            } else {
                currentHash = keccak256(
                    abi.encodePacked(currentHash, sibling)
                );
            }
        }

        return currentHash;
    }
}
