// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.34;

import {Test} from "forge-std/Test.sol";
import {console} from "forge-std/console.sol";
import {ECDSA} from "openzeppelin/utils/cryptography/ECDSA.sol";
import {BeefyClient} from "../src/BeefyClient.sol";
import {SubstrateMerkleProof} from "../src/utils/SubstrateMerkleProof.sol";
import {CompactProofLib} from "./utils/CompactProofLib.sol";

/// memory->calldata bridge for the on-chain library.
contract MerkleHarness {
    function computeRoot(bytes32 leaf, uint256 position, uint256 width, bytes32[] calldata proof)
        external
        pure
        returns (bool, bytes32)
    {
        return SubstrateMerkleProof.computeRoot(leaf, position, width, proof);
    }

    function computeMultiRoot(
        uint256[] memory positions,
        bytes32[] memory leaves,
        uint256 width,
        bytes32[] calldata siblings
    ) external pure returns (bool, bytes32) {
        return SubstrateMerkleProof.computeMultiRoot(positions, leaves, width, siblings);
    }
}

/// @dev A real mainnet `submitFinal` in the legacy `ValidatorProof[]` format, shared by the
/// offline test below and the fork replay in integration/ForkBeefyMultiproof.t.sol.
///
/// Tx: https://etherscan.io/tx/0xe8eb06c8e18879418408e255b0fd0e9cc72b9c668638f324735b98a0cb045936
/// BeefyClient: 0x7cfc5C8b341991993080Af67D940B6aD19a010E1
///
/// `test/data/mainnet_submitFinal_e8eb06.hex` is that tx's calldata, unmodified.
abstract contract MainnetSubmitFinalFixture is Test {
    address constant BC = 0x7cfc5C8b341991993080Af67D940B6aD19a010E1;
    address constant RELAYER = 0xBa9bC9a8Aa87872f7B990031bde984A00b9CEd49;
    uint256 constant FINAL_BLOCK = 26_046_024;
    string constant FINAL_FILE = "test/data/mainnet_submitFinal_e8eb06.hex";

    // The commitment is signed by validator set 5688, which was the *next* set at
    // FINAL_BLOCK - 1 (`nextValidatorSet()` on the live contract). Every legacy path in the tx
    // must verify against this root, so a wrong constant fails the test.
    uint64 constant VSET_ID = 5688;
    uint256 constant VSET_LENGTH = 600;
    bytes32 constant VSET_ROOT =
        0x3318c64b0931ea8df8f30ee9d6d890009899c0596f6446770c27bac21bab964e;

    function decodeFinal(bytes calldata cd)
        external
        pure
        returns (
            BeefyClient.Commitment memory c,
            uint256[] memory bf,
            BeefyClient.ValidatorProof[] memory ps,
            BeefyClient.MMRLeaf memory leaf,
            bytes32[] memory leafProof,
            uint256 order
        )
    {
        (c, bf, ps, leaf, leafProof, order) = abi.decode(
            cd[4:],
            (
                BeefyClient.Commitment,
                uint256[],
                BeefyClient.ValidatorProof[],
                BeefyClient.MMRLeaf,
                bytes32[],
                uint256
            )
        );
    }

    function _load()
        internal
        view
        returns (
            BeefyClient.Commitment memory commitment,
            uint256[] memory bitfield,
            BeefyClient.ValidatorProof[] memory proofs,
            BeefyClient.MMRLeaf memory leaf,
            bytes32[] memory leafProof,
            uint256 leafProofOrder
        )
    {
        bytes memory legacyCd = vm.parseBytes(vm.readFile(FINAL_FILE));
        (commitment, bitfield, proofs, leaf, leafProof, leafProofOrder) =
            this.decodeFinal(legacyCd);
    }

    function _deploy() internal returns (BeefyClient) {
        BeefyClient.ValidatorSet memory d0 =
            BeefyClient.ValidatorSet({id: 0, length: 1, root: bytes32(0)});
        BeefyClient.ValidatorSet memory d1 =
            BeefyClient.ValidatorSet({id: 1, length: 1, root: bytes32(0)});
        return new BeefyClient(3, 8, 16, 111, 0, d0, d1);
    }
}

/// Can the legacy proofs be reassembled into a multiproof the contract accepts? Runs offline.
contract MainnetSubmitFinalMultiproofTest is MainnetSubmitFinalFixture {
    MerkleHarness harness;

    function setUp() public {
        harness = new MerkleHarness();
    }

    /// Offline: every real legacy path verifies, and the same leaves -- derived from the
    /// recovered signers, as the contract does -- reproduce the root as one multiproof.
    function testMainnetMultiproofLibraryAcceptsRealProofs() public {
        (
            BeefyClient.Commitment memory commitment,,
            BeefyClient.ValidatorProof[] memory proofs,,,
        ) = _load();
        assertEq(commitment.validatorSetID, VSET_ID, "commitment signed by another set");

        bytes32 commitmentHash = _deploy().computeCommitmentHash(commitment);

        uint256[] memory positions = new uint256[](proofs.length);
        bytes32[] memory leaves = new bytes32[](proofs.length);
        uint256 legacySiblingCount;
        for (uint256 i = 0; i < proofs.length; i++) {
            if (i > 0) {
                assertGt(proofs[i].index, proofs[i - 1].index, "proofs not ascending");
            }
            address signer = ECDSA.recover(commitmentHash, proofs[i].v, proofs[i].r, proofs[i].s);
            assertEq(signer, proofs[i].account, "signature does not match account");

            positions[i] = proofs[i].index;
            leaves[i] = keccak256(abi.encodePacked(signer));

            (bool ok, bytes32 got) =
                harness.computeRoot(leaves[i], proofs[i].index, VSET_LENGTH, proofs[i].proof);
            assertTrue(ok, "legacy path structurally invalid");
            assertEq(got, VSET_ROOT, "legacy path root mismatch");
            legacySiblingCount += proofs[i].proof.length;
        }

        BeefyClient.CompactValidatorProofs memory compact =
            CompactProofLib.toCompact(proofs, VSET_LENGTH);

        (bool multiOk, bytes32 multiRoot) =
            harness.computeMultiRoot(positions, leaves, VSET_LENGTH, compact.siblings);
        assertTrue(multiOk, "multiproof structurally invalid");
        assertEq(multiRoot, VSET_ROOT, "multiproof root mismatch");

        console.log("legacy proofs", proofs.length);
        console.log("legacy siblings", legacySiblingCount);
        console.log("multiproof siblings", compact.siblings.length);
        assertLt(compact.siblings.length, legacySiblingCount, "expected sibling compression");
    }
}
