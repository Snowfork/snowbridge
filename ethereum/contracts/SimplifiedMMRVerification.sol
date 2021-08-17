pragma solidity ^0.8.5;

contract  SimplifiedMMRVerification {
    function verifyInclusionProof(
        bytes32 root,
        bytes32 leafNodeHash,
        bytes32[] memory restOfThePeaks,
        bool hasRightBaggedPeak,
        bytes32 rightBaggedPeak,
        bytes32[] memory merkleProofItems,
        bool[] memory merkleProofOrder
    ) public pure returns (bool) {
        require(merkleProofOrder.length == merkleProofItems.length);
        // TODO: Do validation of other input parameters

        uint numberOfPeaks = 1 + restOfThePeaks.length;
        if (hasRightBaggedPeak) {
            numberOfPeaks++;
        }

        bytes32[] memory reversedPeaks = new bytes32[](numberOfPeaks);
        uint peakInsertionPointer = 0;

        if (hasRightBaggedPeak) {
            reversedPeaks[peakInsertionPointer++] = rightBaggedPeak;
        }

        bytes32 merkleRootPeak = calculateMerkleRoot(leafNodeHash, merkleProofItems, merkleProofOrder);
        reversedPeaks[peakInsertionPointer++] = merkleRootPeak;

        if (restOfThePeaks.length > 0) {
            for (uint i = restOfThePeaks.length - 1; i >= 0; i--) {
                reversedPeaks[peakInsertionPointer] = restOfThePeaks[i];
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

    function calculateMerkleRoot(
        bytes32 leafNodeHash,
        bytes32[] memory merkleProofItems,
        bool[] memory merkleProofOrder
    ) internal pure returns (bytes32) {
        bytes32 currentHash = leafNodeHash;

        for (uint currentPosition = 0; currentPosition < merkleProofItems.length; currentPosition++) {
            bool isSiblingLeft = merkleProofOrder[currentPosition];
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
