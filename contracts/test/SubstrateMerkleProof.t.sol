// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import {Test} from "forge-std/Test.sol";
import {SubstrateMerkleProof} from "../src/utils/SubstrateMerkleProof.sol";
import {MerkleLibSubstrate} from "./utils/MerkleLib.sol";

contract SubstrateMerkleProofWrapper {
    function verify(
        bytes32 root,
        bytes32 leaf,
        uint256 position,
        uint256 width,
        bytes32[] calldata proof
    ) external pure returns (bool) {
        return SubstrateMerkleProof.verify(root, leaf, position, width, proof);
    }
}

contract SubstrateMerkleProofTest is Test {
    SubstrateMerkleProofWrapper public wrapper;

    function setUp() public {
        wrapper = new SubstrateMerkleProofWrapper();
    }

    function efficientHash(bytes32 a, bytes32 b) internal pure returns (bytes32 value) {
        return MerkleLibSubstrate.hashPair(a, b);
    }

    function buildNextLevel(bytes32[] memory level) internal pure returns (bytes32[] memory) {
        return MerkleLibSubstrate.nextLevel(level);
    }

    function computeDepth(uint256 width) internal pure returns (uint256 depth) {
        return MerkleLibSubstrate.depth(width);
    }

    function generateLeaves(uint256 width) internal pure returns (bytes32[] memory leaves) {
        return MerkleLibSubstrate.genLeaves(width);
    }

    function generateProofFor(bytes32[] memory leaves, uint256 index)
        internal
        pure
        returns (bytes32[] memory proof)
    {
        return MerkleLibSubstrate.genProof(leaves, index);
    }

    function computeRootFromLeaves(bytes32[] memory leaves) internal pure returns (bytes32) {
        return MerkleLibSubstrate.rootFromLeaves(leaves);
    }

    function testVerifyAllLeavesEvenWidth() public {
        uint256 width = 16; // arbitrary width (power-of-two or not)
        bytes32[] memory leaves = generateLeaves(width);
        bytes32 root = computeRootFromLeaves(leaves);

        for (uint256 i = 0; i < width; i++) {
            bytes32[] memory proof = generateProofFor(leaves, i);
            assertTrue(wrapper.verify(root, leaves[i], i, width, proof));

            // negative: tamper with leaf
            bytes32 badLeaf = keccak256(abi.encodePacked("bad:", i));
            assertFalse(wrapper.verify(root, badLeaf, i, width, proof));

            // negative: tamper with proof (flip first proof element)
            if (proof.length > 0) {
                bytes32 orig = proof[0];
                proof[0] = bytes32(0);
                assertFalse(wrapper.verify(root, leaves[i], i, width, proof));
                proof[0] = orig;
            }
        }
    }

    function testVerifyAllLeavesOddWidth() public {
        uint256 width = 15; // non-power-of-two width
        bytes32[] memory leaves = generateLeaves(width);
        bytes32 root = computeRootFromLeaves(leaves);

        for (uint256 i = 0; i < width; i++) {
            bytes32[] memory proof = generateProofFor(leaves, i);
            assertTrue(wrapper.verify(root, leaves[i], i, width, proof));
        }
    }

    function testInvalidPosition() public {
        uint256 width = 8;
        bytes32[] memory leaves = generateLeaves(width);
        bytes32 root = computeRootFromLeaves(leaves);
        bytes32[] memory proof = generateProofFor(leaves, 0);
        // position >= width should return false
        assertFalse(wrapper.verify(root, leaves[0], width, width, proof));
    }
}
