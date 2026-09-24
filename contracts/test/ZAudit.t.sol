// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.34;

import {Test} from "forge-std/Test.sol";
import {Bits} from "../src/utils/Bits.sol";
import {Bitfield} from "../src/utils/Bitfield.sol";
import {SubstrateMerkleProof} from "../src/utils/SubstrateMerkleProof.sol";

contract ZAudit is Test {
    using Bits for uint256;

    // ---- 1. lowestBitSet is now load-bearing; it had one hand-picked test value ----
    function testFuzz_lowestBitSet(uint256 x) public pure {
        vm.assume(x != 0);
        uint8 naive;
        uint256 v = x;
        while (v & 1 == 0) {
            v >>= 1;
            naive++;
        }
        assertEq(x.lowestBitSet(), naive, "lowestBitSet disagrees with naive ctz");
    }

    // ---- 2. toIndices must equal a naive ascending scan, for ARBITRARY bitfields ----
    function testFuzz_toIndicesMatchesNaiveScan(uint256 a, uint256 b, uint256 c) public pure {
        uint256[] memory bf = new uint256[](3);
        bf[0] = a;
        bf[1] = b;
        bf[2] = c;

        uint256 n;
        for (uint256 i = 0; i < 768; i++) {
            if (Bitfield.isSet(bf, i)) n++;
        }
        vm.assume(n > 0);

        uint256[] memory got = Bitfield.toIndices(bf, n);
        assertEq(got.length, n);

        uint256 k;
        for (uint256 i = 0; i < 768; i++) {
            if (Bitfield.isSet(bf, i)) {
                assertEq(got[k], i, "toIndices disagrees with naive scan");
                if (k > 0) assertGt(got[k], got[k - 1], "not strictly ascending");
                k++;
            }
        }
    }

    // ---- 3. A non-canonical proof length must only ever REJECT ----
    // computeRoot walks by geometry and requires exact consumption, so a wrong length must fail
    // closed. Fuzz it directly with deliberately wrong lengths.
    function testFuzz_wrongProofLengthAlwaysRejects(
        uint256 wSeed,
        uint256 pSeed,
        uint256 lenSeed,
        bytes32 root,
        bytes32 leaf
    ) public view {
        uint256 width = bound(wSeed, 2, 700);
        uint256 position = bound(pSeed, 0, width - 1);
        uint256 canonical = canonicalPathLength(position, width);
        uint256 wrongLen = bound(lenSeed, 0, 20);
        vm.assume(wrongLen != canonical);

        bytes32[] memory proof = new bytes32[](wrongLen);
        for (uint256 i = 0; i < wrongLen; i++) {
            proof[i] = keccak256(abi.encode(root, i));
        }
        assertFalse(
            this.verifyExternal(root, leaf, position, width, proof),
            "non-canonical proof length verified"
        );
    }

    function verifyExternal(
        bytes32 root,
        bytes32 leaf,
        uint256 position,
        uint256 width,
        bytes32[] calldata proof
    ) external pure returns (bool) {
        return SubstrateMerkleProof.verify(root, leaf, position, width, proof);
    }

    // ---- 4. An out-of-range position must fail closed through computeRootAt and verify ----
    function testFuzz_outOfRangePositionFailsClosed(uint256 wSeed, uint256 over, bytes32 root)
        public
        view
    {
        uint256 width = bound(wSeed, 1, 700);
        uint256 position = width + bound(over, 0, 1000);
        (bool valid,,) =
            this.computeRootAtExternal(bytes32(uint256(1)), position, width, new bytes32[](0), 0);
        assertFalse(valid, "computeRootAt accepted an out-of-range position");
        assertFalse(
            this.verifyExternal(root, bytes32(uint256(1)), position, width, new bytes32[](0)),
            "out-of-range position verified"
        );
    }

    function computeRootAtExternal(
        bytes32 leaf,
        uint256 position,
        uint256 width,
        bytes32[] calldata siblings,
        uint256 offset
    ) external pure returns (bool, bytes32, uint256) {
        return SubstrateMerkleProof.computeRootAt(leaf, position, width, siblings, offset);
    }

    function canonicalPathLength(uint256 p, uint256 w) internal pure returns (uint256 n) {
        while (w > 1) {
            if (!(p + 1 == w && w & 1 == 1)) {
                n++;
            }
            p >>= 1;
            w = ((w - 1) >> 1) + 1;
        }
    }
}
