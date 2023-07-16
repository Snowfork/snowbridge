// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.20;

import {Strings} from "openzeppelin/utils/Strings.sol";
import {Test} from "forge-std/Test.sol";
import {console} from "forge-std/console.sol";

import {BeefyClient} from "../src/BeefyClient.sol";
import {BeefyClientMock} from "./mocks/BeefyClientMock.sol";
import {ScaleCodec} from "../src/ScaleCodec.sol";
import {Bitfield} from "../src/utils/Bitfield.sol";

contract BeefyClientTest is Test {
    BeefyClientMock beefyClient;
    uint8 randaoCommitDelay;
    uint8 randaoCommitExpiration;
    uint32 blockNumber;
    uint32 difficulty;
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

    function setUp() public {
        randaoCommitDelay = 3;
        randaoCommitExpiration = 8;
        difficulty = 377;

        beefyClient = new BeefyClientMock(randaoCommitDelay, randaoCommitExpiration);

        // Allocate for input variables
        string[] memory inputs = new string[](10);
        inputs[0] = "node_modules/.bin/ts-node";
        inputs[1] = "scripts/ffiWrapper.ts";
        inputs[2] = "GenerateInitialSet";

        // generate initial fixture data with ffi
        (blockNumber, setId, setSize, bitSetArray, absentBitSetArray, commitHash, mmrRoot) =
            abi.decode(vm.ffi(inputs), (uint32, uint32, uint32, uint256[], uint256[], bytes32, bytes32));
        bitfield = beefyClient.createInitialBitfield(bitSetArray, setSize);
        absentBitfield = beefyClient.createInitialBitfield(absentBitSetArray, setSize);

        // To avoid another round of ffi in multiple tests
        // except for the initial merkle root and proof for validators
        // we also precalculate finalValidatorProofs and cached here
        finalBitfield =
            Bitfield.subsample(difficulty, bitfield, beefyClient.minimumSignatureThreshold_public(setSize), setSize);

        inputs[2] = "GenerateProofs";
        inputs[3] = Strings.toString(finalBitfield.length);
        for (uint256 i = 0; i < finalBitfield.length; i++) {
            inputs[i + 4] = Strings.toString(finalBitfield[i]);
        }
        BeefyClient.ValidatorProof[] memory proofs;
        (root, proofs, mmrLeafProofs, mmrLeaf, leafProofOrder) =
            abi.decode(vm.ffi(inputs), (bytes32, BeefyClient.ValidatorProof[], bytes32[], BeefyClient.MMRLeaf, uint256));
        // Cache finalValidatorProofs to storage in order to reuse in submitFinal later
        for (uint256 i = 0; i < proofs.length; i++) {
            finalValidatorProofs.push(proofs[i]);
        }
        console.log("current validator's merkle root is: %s", Strings.toHexString(uint256(root), 32));
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
            console.log("bits index at %d is %x", i, bits[i]);
        }
    }

    function testSubmit() public returns (BeefyClient.Commitment memory) {
        BeefyClient.Commitment memory commitment = initialize(setId);

        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[0]);

        // mine random delay blocks
        vm.roll(block.number + randaoCommitDelay);

        // set difficulty as PrevRandao
        vm.prevrandao(bytes32(uint256(difficulty)));

        beefyClient.commitPrevRandao(commitHash);

        beefyClient.submitFinal(commitment, bitfield, finalValidatorProofs);

        assertEq(beefyClient.latestBeefyBlock(), blockNumber);
        return commitment;
    }

    function testSubmitFailInvalidSignature() public {
        BeefyClient.Commitment memory commitment = initialize(setId);

        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[0]);

        // mine random delay blocks
        vm.roll(block.number + randaoCommitDelay);

        // set difficulty as PrevRandao
        vm.prevrandao(bytes32(uint256(difficulty)));

        beefyClient.commitPrevRandao(commitHash);

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

        // set difficulty as PrevRandao
        vm.prevrandao(bytes32(uint256(difficulty)));

        beefyClient.commitPrevRandao(commitHash);

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
        vm.prevrandao(bytes32(uint256(difficulty)));
        beefyClient.commitPrevRandao(commitHash);
        //submit again will be reverted with StaleCommitment
        vm.expectRevert(BeefyClient.StaleCommitment.selector);
        beefyClient.submitFinal(commitment, bitfield, finalValidatorProofs);
    }

    function testSubmitFailWithInvalidBitfield() public {
        BeefyClient.Commitment memory commitment = initialize(setId);

        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[0]);

        vm.roll(block.number + randaoCommitDelay);

        vm.prevrandao(bytes32(uint256(difficulty)));

        beefyClient.commitPrevRandao(commitHash);

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
        beefyClient.commitPrevRandao(commitHash);

        // reverted for commit PrevRandao too late
        vm.roll(block.number + randaoCommitDelay + randaoCommitExpiration + 1);
        vm.expectRevert(BeefyClient.TicketExpired.selector);
        beefyClient.commitPrevRandao(commitHash);
    }

    function testSubmitFailForPrevRandaoCapturedMoreThanOnce() public {
        BeefyClient.Commitment memory commitment = initialize(setId);

        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[0]);
        vm.roll(block.number + randaoCommitDelay);
        vm.prevrandao(bytes32(uint256(difficulty)));
        beefyClient.commitPrevRandao(commitHash);

        vm.expectRevert(BeefyClient.PrevRandaoAlreadyCaptured.selector);
        beefyClient.commitPrevRandao(commitHash);
    }

    function testSubmitWithHandover() public {
        //initialize with previous set
        BeefyClient.Commitment memory commitment = initialize(setId - 1);

        beefyClient.submitInitialWithHandover(commitment, bitfield, finalValidatorProofs[0]);

        vm.roll(block.number + randaoCommitDelay);

        vm.prevrandao(bytes32(uint256(difficulty)));

        beefyClient.commitPrevRandao(commitHash);

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
        BeefyClient.Commitment memory commitment = testSubmit();

        beefyClient.submitInitialWithHandover(commitment, bitfield, finalValidatorProofs[0]);

        vm.roll(block.number + randaoCommitDelay);

        vm.prevrandao(bytes32(uint256(difficulty)));

        beefyClient.commitPrevRandao(commitHash);

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
        assertTrue(initialBitfield.length == 2);
        printBitArray(initialBitfield);
        assertEq(initialBitfield[0], 0xd9fbb69bb8dfe46bffd2fd7feefffb185aef39fafcec0beba6db619efad1f6db);
        assertEq(initialBitfield[1], 0x7f76cee2a3f);
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
        vm.prevrandao(bytes32(uint256(difficulty)));
        beefyClient.commitPrevRandao(commitHash);

        uint256[] memory finalBits = beefyClient.createFinalBitfield(commitHash, bitfield);
        assertTrue(Bitfield.countSetBits(finalBits) < Bitfield.countSetBits(bitfield));
    }

    function testCreateFinalBitfieldInvalid() public {
        BeefyClient.Commitment memory commitment = initialize(setId);
        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[0]);
        vm.roll(block.number + randaoCommitDelay);
        vm.prevrandao(bytes32(uint256(difficulty)));
        beefyClient.commitPrevRandao(commitHash);

        // make invalid bitfield not same as initialized
        bitfield[0] = 0;
        vm.expectRevert(BeefyClient.InvalidBitfield.selector);
        beefyClient.createFinalBitfield(commitHash, bitfield);
    }

    function testSubmitFailWithInvalidValidatorSet() public {
        BeefyClient.Commitment memory commitment = initialize(setId);
        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[0]);
        vm.roll(block.number + randaoCommitDelay);
        vm.prevrandao(bytes32(uint256(difficulty)));
        beefyClient.commitPrevRandao(commitHash);

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

        vm.prevrandao(bytes32(uint256(difficulty)));

        beefyClient.commitPrevRandao(commitHash);

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
        vm.prevrandao(bytes32(uint256(difficulty)));
        beefyClient.commitPrevRandao(commitHash);

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

        vm.prevrandao(bytes32(uint256(difficulty)));

        beefyClient.commitPrevRandao(commitHash);

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

        vm.prevrandao(bytes32(uint256(difficulty)));

        beefyClient.commitPrevRandao(commitHash);

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
}
