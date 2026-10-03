// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.34;

import {Test} from "forge-std/Test.sol";
import {BeefyClient} from "../src/BeefyClient.sol";
import {Bitfield} from "../src/utils/Bitfield.sol";
import {ScaleCodec} from "../src/utils/ScaleCodec.sol";
import {BeefyClientMock} from "./mocks/BeefyClientMock.sol";
import {MerkleLibSubstrate} from "./utils/MerkleLib.sol";
import {CompactProofLib} from "./utils/CompactProofLib.sol";

/// @dev Drives BeefyClient with random sequences of honest and forged updates, on both paths and
/// across handovers. Validator keys are derived from the set id, so every set is reproducible.
/// The attacker holds the lowest `(n - 1) / 3` keys of each set. The sample is capped at
/// `n / 3 + 1` signatures, more than the attacker holds, so every forgery must fail.
contract BeefyClientHandler is Test {
    // forge-lint: disable-next-line(unsafe-typecast)
    bytes2 constant MMR_ROOT_ID = bytes2("mh");
    uint256 constant RANDAO_DELAY = 3;

    struct ValidatorSetKeys {
        uint256[] keys;
        address[] signers;
        bytes32[][] paths;
        bytes32 root;
    }

    struct Update {
        uint64 setId;
        BeefyClient.Commitment commitment;
        bytes32 commitmentHash;
        BeefyClient.MMRLeaf leaf;
        bool handover;
    }

    BeefyClientMock public client;

    // Ghost state: what the client must hold if it accepted exactly the honest updates.
    uint32 public lastAcceptedBlock;
    bytes32 public lastAcceptedRoot;
    uint64 public ghostCurrentId = 1;
    uint64 public ghostNextId = 2;
    mapping(bytes32 => bool) public isHonestRoot;

    uint256 public accepted;
    uint256 public handovers;
    uint256 public forgedAttempts;
    uint256 public honestRejected;
    uint256 public forgedAccepted;
    bool public blockWentBack;

    uint256 nonce;
    BeefyClient.MMRLeaf noLeaf;

    constructor() {
        client = new BeefyClientMock(
            RANDAO_DELAY,
            8,
            4,
            111,
            0,
            BeefyClient.ValidatorSet(1, uint128(sizeOf(1)), keysOf(1).root),
            BeefyClient.ValidatorSet(2, uint128(sizeOf(2)), keysOf(2).root)
        );
    }

    // ---- actions ------------------------------------------------------------------------

    function honestFiatShamir(uint8 gap, bool handover) external checksBlock {
        Update memory u = honestUpdate(gap, handover);
        uint256[] memory bitfield = claimAll(sizeOf(u.setId));
        try this.fiatShamir(u, bitfield, u.commitmentHash, false) {
            recordAccepted(u);
        } catch {
            honestRejected++;
        }
    }

    function honestInteractive(uint8 gap, bool handover, uint8 who, uint256 randao)
        external
        checksBlock
    {
        Update memory u = honestUpdate(gap, handover);
        uint256[] memory bitfield = claimAll(sizeOf(u.setId));
        try this.interactive(u, bitfield, who % sizeOf(u.setId), randao, u.commitmentHash, false) {
            recordAccepted(u);
        } catch {
            honestRejected++;
        }
    }

    /// The attacker signs a forged MMR root (or a forged handover to its own validator set)
    /// with its keys, and answers every honest slot with a genuine signature over the honest
    /// commitment for the same block.
    function forgedFiatShamir(uint8 gap, bool handover, bytes32 salt) external checksBlock {
        (Update memory forged, bytes32 decoy) = forgedUpdate(gap, handover, salt);
        forgedAttempts++;
        try this.fiatShamir(forged, claimQuorum(sizeOf(forged.setId)), decoy, true) {
            forgedAccepted++;
        } catch {}
    }

    function forgedInteractive(uint8 gap, bool handover, bytes32 salt, uint256 randao)
        external
        checksBlock
    {
        (Update memory forged, bytes32 decoy) = forgedUpdate(gap, handover, salt);
        forgedAttempts++;
        // Validator 0 is the attacker's, so its initial signature is genuine.
        try this.interactive(forged, claimQuorum(sizeOf(forged.setId)), 0, randao, decoy, true) {
            forgedAccepted++;
        } catch {}
    }

    /// An honest commitment that is not newer than the latest accepted block.
    function staleHonest(uint8 back, bool interactivePath) external checksBlock {
        uint32 latest = uint32(client.latestBeefyBlock());
        Update memory u = honestUpdate(0, false);
        u.commitment.blockNumber = latest - uint32(back % (latest + 1));
        u.commitmentHash = keccak256(client.encodeCommitment_public(u.commitment));
        forgedAttempts++;
        uint256[] memory bitfield = claimAll(sizeOf(u.setId));
        if (interactivePath) {
            try this.interactive(u, bitfield, 0, 1, u.commitmentHash, false) {
                forgedAccepted++;
            } catch {}
        } else {
            try this.fiatShamir(u, bitfield, u.commitmentHash, false) {
                forgedAccepted++;
            } catch {}
        }
    }

    function rollBlocks(uint8 blocks) external checksBlock {
        vm.roll(block.number + blocks);
    }

    // ---- submission steps (external so a revert anywhere rolls the whole update back) -----

    function fiatShamir(
        Update memory u,
        uint256[] memory bitfield,
        bytes32 honestSlotsSign,
        bool attacker
    ) external {
        require(msg.sender == address(this));
        uint256[] memory sample = client.createFiatShamirFinalBitfield(u.commitment, bitfield);
        client.submitFiatShamir(
            u.commitment,
            bitfield,
            proofsFor(u, sample, honestSlotsSign, attacker),
            u.handover ? u.leaf : noLeaf,
            new bytes32[](0),
            0
        );
    }

    function interactive(
        Update memory u,
        uint256[] memory bitfield,
        uint256 initialSigner,
        uint256 randao,
        bytes32 honestSlotsSign,
        bool attacker
    ) external {
        require(msg.sender == address(this));
        ValidatorSetKeys memory k = keysOf(u.setId);
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(k.keys[initialSigner], u.commitmentHash);
        client.submitInitial(
            u.commitment,
            bitfield,
            BeefyClient.ValidatorProof(
                v, r, s, initialSigner, k.signers[initialSigner], k.paths[initialSigner]
            )
        );
        vm.roll(block.number + RANDAO_DELAY);
        vm.prevrandao(bytes32(bound(randao, 1, type(uint256).max)));
        client.commitPrevRandao(u.commitmentHash);
        uint256[] memory sample = client.createFinalBitfield(u.commitmentHash, bitfield);
        client.submitFinal(
            u.commitment,
            bitfield,
            proofsFor(u, sample, honestSlotsSign, attacker),
            u.handover ? u.leaf : noLeaf,
            new bytes32[](0),
            0
        );
    }

    // ---- building updates ---------------------------------------------------------------

    function honestUpdate(uint8 gap, bool handover) internal returns (Update memory u) {
        u.handover = handover;
        u.setId = handover ? ghostNextId : ghostCurrentId;
        bytes32 mmrRoot;
        if (handover) {
            u.leaf = leafFor(ghostNextId + 1, keysOf(ghostNextId + 1).root);
            mmrRoot = leafHash(u.leaf);
        } else {
            mmrRoot = keccak256(abi.encode("honest", nonce++));
        }
        isHonestRoot[mmrRoot] = true;
        setCommitment(u, client.latestBeefyBlock() + 1 + gap % 4, mmrRoot);
    }

    /// A forged update for the next block, and the hash of the honest commitment for that block,
    /// which honest validators did sign.
    function forgedUpdate(uint8 gap, bool handover, bytes32 salt)
        internal
        view
        returns (Update memory u, bytes32 decoy)
    {
        u.handover = handover;
        u.setId = handover ? ghostNextId : ghostCurrentId;
        uint64 blockNumber = client.latestBeefyBlock() + 1 + gap % 4;
        bytes32 mmrRoot;
        if (handover) {
            // Hand the bridge to a validator set the attacker made up.
            u.leaf = leafFor(ghostNextId + 1, keccak256(abi.encode("attacker set", salt)));
            mmrRoot = leafHash(u.leaf);
        } else {
            mmrRoot = keccak256(abi.encode("forged", salt));
        }
        setCommitment(u, blockNumber, mmrRoot);

        Update memory honest;
        honest.setId = u.setId;
        setCommitment(honest, blockNumber, keccak256(abi.encode("honest decoy", salt)));
        decoy = honest.commitmentHash;
    }

    function setCommitment(Update memory u, uint64 blockNumber, bytes32 mmrRoot) internal view {
        BeefyClient.PayloadItem[] memory payload = new BeefyClient.PayloadItem[](1);
        payload[0] = BeefyClient.PayloadItem(MMR_ROOT_ID, bytes.concat(mmrRoot));
        u.commitment = BeefyClient.Commitment(uint32(blockNumber), u.setId, payload);
        u.commitmentHash = keccak256(client.encodeCommitment_public(u.commitment));
    }

    function leafFor(uint64 nextId, bytes32 nextRoot)
        internal
        pure
        returns (BeefyClient.MMRLeaf memory)
    {
        return BeefyClient.MMRLeaf(
            0,
            0,
            bytes32(0),
            nextId,
            uint32(sizeOf(nextId)),
            nextRoot,
            keccak256(abi.encode("parachain heads", nextId))
        );
    }

    /// With an empty leaf proof, the commitment's MMR root is the leaf hash.
    function leafHash(BeefyClient.MMRLeaf memory leaf) internal pure returns (bytes32) {
        return keccak256(
            bytes.concat(
                ScaleCodec.encodeU8(leaf.version),
                ScaleCodec.encodeU32(leaf.parentNumber),
                leaf.parentHash,
                ScaleCodec.encodeU64(leaf.nextAuthoritySetID),
                ScaleCodec.encodeU32(leaf.nextAuthoritySetLen),
                leaf.nextAuthoritySetRoot,
                leaf.parachainHeadsRoot
            )
        );
    }

    /// One signature per sampled validator. The attacker signs `u` only with its own keys and
    /// answers the other slots with signatures over `honestSlotsSign`.
    function proofsFor(
        Update memory u,
        uint256[] memory sample,
        bytes32 honestSlotsSign,
        bool attacker
    ) internal view returns (BeefyClient.CompactValidatorProofs memory) {
        ValidatorSetKeys memory k = keysOf(u.setId);
        uint256 n = k.keys.length;
        uint256 f = (n - 1) / 3;
        BeefyClient.ValidatorProof[] memory ps =
            new BeefyClient.ValidatorProof[](Bitfield.countSetBits(sample));
        uint256 j;
        for (uint256 i = 0; i < n; i++) {
            if (!Bitfield.isSet(sample, i)) continue;
            bytes32 signed = attacker && i >= f ? honestSlotsSign : u.commitmentHash;
            (uint8 v, bytes32 r, bytes32 s) = vm.sign(k.keys[i], signed);
            ps[j++] = BeefyClient.ValidatorProof(v, r, s, i, k.signers[i], k.paths[i]);
        }
        return CompactProofLib.toCompact(ps, n);
    }

    function claimAll(uint256 n) internal pure returns (uint256[] memory bitfield) {
        bitfield = new uint256[](Bitfield.containerLength(n));
        for (uint256 i = 0; i < n; i++) {
            Bitfield.set(bitfield, i);
        }
    }

    /// The attacker's keys plus just enough honest validators for a quorum.
    function claimQuorum(uint256 n) internal pure returns (uint256[] memory bitfield) {
        bitfield = new uint256[](Bitfield.containerLength(n));
        for (uint256 i = 0; i < n - (n - 1) / 3; i++) {
            Bitfield.set(bitfield, i);
        }
    }

    function recordAccepted(Update memory u) internal {
        accepted++;
        lastAcceptedBlock = u.commitment.blockNumber;
        lastAcceptedRoot = bytes32(u.commitment.payload[0].data);
        if (u.handover) {
            handovers++;
            ghostCurrentId = ghostNextId;
            ghostNextId = u.leaf.nextAuthoritySetID;
        }
    }

    modifier checksBlock() {
        uint64 before = client.latestBeefyBlock();
        _;
        if (client.latestBeefyBlock() < before) blockWentBack = true;
    }

    // ---- validator sets -------------------------------------------------------------------

    /// 16 to 24 validators, so set sizes change across handovers.
    function sizeOf(uint64 setId) public pure returns (uint256) {
        return 16 + uint256(setId) % 9;
    }

    function keysOf(uint64 setId) public pure returns (ValidatorSetKeys memory k) {
        uint256 n = sizeOf(setId);
        k.keys = new uint256[](n);
        k.signers = new address[](n);
        bytes32[] memory leaves = new bytes32[](n);
        for (uint256 i = 0; i < n; i++) {
            k.keys[i] = uint256(keccak256(abi.encode("validator", setId, i)));
            k.signers[i] = vm.addr(k.keys[i]);
            leaves[i] = keccak256(abi.encodePacked(k.signers[i]));
        }
        (k.root, k.paths) = MerkleLibSubstrate.buildBinaryMerkleTree(leaves);
    }
}

