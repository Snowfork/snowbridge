// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.20;

import {Strings} from "openzeppelin/utils/Strings.sol";
import {Test} from "forge-std/Test.sol";
import {console} from "forge-std/console.sol";
import {stdJson} from "forge-std/StdJson.sol";

import {BeefyClient} from "../src/BeefyClient.sol";
import {BeefyClientMock} from "./mocks/BeefyClientMock.sol";
import {ScaleCodec} from "../src/utils/ScaleCodec.sol";
import {Bitfield} from "../src/utils/Bitfield.sol";

contract BeefyClientTest is Test {
    using stdJson for string;

    BeefyClientMock beefyClient;
    uint8 randaoCommitDelay;
    uint8 randaoCommitExpiration;
    uint32 blockNumber;
    uint32 prevRandao;
    uint32 setSize;
    uint32 setId;
    uint128 currentSetId;
    uint128 nextSetId;
    bytes32 commitHash;
    bytes32 root;
    uint256[] bitSetArray;
    uint256[] absentBitSetArray;
    uint256[] bitfield;
    uint256[] absentBitfield;
    bytes32 mmrRoot;
    uint256[] finalBitfield;
    BeefyClient.ValidatorProof validatorProof;
    BeefyClient.ValidatorProof[] finalValidatorProofs;
    bytes32[] mmrLeafProofs;
    BeefyClient.MMRLeaf mmrLeaf;
    uint256 leafProofOrder;
    bytes2 mmrRootID = bytes2("mh");
    string beefyCommitmentFile;
    string beefyCommitmentRaw;
    string beefyValidatorSetFile;
    string beefyValidatorSetRaw;
    string beefyValidatorProofFile;
    string beefyValidatorProofRaw;
    string beefyFinalBitFieldFile;

    function setUp() public {
        randaoCommitDelay = uint8(vm.envOr("RANDAO_COMMIT_DELAY", uint256(3)));
        randaoCommitExpiration = uint8(vm.envOr("RANDAO_COMMIT_EXP", uint256(8)));
        prevRandao = uint32(vm.envOr("PREV_RANDAO", uint256(377)));

        beefyClient = new BeefyClientMock(randaoCommitDelay, randaoCommitExpiration);

        beefyCommitmentFile = string.concat(vm.projectRoot(), "/test/data/beefy-commitment.json");
        beefyCommitmentRaw = vm.readFile(beefyCommitmentFile);

        beefyValidatorSetFile = string.concat(vm.projectRoot(), "/test/data/beefy-validator-set.json");
        beefyValidatorSetRaw = vm.readFile(beefyValidatorSetFile);

        beefyValidatorProofFile = string.concat(vm.projectRoot(), "/test/data/beefy-final-proof.json");
        beefyValidatorProofRaw = vm.readFile(beefyValidatorProofFile);

        beefyFinalBitFieldFile = string.concat(vm.projectRoot(), "/test/data/beefy-final-bitfield.json");

        blockNumber = uint32(beefyCommitmentRaw.readUint(".params.commitment.blockNumber"));
        setId = uint32(beefyCommitmentRaw.readUint(".params.commitment.validatorSetID"));
        commitHash = beefyCommitmentRaw.readBytes32(".commitmentHash");

        setSize = uint32(beefyValidatorSetRaw.readUint(".validatorSetSize"));
        root = beefyValidatorSetRaw.readBytes32(".validatorRoot");
        bitSetArray = beefyValidatorSetRaw.readUintArray(".participants");
        absentBitSetArray = beefyValidatorSetRaw.readUintArray(".absentees");

        console.log("current validator's merkle root is: %s", Strings.toHexString(uint256(root), 32));

        mmrRoot = beefyCommitmentRaw.readBytes32(".params.commitment.payload[0].data");
        mmrLeafProofs = beefyCommitmentRaw.readBytes32Array(".params.leafProof");
        leafProofOrder = beefyCommitmentRaw.readUint(".params.leafProofOrder");
        decodeMMRLeaf();

        bitfield = beefyClient.createInitialBitfield(bitSetArray, setSize);
        absentBitfield = beefyClient.createInitialBitfield(absentBitSetArray, setSize);

        loadFinalProofs();
    }

    function initialize(uint32 _setId) public returns (BeefyClient.Commitment memory) {
        currentSetId = _setId;
        nextSetId = _setId + 1;
        BeefyClient.ValidatorSet memory vset = BeefyClient.ValidatorSet(currentSetId, setSize, root);
        BeefyClient.ValidatorSet memory nextvset = BeefyClient.ValidatorSet(nextSetId, setSize, root);
        beefyClient.initialize_public(0, vset, nextvset);
        BeefyClient.PayloadItem[] memory payload = new BeefyClient.PayloadItem[](1);
        payload[0] = BeefyClient.PayloadItem(mmrRootID, abi.encodePacked(mmrRoot));
        return BeefyClient.Commitment(blockNumber, setId, payload);
    }

    function printBitArray(uint256[] memory bits) private view {
        for (uint256 i = 0; i < bits.length; i++) {
            console.log("bits index at %d is %d", i, bits[i]);
        }
    }

    function loadFinalProofs() internal {
        bytes memory proofRaw = beefyValidatorProofRaw.readBytes(".finalValidatorsProofRaw");
        BeefyClient.ValidatorProof[] memory proofs = abi.decode(proofRaw, (BeefyClient.ValidatorProof[]));
        for (uint256 i = 0; i < proofs.length; i++) {
            finalValidatorProofs.push(proofs[i]);
        }
    }

    // Ideally should also update `finalValidatorProofs` with another round of ffi based on the `finalBitfield` here
    // For simplicity we just use the proof previously cached
    // still update `finalBitfield` here is to simulate more close to the real workflow and make gas estimation more accurate
    function createFinalProofs() internal {
        finalBitfield = beefyClient.createFinalBitfield(commitHash, bitfield);
    }

    function commitPrevRandao() internal {
        vm.prevrandao(bytes32(uint256(prevRandao)));
        beefyClient.commitPrevRandao(commitHash);
    }

    // Regenerate bitField file
    function regenerateBitField() internal {
        console.log("print initialBitField");
        printBitArray(bitfield);
        prevRandao = uint32(vm.envOr("PREV_RANDAO", prevRandao));
        finalBitfield =
            Bitfield.subsample(prevRandao, bitfield, beefyClient.minimumSignatureThreshold_public(setSize), setSize);
        console.log("print finalBitField");
        printBitArray(finalBitfield);

        string memory finalBitFieldRaw = "";
        finalBitFieldRaw = finalBitFieldRaw.serialize("finalBitFieldRaw", abi.encode(finalBitfield));

        string memory finaliBitFieldStr = "";
        finaliBitFieldStr = finaliBitFieldStr.serialize("finalBitField", finalBitfield);

        string memory output = finalBitFieldRaw.serialize("final", finaliBitFieldStr);

        vm.writeJson(output, beefyFinalBitFieldFile);
    }

    function decodeMMRLeaf() internal {
        uint8 version = uint8(beefyCommitmentRaw.readUint(".params.leaf.version"));
        uint32 parentNumber = uint32(beefyCommitmentRaw.readUint(".params.leaf.parentNumber"));
        bytes32 parentHash = beefyCommitmentRaw.readBytes32(".params.leaf.parentHash");
        uint64 nextAuthoritySetID = uint64(beefyCommitmentRaw.readUint(".params.leaf.nextAuthoritySetID"));
        uint32 nextAuthoritySetLen = uint32(beefyCommitmentRaw.readUint(".params.leaf.nextAuthoritySetLen"));
        bytes32 nextAuthoritySetRoot = beefyCommitmentRaw.readBytes32(".params.leaf.nextAuthoritySetRoot");
        bytes32 parachainHeadsRoot = beefyCommitmentRaw.readBytes32(".params.leaf.parachainHeadsRoot");
        mmrLeaf = BeefyClient.MMRLeaf(
            version,
            parentNumber,
            parentHash,
            nextAuthoritySetID,
            nextAuthoritySetLen,
            nextAuthoritySetRoot,
            parachainHeadsRoot
        );
    }

    function testSubmit() public returns (BeefyClient.Commitment memory) {
        BeefyClient.Commitment memory commitment = initialize(setId);

        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[0]);

        // mine random delay blocks
        vm.roll(block.number + randaoCommitDelay);

        commitPrevRandao();

        createFinalProofs();

        beefyClient.submitFinal(commitment, bitfield, finalValidatorProofs);

        assertEq(beefyClient.latestBeefyBlock(), blockNumber);
        return commitment;
    }

    function testSubmitFailInvalidSignature() public {
        BeefyClient.Commitment memory commitment = initialize(setId);

        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[0]);

        // mine random delay blocks
        vm.roll(block.number + randaoCommitDelay);

        commitPrevRandao();

        createFinalProofs();

        // make an invalid signature
        finalValidatorProofs[0].r = 0xb5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c;
        vm.expectRevert(BeefyClient.InvalidSignature.selector);
        beefyClient.submitFinal(commitment, bitfield, finalValidatorProofs);
    }

    function testSubmitFailValidatorNotInBitfield() public {
        BeefyClient.Commitment memory commitment = initialize(setId);

        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[0]);

        // mine random delay blocks
        vm.roll(block.number + randaoCommitDelay);

        commitPrevRandao();

        createFinalProofs();

        // make an invalid validator index
        finalValidatorProofs[0].index = 0;
        vm.expectRevert(BeefyClient.InvalidValidatorProof.selector);
        beefyClient.submitFinal(commitment, bitfield, finalValidatorProofs);
    }

    function testSubmitFailWithStaleCommitment() public {
        // first round of submit should be fine
        BeefyClient.Commitment memory commitment = testSubmit();

        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[0]);
        vm.roll(block.number + randaoCommitDelay);

        commitPrevRandao();

        createFinalProofs();

        //submit again will be reverted with StaleCommitment
        vm.expectRevert(BeefyClient.StaleCommitment.selector);
        beefyClient.submitFinal(commitment, bitfield, finalValidatorProofs);
    }

    function testSubmitFailWithInvalidBitfield() public {
        BeefyClient.Commitment memory commitment = initialize(setId);

        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[0]);

        vm.roll(block.number + randaoCommitDelay);

        commitPrevRandao();

        createFinalProofs();

        // invalid bitfield here
        bitfield[0] = 0;
        vm.expectRevert(BeefyClient.InvalidBitfield.selector);
        beefyClient.submitFinal(commitment, bitfield, finalValidatorProofs);
    }

    function testSubmitFailWithoutPrevRandao() public {
        BeefyClient.Commitment memory commitment = initialize(setId);

        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[0]);

        // reverted without commit PrevRandao
        vm.expectRevert(BeefyClient.PrevRandaoNotCaptured.selector);
        beefyClient.submitFinal(commitment, bitfield, finalValidatorProofs);
    }

    function testSubmitFailForPrevRandaoTooEarlyOrTooLate() public {
        BeefyClient.Commitment memory commitment = initialize(setId);

        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[0]);
        // reverted for commit PrevRandao too early
        vm.expectRevert(BeefyClient.WaitPeriodNotOver.selector);
        commitPrevRandao();

        // reverted for commit PrevRandao too late
        vm.roll(block.number + randaoCommitDelay + randaoCommitExpiration + 1);
        vm.expectRevert(BeefyClient.TicketExpired.selector);
        commitPrevRandao();
    }

    function testSubmitFailForPrevRandaoCapturedMoreThanOnce() public {
        BeefyClient.Commitment memory commitment = initialize(setId);

        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[0]);
        vm.roll(block.number + randaoCommitDelay);
        commitPrevRandao();

        vm.expectRevert(BeefyClient.PrevRandaoAlreadyCaptured.selector);
        commitPrevRandao();
    }

    function testSubmitWithHandover() public {
        //initialize with previous set
        BeefyClient.Commitment memory commitment = initialize(setId - 1);

        beefyClient.submitInitialWithHandover(commitment, bitfield, finalValidatorProofs[0]);

        vm.roll(block.number + randaoCommitDelay);

        commitPrevRandao();

        createFinalProofs();

        beefyClient.submitFinalWithHandover(
            commitment, bitfield, finalValidatorProofs, mmrLeaf, mmrLeafProofs, leafProofOrder
        );
        assertEq(beefyClient.latestBeefyBlock(), blockNumber);
    }

    function testSubmitWithHandoverFailWithoutPrevRandao() public {
        //initialize with previous set
        BeefyClient.Commitment memory commitment = initialize(setId - 1);

        beefyClient.submitInitialWithHandover(commitment, bitfield, finalValidatorProofs[0]);

        vm.expectRevert(BeefyClient.PrevRandaoNotCaptured.selector);
        beefyClient.submitFinalWithHandover(
            commitment, bitfield, finalValidatorProofs, mmrLeaf, mmrLeafProofs, leafProofOrder
        );
    }

    function testSubmitWithHandoverFailStaleCommitment() public {
        BeefyClient.Commitment memory commitment = initialize(setId - 1);
        beefyClient.setLatestBeefyBlock(blockNumber);

        beefyClient.submitInitialWithHandover(commitment, bitfield, finalValidatorProofs[0]);

        vm.roll(block.number + randaoCommitDelay);

        commitPrevRandao();

        createFinalProofs();

        vm.expectRevert(BeefyClient.StaleCommitment.selector);
        beefyClient.submitFinalWithHandover(
            commitment, bitfield, finalValidatorProofs, mmrLeaf, mmrLeafProofs, leafProofOrder
        );
    }

    function testScaleEncodeCommit() public {
        BeefyClient.PayloadItem[] memory _payload = new BeefyClient.PayloadItem[](2);
        _payload[0] = BeefyClient.PayloadItem(bytes2("ab"), hex"000102");
        _payload[1] =
            BeefyClient.PayloadItem(mmrRootID, hex"3ac49cd24778522203e8bf40a4712ea3f07c3803bbd638cb53ebb3564ec13e8c");

        BeefyClient.Commitment memory _commitment = BeefyClient.Commitment(5, 7, _payload);

        bytes memory encoded = beefyClient.encodeCommitment_public(_commitment);

        assertEq(
            encoded,
            hex"0861620c0001026d68803ac49cd24778522203e8bf40a4712ea3f07c3803bbd638cb53ebb3564ec13e8c050000000700000000000000"
        );
    }

    function testCreateInitialBitfield() public {
        initialize(setId);
        uint256[] memory initialBitfield = beefyClient.createInitialBitfield(bitSetArray, setSize);
        assertTrue(initialBitfield.length == (setSize + 255) / 256);
        printBitArray(initialBitfield);
    }

    function testCreateInitialBitfieldInvalid() public {
        initialize(setId);
        vm.expectRevert(BeefyClient.InvalidBitfieldLength.selector);
        beefyClient.createInitialBitfield(bitSetArray, bitSetArray.length - 1);
    }

    function testCreateFinalBitfield() public {
        BeefyClient.Commitment memory commitment = initialize(setId);
        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[0]);
        vm.roll(block.number + randaoCommitDelay);
        commitPrevRandao();

        uint256[] memory finalBits = beefyClient.createFinalBitfield(commitHash, bitfield);
        assertTrue(Bitfield.countSetBits(finalBits) < Bitfield.countSetBits(bitfield));
    }

    function testCreateFinalBitfieldInvalid() public {
        BeefyClient.Commitment memory commitment = initialize(setId);
        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[0]);
        vm.roll(block.number + randaoCommitDelay);
        commitPrevRandao();

        // make invalid bitfield not same as initialized
        bitfield[0] = 0;
        vm.expectRevert(BeefyClient.InvalidBitfield.selector);
        beefyClient.createFinalBitfield(commitHash, bitfield);
    }

    function testSubmitFailWithInvalidValidatorSet() public {
        BeefyClient.Commitment memory commitment = initialize(setId);
        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[0]);

        vm.roll(block.number + randaoCommitDelay);
        commitPrevRandao();

        createFinalProofs();

        //reinitialize with next validator set
        initialize(setId + 1);
        //submit will be reverted with InvalidCommitment
        vm.expectRevert(BeefyClient.InvalidCommitment.selector);
        beefyClient.submitFinal(commitment, bitfield, finalValidatorProofs);
    }

    function testSubmitWithHandoverFailWithInvalidValidatorSet() public {
        //initialize with previous set
        BeefyClient.Commitment memory commitment = initialize(setId - 1);

        beefyClient.submitInitialWithHandover(commitment, bitfield, finalValidatorProofs[0]);

        vm.roll(block.number + randaoCommitDelay);
        commitPrevRandao();

        createFinalProofs();

        //reinitialize with next validator set
        initialize(setId);
        //submit will be reverted with InvalidCommitment
        vm.expectRevert(BeefyClient.InvalidCommitment.selector);
        beefyClient.submitFinalWithHandover(
            commitment, bitfield, finalValidatorProofs, mmrLeaf, mmrLeafProofs, leafProofOrder
        );
    }

    function testSubmitFailWithInvalidTicket() public {
        BeefyClient.Commitment memory commitment = initialize(setId);
        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[0]);

        vm.roll(block.number + randaoCommitDelay);
        commitPrevRandao();

        createFinalProofs();

        // Changing the commitment changes its hash, so the ticket can't be found.
        // A zero value ticket is returned in this case, because submitInitial hasn't run for this commitment.
        BeefyClient.Commitment memory _commitment = BeefyClient.Commitment(blockNumber, setId + 1, commitment.payload);
        //submit will be reverted with InvalidTicket
        vm.expectRevert(BeefyClient.InvalidTicket.selector);
        beefyClient.submitFinal(_commitment, bitfield, finalValidatorProofs);
    }

    function testSubmitFailWithInvalidMMRLeaf() public {
        //initialize with previous set
        BeefyClient.Commitment memory commitment = initialize(setId - 1);

        beefyClient.submitInitialWithHandover(commitment, bitfield, finalValidatorProofs[0]);

        vm.roll(block.number + randaoCommitDelay);

        vm.prevrandao(bytes32(uint256(prevRandao)));

        beefyClient.commitPrevRandao(commitHash);

        createFinalProofs();

        //construct nextAuthoritySetID with a wrong value
        mmrLeaf.nextAuthoritySetID = setId;
        //submit will be reverted with InvalidCommitment
        vm.expectRevert(BeefyClient.InvalidMMRLeaf.selector);
        beefyClient.submitFinalWithHandover(
            commitment, bitfield, finalValidatorProofs, mmrLeaf, mmrLeafProofs, leafProofOrder
        );
    }

    function testSubmitFailWithInvalidMMRLeafProof() public {
        //initialize with previous set
        BeefyClient.Commitment memory commitment = initialize(setId - 1);

        beefyClient.submitInitialWithHandover(commitment, bitfield, finalValidatorProofs[0]);

        vm.roll(block.number + randaoCommitDelay);

        commitPrevRandao();

        createFinalProofs();

        //construct parentNumber with a wrong value
        mmrLeaf.parentNumber = 1;
        //submit will be reverted with InvalidCommitment
        vm.expectRevert(BeefyClient.InvalidMMRLeafProof.selector);
        beefyClient.submitFinalWithHandover(
            commitment, bitfield, finalValidatorProofs, mmrLeaf, mmrLeafProofs, leafProofOrder
        );
    }

    function testSubmitFailWithNotEnoughClaims() public {
        BeefyClient.Commitment memory commitment = initialize(setId);
        uint256[] memory initialBits = absentBitfield;
        Bitfield.set(initialBits, finalValidatorProofs[0].index);
        printBitArray(initialBits);
        vm.expectRevert(BeefyClient.NotEnoughClaims.selector);
        beefyClient.submitInitial(commitment, initialBits, finalValidatorProofs[0]);
    }

    function testSubmitInitialFailWithIncorrectValidatorSet() public {
        BeefyClient.Commitment memory commitment = initialize(setId);

        vm.expectRevert(BeefyClient.InvalidCommitment.selector);
        beefyClient.submitInitialWithHandover(commitment, bitfield, finalValidatorProofs[0]);
    }

    function testSubmitInitialHandoverFailWithIncorrectValidatorSet() public {
        BeefyClient.Commitment memory commitment = initialize(setId - 1);

        vm.expectRevert(BeefyClient.InvalidCommitment.selector);
        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[0]);
    }

    function testRegenerateBitField() public {
        regenerateBitField();
    }
}
