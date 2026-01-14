// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.33;

import {Test} from "forge-std/Test.sol";
// console and stdJson removed — fixtures are generated programmatically

import {MMRProof} from "../src/utils/MMRProof.sol";
import {MMRProofWrapper} from "./mocks/MMRProofWrapper.sol";
import {MerkleLib} from "./utils/MerkleLib.sol";

contract MMRProofTest is Test {
    // using stdJson for string;

    struct Fixture {
        bytes32[] leaves;
        Proof[] proofs;
        bytes32 rootHash;
    }

    struct Proof {
        bytes32[] items;
        uint256 order;
    }

    MMRProofWrapper public wrapper;

    function setUp() public {
        wrapper = new MMRProofWrapper();
    }

    function fixture() public pure returns (Fixture memory) {
        return generateFixture(16);
    }

    // Programmatically build a simple binary Merkle tree with `leafCount` leaves
    // and produce per-leaf inclusion proofs compatible with `MMRProof.verifyLeafProof`.
    function generateFixture(uint256 leafCount) internal pure returns (Fixture memory) {
        // require power-of-two to keep implementation simple and deterministic
        require(
            leafCount > 0 && (leafCount & (leafCount - 1)) == 0, "leafCount must be power of two"
        );

        bytes32[] memory leaves = new bytes32[](leafCount);
        for (uint256 i = 0; i < leafCount; i++) {
            leaves[i] = keccak256(abi.encodePacked("leaf:", i));
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

        bytes32 root = current[0];

        // build proofs for each leaf
        Proof[] memory proofs = new Proof[](leafCount);
        for (uint256 idx = 0; idx < leafCount; idx++) {
            // maximum depth is log2(leafCount)
            uint256 depth = 0;
            uint256 tmp = leafCount;
            while (tmp > 1) {
                tmp = tmp / 2;
                depth++;
            }

            bytes32[] memory items = new bytes32[](depth);
            uint256 orderBits = 0;
            uint256 proofLen = 0;

            // rebuild levels and collect sibling hashes
            bytes32[] memory level = leaves;
            uint256 index = idx;
            while (level.length > 1) {
                uint256 sibling = index ^ 1;
                // sibling exists because leafCount is power of two
                items[proofLen] = level[sibling];
                // if current index is odd, current node is right child => order bit = 1
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

            // trim items to actual proofLen
            bytes32[] memory trimmed = new bytes32[](proofLen);
            for (uint256 k = 0; k < proofLen; k++) {
                trimmed[k] = items[k];
            }

            proofs[idx] = Proof({items: trimmed, order: orderBits});
        }

        Fixture memory f;
        f.leaves = leaves;
        f.proofs = proofs;
        f.rootHash = root;
        return f;
    }

    function testVerifyLeafProof() public {
        Fixture memory fix = fixture();

        for (uint256 i = 0; i < fix.leaves.length; i++) {
            assertTrue(
                wrapper.verifyLeafProof(
                    fix.rootHash, fix.leaves[i], fix.proofs[i].items, fix.proofs[i].order
                )
            );
        }
    }

    // Additional smoke test that reuses the shared MerkleLib helper to build a proof
    // for a target leaf and verify it via the wrapper.
    function testVerifyLeafProofWithLib() public {
        // use the same deterministic construction as generateFixture
        bytes32[] memory leaves = new bytes32[](16);
        for (uint256 i = 0; i < 16; i++) {
            leaves[i] = keccak256(abi.encodePacked("leaf:", i));
        }
        // pick a target index and build proof via library
        uint256 target = 5;
        (bytes32 root, bytes32[] memory items, uint256 order) =
            MerkleLib.buildMerkleWithTargetLeaf(16, target, leaves[target]);
        assertTrue(wrapper.verifyLeafProof(root, leaves[target], items, order));
    }

    function testVerifyLeafProofFailsExceededProofSize() public {
        Fixture memory fix = fixture();

        vm.expectRevert(MMRProof.ProofSizeExceeded.selector);
        wrapper.verifyLeafProof(
            fix.rootHash, fix.leaves[0], new bytes32[](257), fix.proofs[0].order
        );
    }
}