contract BeefyClientInvariantTest is Test {
    BeefyClientHandler handler;
    BeefyClient client;

    function setUp() public {
        handler = new BeefyClientHandler();
        client = handler.client();
        targetContract(address(handler));
        bytes4[] memory selectors = new bytes4[](6);
        selectors[0] = BeefyClientHandler.honestFiatShamir.selector;
        selectors[1] = BeefyClientHandler.honestInteractive.selector;
        selectors[2] = BeefyClientHandler.forgedFiatShamir.selector;
        selectors[3] = BeefyClientHandler.forgedInteractive.selector;
        selectors[4] = BeefyClientHandler.staleHonest.selector;
        selectors[5] = BeefyClientHandler.rollBlocks.selector;
        targetSelector(FuzzSelector(address(handler), selectors));
    }

    /// forge-config: default.invariant.runs = 32
    /// forge-config: default.invariant.depth = 24
    /// forge-config: production.invariant.runs = 32
    /// forge-config: production.invariant.depth = 24
    function invariant_noForgeryIsAccepted() public view {
        assertEq(handler.forgedAccepted(), 0, "a forged or stale update was accepted");
        bytes32 root = client.latestMMRRoot();
        assertTrue(root == bytes32(0) || handler.isHonestRoot(root), "MMR root not honest");
    }

    /// forge-config: default.invariant.runs = 32
    /// forge-config: default.invariant.depth = 24
    /// forge-config: production.invariant.runs = 32
    /// forge-config: production.invariant.depth = 24
    function invariant_everyHonestUpdateIsAccepted() public view {
        assertEq(handler.honestRejected(), 0, "an honest update was rejected");
    }

    /// forge-config: default.invariant.runs = 32
    /// forge-config: default.invariant.depth = 24
    /// forge-config: production.invariant.runs = 32
    /// forge-config: production.invariant.depth = 24
    function invariant_stateIsTheLastAcceptedUpdate() public view {
        assertFalse(handler.blockWentBack(), "latestBeefyBlock went back");
        assertEq(client.latestBeefyBlock(), handler.lastAcceptedBlock(), "latestBeefyBlock");
        assertEq(client.latestMMRRoot(), handler.lastAcceptedRoot(), "latestMMRRoot");
    }

    /// forge-config: default.invariant.runs = 32
    /// forge-config: default.invariant.depth = 24
    /// forge-config: production.invariant.runs = 32
    /// forge-config: production.invariant.depth = 24
    function invariant_validatorSetsFollowHonestHandovers() public view {
        (uint128 currentId, uint128 currentLength, bytes32 currentRoot,) =
            client.currentValidatorSet();
        (uint128 nextId, uint128 nextLength, bytes32 nextRoot,) = client.nextValidatorSet();
        assertEq(currentId, handler.ghostCurrentId(), "current set id");
        assertEq(nextId, handler.ghostNextId(), "next set id");
        assertEq(nextId, currentId + 1, "next set is not current + 1");
        assertEq(currentLength, handler.sizeOf(uint64(currentId)), "current set length");
        assertEq(nextLength, handler.sizeOf(uint64(nextId)), "next set length");
        assertEq(currentRoot, handler.keysOf(uint64(currentId)).root, "current set root");
        assertEq(nextRoot, handler.keysOf(uint64(nextId)).root, "next set root");
    }
}
