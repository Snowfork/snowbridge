// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "openzeppelin/utils/Strings.sol";
import "forge-std/Test.sol";
import "forge-std/console.sol";

import "./mocks/BeefyClientMock.sol";
import "../ScaleCodec.sol";
import "../utils/Bitfield.sol";

interface CheatCodes {
    function prank(address) external;

    function roll(uint256) external;

    function warp(uint256) external;

    function expectRevert(bytes4 message) external;

    function difficulty(uint256) external;
}

contract BeefyClientTest is Test {
    CheatCodes cheats = CheatCodes(HEVM_ADDRESS);
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
    BeefyClient.Payload payload;
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
        inputs[1] = "src/test/scripts/ffiWrapper.ts";
        inputs[2] = "GenerateInitialSet";

        // generate initial fixture data with ffi
        (blockNumber, setId, setSize, bitSetArray, commitHash, payload) =
            abi.decode(vm.ffi(inputs), (uint32, uint32, uint32, uint256[], bytes32, BeefyClient.Payload));
        bitfield = Bitfield.createBitfield(bitSetArray, setSize);

        // To avoid another round of ffi in multiple tests
        // except for the initial merkle root and proof for validators
        // we also precalculate finalValidatorProofs and cached here
        finalBitfield = Bitfield.randomNBitsWithPriorCheck(
            difficulty, bitfield, beefyClient.minimumSignatureThreshold_public(setSize), setSize
        );

        inputs[2] = "GenerateProofs";
        inputs[3] = Strings.toString(finalBitfield.length);
        for (uint256 i = 0; i < finalBitfield.length; i++) {
            inputs[i + 4] = Strings.toString(finalBitfield[i]);
        }
        BeefyClient.ValidatorProof[] memory _proofs;
        (root, validatorProof, _proofs, mmrLeafProofs, mmrLeaf, leafProofOrder) = abi.decode(
            vm.ffi(inputs),
            (bytes32, BeefyClient.ValidatorProof, BeefyClient.ValidatorProof[], bytes32[], BeefyClient.MMRLeaf, uint256)
        );
        // Cache finalValidatorProofs to storage in order to reuse in submitFinal later
        for (uint256 i = 0; i < _proofs.length; i++) {
            finalValidatorProofs.push(_proofs[i]);
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

        beefyClient.submitInitial(commitHash, bitfield, validatorProof);

        // mine random delay blocks
        cheats.roll(block.number + randaoCommitDelay);

        // set difficulty as PrevRandao
        cheats.difficulty(difficulty);

        beefyClient.commitPrevRandao(commitHash);

        BeefyClient.Commitment memory commitment = BeefyClient.Commitment(blockNumber, setId, payload);
        beefyClient.submitFinal(commitment, bitfield, finalValidatorProofs);

        assertEq(beefyClient.latestBeefyBlock(), blockNumber);
    }

    function testSubmitFailWithStaleCommitment() public {
        // first round of submit should be fine
        testSubmit();

        beefyClient.submitInitial(commitHash, bitfield, validatorProof);
        cheats.roll(block.number + randaoCommitDelay);
        cheats.difficulty(difficulty);
        beefyClient.commitPrevRandao(commitHash);
        BeefyClient.Commitment memory commitment = BeefyClient.Commitment(blockNumber, setId, payload);
        //submit again will be reverted with StaleCommitment
        cheats.expectRevert(BeefyClient.StaleCommitment.selector);
        beefyClient.submitFinal(commitment, bitfield, finalValidatorProofs);
    }

    function testSubmitFailWithInvalidBitfield() public {
        initialize(setId);

        beefyClient.submitInitial(commitHash, bitfield, validatorProof);

        cheats.roll(block.number + randaoCommitDelay);

        cheats.difficulty(difficulty);

        beefyClient.commitPrevRandao(commitHash);

        BeefyClient.Commitment memory commitment = BeefyClient.Commitment(blockNumber, setId, payload);
        // invalid bitfield here
        bitfield[0] = 0;
        cheats.expectRevert(BeefyClient.InvalidBitfield.selector);
        beefyClient.submitFinal(commitment, bitfield, finalValidatorProofs);
    }

    function testSubmitFailWithoutPrevRandao() public {
        initialize(setId);
        beefyClient.submitInitial(commitHash, bitfield, validatorProof);
        BeefyClient.Commitment memory commitment = BeefyClient.Commitment(blockNumber, setId, payload);
        // reverted without commit PrevRandao
        cheats.expectRevert(BeefyClient.PrevRandaoNotCaptured.selector);
        beefyClient.submitFinal(commitment, bitfield, finalValidatorProofs);
    }

    function testSubmitFailForPrevRandaoTooEarlyOrTooLate() public {
        initialize(setId);
        beefyClient.submitInitial(commitHash, bitfield, validatorProof);
        // reverted for commit PrevRandao too early
        cheats.expectRevert(BeefyClient.WaitPeriodNotOver.selector);
        beefyClient.commitPrevRandao(commitHash);

        // reverted for commit PrevRandao too late
        cheats.roll(block.number + randaoCommitDelay + randaoCommitExpiration + 1);
        cheats.expectRevert(BeefyClient.TaskExpired.selector);
        beefyClient.commitPrevRandao(commitHash);
    }

    function testSubmitFailWithInvalidSignatureProof() public {
        initialize(setId);

        bytes32 originalSignatureS = validatorProof.s;

        // bad signature in proof
        validatorProof.s = bytes32("Hello");
        cheats.expectRevert(BeefyClient.InvalidSignature.selector);
        beefyClient.submitInitial(commitHash, bitfield, validatorProof);

        // restore with original signature
        validatorProof.s = originalSignatureS;

        // bad account in proof
        validatorProof.account = address(0x0);
        cheats.expectRevert(BeefyClient.InvalidValidatorProof.selector);
        beefyClient.submitInitial(commitHash, bitfield, validatorProof);
    }

    function testSubmitWithHandover() public {
        //initialize with previous set
        initialize(setId - 1);

        beefyClient.submitInitialWithHandover(commitHash, bitfield, validatorProof);

        cheats.roll(block.number + randaoCommitDelay);

        cheats.difficulty(difficulty);

        beefyClient.commitPrevRandao(commitHash);

        BeefyClient.Commitment memory commitment = BeefyClient.Commitment(blockNumber, setId, payload);
        beefyClient.submitFinalWithHandover(
            commitment, bitfield, finalValidatorProofs, mmrLeaf, mmrLeafProofs, leafProofOrder
        );
        assertEq(beefyClient.latestBeefyBlock(), blockNumber);
    }

    function testScaleEncodeCommit() public {
        BeefyClient.Payload memory _payload = BeefyClient.Payload(
            0x3ac49cd24778522203e8bf40a4712ea3f07c3803bbd638cb53ebb3564ec13e8c, hex"0861620c0001026d6880", hex""
        );
        BeefyClient.Commitment memory _commitment = BeefyClient.Commitment(5, 7, _payload);
        bytes memory encoded = beefyClient.encodeCommitment_public(_commitment);
        assertEq(
            encoded,
            hex"0861620c0001026d68803ac49cd24778522203e8bf40a4712ea3f07c3803bbd638cb53ebb3564ec13e8c050000000700000000000000"
        );
    }
}
