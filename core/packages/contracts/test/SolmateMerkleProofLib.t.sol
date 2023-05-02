// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "forge-std/console.sol";
import "forge-std/Test.sol";

import {MerkleProofLib} from "solmate/utils/MerkleProofLib.sol";

contract SolmateMerkleProofLibTest is Test {
    function testVerifyEmptyMerkleProofSuppliedLeafAndRootSame() public {
        bytes32[] memory proof;
        assertEq(this.verify(proof, 0x00, 0x00), true);
    }

    function testVerifyEmptyMerkleProofSuppliedLeafAndRootDifferent() public {
        bytes32[] memory proof;
        bytes32 leaf = "a";
        assertEq(this.verify(proof, 0x00, leaf), false);
    }

    function testVerifyValidProofSupplied() public {
        // Merkle tree created from leaves ['a', 'b', 'c'].
        // Leaf is 'a'.
        bytes32[] memory proof = new bytes32[](2);
        proof[0] = 0xb5553de315e0edf504d9150af82dafa5c4667fa618ed0a6f19c69b41166c5510;
        proof[1] = 0x0b42b6393c1f53060fe3ddbfcd7aadcca894465a5a438f69c87d790b2299b9b2;
        bytes32 root = 0x5842148bc6ebeb52af882a317c765fccd3ae80589b21a9b8cbf21abb630e46a7;
        bytes32 leaf = 0x3ac225168df54212a25c1c01fd35bebfea408fdac2e31ddd6f80a4bbf9a5f1cb;
        assertEq(this.verify(proof, root, leaf), true);
    }

    function testVerifyShortProofSupplied() public {
        // Merkle tree created from leaves ['a', 'b', 'c'].
        // Leaf is 'c'.
        bytes32[] memory proof = new bytes32[](1);
        proof[0] = 0x805b21d846b189efaeb0377d6bb0d201b3872a363e607c25088f025b0c6ae1f8;
        bytes32 root = 0x5842148bc6ebeb52af882a317c765fccd3ae80589b21a9b8cbf21abb630e46a7;
        bytes32 leaf = 0x0b42b6393c1f53060fe3ddbfcd7aadcca894465a5a438f69c87d790b2299b9b2;

        assertEq(this.verify(proof, root, leaf), true);
    }

    function testVerifyLargeProofSupplied() public {
        // Merkle tree created from leaves ['1', '2', '3', ..., '1000'].
        // Leaf is '42'.
        bytes32[] memory proof = new bytes32[](10);
        proof[0] = 0xcb7c14ce178f56e2e8d86ab33ebc0ae081ba8556a00cd122038841867181caac;
        proof[1] = 0x081c6649d757735e00e82890581518fc42224f0942420fe385ccb1ee67fb5c34;
        proof[2] = 0x6e077cb1dd754700e9f78cb6107b091478fbc6d9039792a4dcdecdee8c316f28;
        proof[3] = 0x06bb7e8a1517e2c41f6583fb693aa50f81e2db3e8aee43b105732f425f26b832;
        proof[4] = 0x4ecb91ca93d01dbee0cd88bec35d95e275de0f3c76a651c57b626bbee417e14e;
        proof[5] = 0xf78b5b2a0d2098d5690099983e7875e3a2274b985023d3cf48088b168f543b74;
        proof[6] = 0x2084f0e5584a39ae0c863c1f95799baa90aafe318185f6a9f67ae11098efbf99;
        proof[7] = 0x355564742ce4c22df4630c1a84466e23dd8339343fe3e2a67bfb0adce6a4d626;
        proof[8] = 0xc751a2cbefe4b10e2108a92131dfda6788a4059aa5a9b87aa001935a189de909;
        proof[9] = 0x698159e44c47a9a66f4efa7fdf710e7b00ae3c3d95522376cacc3580ad5600d3;
        bytes32 root = 0x147c1ac0abf462d97b0f59c5d3616f81c96566d1e2ddab5359f16c3bf0b48cc9;
        bytes32 leaf = 0xbeced09521047d05b8960b7e7bcc1d1292cf3e4b2a6b63f48335cbde5f7545d2;

        assertEq(this.verify(proof, root, leaf), true);
    }

    function testVerifyInvalidProofSupplied() public {
        // Merkle tree created from leaves ['a', 'b', 'c'].
        // Leaf is 'a'.
        // Proof is same as testValidProofSupplied but last byte of first element is modified.
        bytes32[] memory proof = new bytes32[](2);
        proof[0] = 0xb5553de315e0edf504d9150af82dafa5c4667fa618ed0a6f19c69b41166c5511;
        proof[1] = 0x0b42b6393c1f53060fe3ddbfcd7aadcca894465a5a438f69c87d790b2299b9b2;
        bytes32 root = 0x5842148bc6ebeb52af882a317c765fccd3ae80589b21a9b8cbf21abb630e46a7;
        bytes32 leaf = 0x3ac225168df54212a25c1c01fd35bebfea408fdac2e31ddd6f80a4bbf9a5f1cb;
        assertEq(this.verify(proof, root, leaf), false);
    }

    function testFuzz_RandomProof(uint8 leafCount, uint8 leafIndex) public {
        vm.assume(leafIndex < leafCount);

        // most significant bit of leafCount is the number of nodes in the second-last layer of a complete binary tree
        // with leafCount leaves
        // most-significant bit of leafCount = leafCount -> reverse -> least-significant bit -> reverse
        uint8 bitReversedLeafCount = reverseBitsU8(leafCount);
        uint8 lsb = bitReversedLeafCount & (~bitReversedLeafCount+1);
        uint8 secondLastLayerCount = reverseBitsU8(lsb);
        uint256 lastLayerCount = 2*(leafCount - secondLastLayerCount);

        bytes32[] memory tree = new bytes32[](2*uint256(leafCount)-1);

        for (uint256 i = 0; i < tree.length; i++) {
            if (i < lastLayerCount) {
                // store bottom layer leaf hash
                tree[tree.length - 1 - i] = keccak256(abi.encodePacked(lastLayerCount - 1 - i));
            } else if (i < leafCount) {
                // store second-last layer leaf hash
                tree[tree.length - 1 - i] = keccak256(abi.encodePacked(leafCount - 1 - (i - lastLayerCount)));
            } else {
                // store sorted concatenation of leaf hashes
                uint256 leftIndex = 2*(tree.length - 1 - i) + 1;
                tree[tree.length - 1 - i] = hashSorted(tree[leftIndex], tree[leftIndex + 1]);
            }
        }

        // Proof size is log2 of secondLastLayerCount, which is always a power of 2 in a complete tree
        uint8 proofSize = 0;
        // Count the zeroes after the single bit set in secondLastLayerCount
        // Subtracting 1 sets all bits smaller than the single bit set in secondLastLayerCount
        // Then use Kernighan's algorithm to count the set bits
        uint8 n = secondLastLayerCount - 1;
        while(n != 0) {
            n &= n - 1;
            proofSize++;
        }
        // Add 1 when the leaf is in the last layer of a non-perfect tree
        if (leafIndex < lastLayerCount && lastLayerCount != leafCount) {
            proofSize += 1;
        }

        bytes32[] memory proof = new bytes32[](proofSize);
        uint256 current;
        if (leafIndex < lastLayerCount) {
            current = tree.length - lastLayerCount + leafIndex;
        } else {
            current = tree.length - leafCount + leafIndex - lastLayerCount ;
        }

        uint256 sibling;
        for (uint8 i = 0; i < proof.length; i++) {
            if (current % 2 == 0) {
                sibling = current - 1;
            } else {
                sibling = current + 1;
            }
            proof[i] = tree[sibling];

            current = (current - 1) / 2;
        }

        assertEq(this.verify(proof, tree[0], keccak256(abi.encodePacked(uint256(leafIndex)))), true);
    }

    function testFuzzSample() public {
        uint8 leafCount = 3;
        uint8 leafIndex = 1;

        // most significant bit of leafCount is the number of nodes in the second-last layer of a complete binary tree
        // with leafCount leaves
        // most-significant bit of leafCount = leafCount -> reverse -> least-significant bit -> reverse
        uint8 bitReversedLeafCount = reverseBitsU8(leafCount);
        uint8 lsb = bitReversedLeafCount & (~bitReversedLeafCount+1);
        uint8 secondLastLayerCount = reverseBitsU8(lsb);
        uint256 lastLayerCount = 2*(leafCount - secondLastLayerCount);

        bytes32[] memory tree = new bytes32[](2*uint256(leafCount)-1);

        for (uint256 i = 0; i < tree.length; i++) {
            if (i < lastLayerCount) {
                // store bottom layer leaf hash
                tree[tree.length - 1 - i] = keccak256(abi.encodePacked(lastLayerCount - 1 - i));
            } else if (i < leafCount) {
                // store second-last layer leaf hash
                tree[tree.length - 1 - i] = keccak256(abi.encodePacked(leafCount - 1 - (i - lastLayerCount)));
            } else {
                // store sorted concatenation of leaf hashes
                uint256 leftIndex = 2*(tree.length - 1 - i) + 1;
                tree[tree.length - 1 - i] = hashSorted(tree[leftIndex], tree[leftIndex + 1]);
            }
        }

        // Proof size is log2 of secondLastLayerCount, which is always a power of 2 in a complete tree
        uint8 proofSize = 0;
        // Count the zeroes after the bit set in secondLastLayerCount
        // Set all bits smaller than the single bit set in secondLastLayerCount
        uint8 n = secondLastLayerCount - 1;
        while(n != 0) {
            n &= n - 1;
            proofSize++;
        }
        // Add 1 when the leaf is in the last layer of a non-perfect tree
        if (leafIndex < lastLayerCount && lastLayerCount != leafCount) {
            proofSize += 1;
        }

        bytes32[] memory proof = new bytes32[](proofSize);
        uint256 current;
        if (leafIndex < lastLayerCount) {
            current = tree.length - lastLayerCount + leafIndex;
        } else {
            current = tree.length - leafCount + leafIndex - lastLayerCount ;
        }

        uint256 sibling;
        for (uint8 i = 0; i < proof.length; i++) {
            if (current % 2 == 0) {
                sibling = current - 1;
            } else {
                sibling = current + 1;
            }
            proof[i] = tree[sibling];

            current = (current - 1) / 2;
        }

        assertEq(this.verify(proof, tree[0], keccak256(abi.encodePacked(uint256(leafIndex)))), true);
    }

    function verify(
        bytes32[] calldata proof,
        bytes32 root,
        bytes32 leaf
    ) external pure returns (bool) {
        return MerkleProofLib.verify(proof, root, leaf);
    }

    function hashSorted(bytes32 a, bytes32 b) internal pure returns (bytes32) {
        if(a < b) {
            return keccak256(abi.encodePacked(a, b));
        } else {
            return keccak256(abi.encodePacked(b, a));
        }
    }

    function reverseBitsU8(uint8 n) internal pure returns (uint8) {
        // swap adjacent bits, then pairs, then nibbles
        uint8 reversed = (n & 0xAA) >> 1 | (n & 0x55) << 1;
        reversed = (reversed & 0xCC) >> 2 | (reversed & 0x33) << 2;
        reversed = (reversed & 0xF0) >> 4 | (reversed & 0x0F) << 4;
        return reversed;
    }
}
