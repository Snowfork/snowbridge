// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.34;

import {Test} from "forge-std/Test.sol";
import {console} from "forge-std/console.sol";
import {ECDSA} from "openzeppelin/utils/cryptography/ECDSA.sol";
import {BeefyClient} from "../src/BeefyClient.sol";
import {Bitfield} from "../src/utils/Bitfield.sol";
import {SubstrateMerkleProof} from "../src/utils/SubstrateMerkleProof.sol";
import {CompactProofLib} from "./utils/CompactProofLib.sol";
import {ReferenceMultiRoot} from "./utils/ReferenceMultiRoot.sol";
import {SpecMultiRoot} from "./utils/SpecMultiRoot.sol";
import {MultiproofAttacks} from "./utils/MultiproofAttacks.sol";

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

    function referenceMultiRoot(
        uint256[] memory positions,
        bytes32[] memory leaves,
        uint256 width,
        bytes32[] calldata siblings
    ) external pure returns (bool, bytes32) {
        return ReferenceMultiRoot.computeMultiRoot(positions, leaves, width, siblings);
    }

    function specMultiRoot(
        uint256[] memory positions,
        bytes32[] memory leaves,
        uint256 width,
        bytes32[] calldata siblings
    ) external pure returns (bool, bytes32) {
        return SpecMultiRoot.computeMultiRoot(positions, leaves, width, siblings);
    }
}

