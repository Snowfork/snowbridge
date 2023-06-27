// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

import "openzeppelin/utils/Strings.sol";
import "forge-std/Test.sol";
import "forge-std/console.sol";

import "./mocks/BeefyClientMock.sol";
import "../src/ScaleCodec.sol";
import "../src/utils/Bitfield.sol";

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
    uint256[] bitfield;
    BeefyClient.PayloadItem[] payload;
    uint256[] finalBitfield;
    BeefyClient.ValidatorProof validatorProof;
    BeefyClient.ValidatorProof[] finalValidatorProofs;
    bytes32[] mmrLeafProofs;
    BeefyClient.MMRLeaf mmrLeaf;
    uint256 leafProofOrder;

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
        (blockNumber, setId, setSize, bitSetArray, commitHash, payload) =
            abi.decode(vm.ffi(inputs), (uint32, uint32, uint32, uint256[], bytes32, BeefyClient.PayloadItem[]));
        bitfield = Bitfield.createBitfield(bitSetArray, setSize);

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
        (root, proofs, mmrLeafProofs, mmrLeaf, leafProofOrder) = abi.decode(
            vm.ffi(inputs),
            (bytes32, BeefyClient.ValidatorProof[], bytes32[], BeefyClient.MMRLeaf, uint256)
        );
        // Cache finalValidatorProofs to storage in order to reuse in submitFinal later
        for (uint256 i = 0; i < proofs.length; i++) {
            finalValidatorProofs.push(proofs[i]);
        }
        console.log("current validator's merkle root is: %s", Strings.toHexString(uint256(root), 32));
    }

    function initialize(uint32 _setId) public {
        currentSetId = _setId;
        nextSetId = _setId + 1;
        BeefyClient.ValidatorSet memory vset = BeefyClient.ValidatorSet(currentSetId, setSize, root);
        BeefyClient.ValidatorSet memory nextvset = BeefyClient.ValidatorSet(nextSetId, setSize, root);
        beefyClient.initialize(0, vset, nextvset);
    }

    function testSubmit() public {
        initialize(setId);

        BeefyClient.Commitment memory commitment = BeefyClient.Commitment(blockNumber, setId, payload);
        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[0]);

        // mine random delay blocks
        vm.roll(block.number + randaoCommitDelay);

        // set difficulty as PrevRandao
        vm.prevrandao(bytes32(uint256(difficulty)));

        beefyClient.commitPrevRandao(commitHash);

        beefyClient.submitFinal(commitment, bitfield, finalValidatorProofs);

        assertEq(beefyClient.latestBeefyBlock(), blockNumber);
    }

    function testSubmitFailInvalidSignature() public {
        initialize(setId);

        BeefyClient.Commitment memory commitment = BeefyClient.Commitment(blockNumber, setId, payload);
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
        initialize(setId);

        BeefyClient.Commitment memory commitment = BeefyClient.Commitment(blockNumber, setId, payload);
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
        testSubmit();

        BeefyClient.Commitment memory commitment = BeefyClient.Commitment(blockNumber, setId, payload);
        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[0]);
        vm.roll(block.number + randaoCommitDelay);
        vm.prevrandao(bytes32(uint256(difficulty)));
        beefyClient.commitPrevRandao(commitHash);
        //submit again will be reverted with StaleCommitment
        vm.expectRevert(BeefyClient.StaleCommitment.selector);
        beefyClient.submitFinal(commitment, bitfield, finalValidatorProofs);
    }

    function testSubmitFailWithInvalidBitfield() public {
        initialize(setId);

        BeefyClient.Commitment memory commitment = BeefyClient.Commitment(blockNumber, setId, payload);
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
        initialize(setId);
        BeefyClient.Commitment memory commitment = BeefyClient.Commitment(blockNumber, setId, payload);
        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[0]);
        // reverted without commit PrevRandao
        vm.expectRevert(BeefyClient.PrevRandaoNotCaptured.selector);
        beefyClient.submitFinal(commitment, bitfield, finalValidatorProofs);
    }

    function testSubmitFailForPrevRandaoTooEarlyOrTooLate() public {
        initialize(setId);
        BeefyClient.Commitment memory commitment = BeefyClient.Commitment(blockNumber, setId, payload);
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
        initialize(setId);
        BeefyClient.Commitment memory commitment = BeefyClient.Commitment(blockNumber, setId, payload);
        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[0]);
        vm.roll(block.number + randaoCommitDelay);
        vm.prevrandao(bytes32(uint256(difficulty)));
        beefyClient.commitPrevRandao(commitHash);

        vm.expectRevert(BeefyClient.PrevRandaoAlreadyCaptured.selector);
        beefyClient.commitPrevRandao(commitHash);
    }

    function testSubmitWithHandover() public {
        //initialize with previous set
        initialize(setId - 1);

        BeefyClient.Commitment memory commitment = BeefyClient.Commitment(blockNumber, setId, payload);
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
        initialize(setId - 1);

        BeefyClient.Commitment memory commitment = BeefyClient.Commitment(blockNumber, setId, payload);
        beefyClient.submitInitialWithHandover(commitment, bitfield, finalValidatorProofs[0]);

        vm.expectRevert(BeefyClient.PrevRandaoNotCaptured.selector);
        beefyClient.submitFinalWithHandover(
            commitment, bitfield, finalValidatorProofs, mmrLeaf, mmrLeafProofs, leafProofOrder
        );
    }

    function testSubmitWithHandoverFailStaleCommitment() public {
        testSubmit();

        BeefyClient.Commitment memory commitment = BeefyClient.Commitment(blockNumber, setId, payload);
        beefyClient.submitInitialWithHandover(commitment, bitfield, finalValidatorProofs[0]);

        vm.roll(block.number + randaoCommitDelay);

        vm.prevrandao(bytes32(uint256(difficulty)));

        beefyClient.commitPrevRandao(commitHash);

        vm.expectRevert(BeefyClient.InvalidCommitment.selector);
        beefyClient.submitFinalWithHandover(
            commitment, bitfield, finalValidatorProofs, mmrLeaf, mmrLeafProofs, leafProofOrder
        );
    }

    function testScaleEncodeCommit() public {
        BeefyClient.PayloadItem[] memory _payload = new BeefyClient.PayloadItem[](2);
        _payload[0] = BeefyClient.PayloadItem(bytes2(0x6162), hex"000102");
        _payload[1] = BeefyClient.PayloadItem(bytes2(0x6d68), hex"3ac49cd24778522203e8bf40a4712ea3f07c3803bbd638cb53ebb3564ec13e8c");

        BeefyClient.Commitment memory _commitment = BeefyClient.Commitment(5, 7, _payload);

        bytes memory encoded = beefyClient.encodeCommitment_public(_commitment);

        assertEq(
            encoded,
            hex"0861620c0001026d68803ac49cd24778522203e8bf40a4712ea3f07c3803bbd638cb53ebb3564ec13e8c050000000700000000000000"
        );
    }
}