/// @dev Real mainnet submissions in the legacy `ValidatorProof[]` format, shared with
/// integration/ForkBeefyMultiproof.t.sol.
///
/// BeefyClient: 0x7cfc5C8b341991993080Af67D940B6aD19a010E1
///
/// Each `test/data/mainnet_*.hex` is the calldata of the call to BeefyClient, unmodified. The
/// Fiat-Shamir one is the inner call of a Multicall3 `aggregate3`. Validator sets are as held
/// by BeefyClient at `blockNumber - 1`.
abstract contract MainnetBeefyFixture is Test {
    address constant BC = 0x7cfc5C8b341991993080Af67D940B6aD19a010E1;
    address constant RELAYER = 0xBa9bC9a8Aa87872f7B990031bde984A00b9CEd49;

    struct MainnetTx {
        string name;
        string file;
        uint256 blockNumber;
        bool fiatShamir;
        // Signed by the next validator set.
        bool handover;
        uint64 vsetId;
        uint256 vsetLength;
        bytes32 vsetRoot;
    }

    /// https://etherscan.io/tx/0xe8eb06c8e18879418408e255b0fd0e9cc72b9c668638f324735b98a0cb045936
    /// `submitFinal`, 28 proofs, handover to set 5688.
    function finalE8eb06() internal pure returns (MainnetTx memory) {
        return MainnetTx({
            name: "submitFinal e8eb06",
            file: "test/data/mainnet_submitFinal_e8eb06.hex",
            blockNumber: 26_046_024,
            fiatShamir: false,
            handover: true,
            vsetId: 5688,
            vsetLength: 600,
            vsetRoot: 0x3318c64b0931ea8df8f30ee9d6d890009899c0596f6446770c27bac21bab964e
        });
    }

    /// https://etherscan.io/tx/0x992ebb00fd02356eed88924625d19ff9b2c3aba1a892f0dbdee4d0727485a544
    /// `submitFinal`, 28 proofs, handover to set 5693.
    function final992ebb() internal pure returns (MainnetTx memory) {
        return MainnetTx({
            name: "submitFinal 992ebb",
            file: "test/data/mainnet_submitFinal_992ebb.hex",
            blockNumber: 26_051_984,
            fiatShamir: false,
            handover: true,
            vsetId: 5693,
            vsetLength: 600,
            vsetRoot: 0xf4284b693365c3127227bacb694bcaf610eed8bb79182537489d784377b8c56f
        });
    }

    /// https://etherscan.io/tx/0x0a9f5a3c4f0ef26f0914312ffbf7608e8b56a2821dd16daba8895bc6cd683ff2
    /// `submitFiatShamir` via Multicall3, 111 proofs, current set 5694.
    function fiatShamir0a9f5a() internal pure returns (MainnetTx memory) {
        return MainnetTx({
            name: "submitFiatShamir 0a9f5a",
            file: "test/data/mainnet_submitFiatShamir_0a9f5a.hex",
            blockNumber: 26_053_670,
            fiatShamir: true,
            handover: false,
            vsetId: 5694,
            vsetLength: 600,
            vsetRoot: 0xf4284b693365c3127227bacb694bcaf610eed8bb79182537489d784377b8c56f
        });
    }

    /// `nextValidatorSet()` at the Fiat-Shamir fixture's block.
    function nextOf5694() internal pure returns (BeefyClient.ValidatorSet memory) {
        return BeefyClient.ValidatorSet({
            id: 5695,
            length: 600,
            root: 0x4b79641cb341e45f54e4fd644b676628c5afd3b60ca7972577e5ec210d59fbb9
        });
    }

    function mainnetTxs() internal pure returns (MainnetTx[] memory txs) {
        txs = new MainnetTx[](3);
        txs[0] = finalE8eb06();
        txs[1] = final992ebb();
        txs[2] = fiatShamir0a9f5a();
    }

    /// `submitFinal` and `submitFiatShamir` share the legacy argument layout.
    function decodeLegacy(bytes calldata cd)
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

    function _load(MainnetTx memory t)
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
        bytes memory legacyCd = vm.parseBytes(vm.readFile(t.file));
        (commitment, bitfield, proofs, leaf, leafProof, leafProofOrder) =
            this.decodeLegacy(legacyCd);
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
contract MainnetSubmitFinalMultiproofTest is MainnetBeefyFixture {
    MerkleHarness harness;

    function setUp() public {
        harness = new MerkleHarness();
    }

    /// Offline: every real legacy path verifies, and the same leaves -- derived from the
    /// recovered signers, as the contract does -- reproduce the root as one multiproof.
    function testMainnetMultiproofLibraryAcceptsRealProofs() public {
        MainnetTx[] memory txs = mainnetTxs();
        for (uint256 t = 0; t < txs.length; t++) {
            _checkLibrary(txs[t]);
        }
    }

    /// Replays the Fiat-Shamir tx on a fresh BeefyClient. It needs no ticket, so no fork.
    function testMainnetFiatShamirReplaysOffline() public {
        MainnetTx memory t = fiatShamir0a9f5a();
        (
            BeefyClient.Commitment memory commitment,
            uint256[] memory bitfield,
            BeefyClient.ValidatorProof[] memory proofs,
            BeefyClient.MMRLeaf memory leaf,
            bytes32[] memory leafProof,
            uint256 leafProofOrder
        ) = _load(t);

        BeefyClient bc = new BeefyClient(
            3,
            8,
            16,
            111,
            0,
            BeefyClient.ValidatorSet({
                id: t.vsetId, length: uint128(t.vsetLength), root: t.vsetRoot
            }),
            nextOf5694()
        );

        uint256[] memory sample = Bitfield.toIndices(
            bc.createFiatShamirFinalBitfield(commitment, bitfield), proofs.length
        );
        for (uint256 i = 0; i < proofs.length; i++) {
            assertEq(sample[i], proofs[i].index, "Fiat-Shamir sample differs from mainnet");
        }

        bc.submitFiatShamir(
            commitment,
            bitfield,
            CompactProofLib.toCompact(proofs, t.vsetLength),
            leaf,
            leafProof,
            leafProofOrder
        );
        assertEq(bc.latestBeefyBlock(), commitment.blockNumber, "beefy block");
    }

    /// The attacks of `MultiproofAttacks` on each real multiproof: none gives the live root,
    /// in the on-chain library or either reference.
    function testMainnetMultiproofAttacksFail() public {
        MainnetTx[] memory txs = mainnetTxs();
        for (uint256 t = 0; t < txs.length; t++) {
            _attack(txs[t]);
        }
    }

    function _attack(MainnetTx memory t) internal {
        (uint256[] memory positions, bytes32[] memory leaves, bytes32[] memory siblings) =
            _realMultiproof(t);
        uint256 n = positions.length;
        uint256 width = t.vsetLength;
        uint256[] memory p;
        bytes32[] memory l;

        for (uint256 k = 0; k < 3; k++) {
            uint256 i = k * (n - 1) / 2;
            (p, l) = MultiproofAttacks.duplicateLeaf(positions, leaves, i, keccak256("forged"));
            _assertNotRoot(
                t, p, l, MultiproofAttacks.padSiblings(siblings, keccak256("pad")), "duplicate"
            );
            (p, l) = MultiproofAttacks.duplicateLeaf(positions, leaves, i, leaves[i]);
            _assertNotRoot(t, p, l, siblings, "duplicate of a real leaf");

            uint256 x = MultiproofAttacks.unsampled(positions, width, k * width / 3);
            (p, l) = MultiproofAttacks.insertLeaf(positions, leaves, x, keccak256("forged"));
            _assertNotRoot(t, p, l, siblings, "inserted leaf");
        }
        for (uint256 seed = 1; seed <= 4; seed++) {
            _assertNotRoot(
                t, positions, leaves, MultiproofAttacks.shuffle(siblings, seed), "shuffled"
            );
        }
        for (uint256 i = 0; i < n; i++) {
            (p, l) = MultiproofAttacks.removeLeaf(positions, leaves, i);
            _assertNotRoot(t, p, l, siblings, "missing leaf");
        }
    }

    function _assertNotRoot(
        MainnetTx memory t,
        uint256[] memory p,
        bytes32[] memory l,
        bytes32[] memory s,
        string memory what
    ) internal view {
        (bool ok, bytes32 root) = harness.computeMultiRoot(p, l, t.vsetLength, s);
        assertFalse(ok && root == t.vsetRoot, string.concat(t.name, ": ", what));
        (bool refOk, bytes32 refRoot) = harness.referenceMultiRoot(p, l, t.vsetLength, s);
        (bool specOk, bytes32 specRoot) = harness.specMultiRoot(p, l, t.vsetLength, s);
        assertTrue(
            ok == refOk && ok == specOk && root == refRoot && root == specRoot,
            string.concat(t.name, ": implementations disagree on ", what)
        );
    }

    /// The real proofs as one multiproof, with leaves from the recovered signers.
    function _realMultiproof(MainnetTx memory t)
        internal
        returns (uint256[] memory positions, bytes32[] memory leaves, bytes32[] memory siblings)
    {
        (
            BeefyClient.Commitment memory commitment,,
            BeefyClient.ValidatorProof[] memory proofs,,,
        ) = _load(t);
        bytes32 commitmentHash = _deploy().computeCommitmentHash(commitment);
        positions = new uint256[](proofs.length);
        leaves = new bytes32[](proofs.length);
        for (uint256 i = 0; i < proofs.length; i++) {
            positions[i] = proofs[i].index;
            leaves[i] = keccak256(
                abi.encodePacked(
                    ECDSA.recover(commitmentHash, proofs[i].v, proofs[i].r, proofs[i].s)
                )
            );
        }
        siblings = CompactProofLib.toCompact(proofs, t.vsetLength).siblings;
    }

    function _checkLibrary(MainnetTx memory t) internal {
        (
            BeefyClient.Commitment memory commitment,,
            BeefyClient.ValidatorProof[] memory proofs,,,
        ) = _load(t);
        assertEq(
            commitment.validatorSetID, t.vsetId, string.concat(t.name, ": signed by another set")
        );

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
                harness.computeRoot(leaves[i], proofs[i].index, t.vsetLength, proofs[i].proof);
            assertTrue(ok, string.concat(t.name, ": legacy path structurally invalid"));
            assertEq(got, t.vsetRoot, string.concat(t.name, ": legacy path root mismatch"));
            legacySiblingCount += proofs[i].proof.length;
        }

        BeefyClient.CompactValidatorProofs memory compact =
            CompactProofLib.toCompact(proofs, t.vsetLength);

        (bool multiOk, bytes32 multiRoot) =
            harness.computeMultiRoot(positions, leaves, t.vsetLength, compact.siblings);
        assertTrue(multiOk, string.concat(t.name, ": multiproof structurally invalid"));
        assertEq(multiRoot, t.vsetRoot, string.concat(t.name, ": multiproof root mismatch"));
        (multiOk, multiRoot) =
            harness.referenceMultiRoot(positions, leaves, t.vsetLength, compact.siblings);
        assertTrue(multiOk && multiRoot == t.vsetRoot, string.concat(t.name, ": reference"));
        (multiOk, multiRoot) =
            harness.specMultiRoot(positions, leaves, t.vsetLength, compact.siblings);
        assertTrue(multiOk && multiRoot == t.vsetRoot, string.concat(t.name, ": spec model"));

        console.log(t.name);
        console.log("  legacy proofs", proofs.length);
        console.log("  legacy siblings", legacySiblingCount);
        console.log("  multiproof siblings", compact.siblings.length);
        assertLt(compact.siblings.length, legacySiblingCount, "expected sibling compression");
    }
}
