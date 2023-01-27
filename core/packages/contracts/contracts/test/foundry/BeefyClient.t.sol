pragma solidity ^0.8.9;

import "../../BeefyClient.sol";
import "../../ScaleCodec.sol";
import "../../utils/Bitfield.sol";
import "@openzeppelin/contracts/utils/Strings.sol";
import "forge-std/Test.sol";
import "forge-std/console.sol";

interface CheatCodes {
    function prank(address) external;

    function roll(uint256) external;

    function warp(uint256) external;

    function expectRevert(bytes4 message) external;

    function difficulty(uint256) external;
}

contract BeefyClientTest is Test {
    CheatCodes cheats = CheatCodes(HEVM_ADDRESS);
    BeefyClient beefyClient;
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

        beefyClient = new BeefyClient(randaoCommitDelay, randaoCommitExpiration);

        // Allocate for input variables
        string[] memory inputs = new string[](10);
        inputs[0] = "test/ffiWrapper.ts";
        // Always add type of command as first arguments
        inputs[1] = "GenerateInitialSet";
        // generate initial fixture data with ffi
        (blockNumber, setId, setSize, bitSetArray, commitHash, payload) = abi.decode(
            vm.ffi(inputs),
            (uint32, uint32, uint32, uint256[], bytes32, BeefyClient.Payload)
        );
        bitfield = Bitfield.createBitfield(bitSetArray, setSize);

        // To avoid another round of ffi in multiple tests
        // except for the initial merkle root and proof for validators
        // we also precalculate finalValidatorProofs and cached here
        finalBitfield = Bitfield.randomNBitsWithPriorCheck(
            difficulty,
            bitfield,
            minimumSignatureThreshold(setSize),
            setSize
        );

        inputs[1] = "GenerateProofs";
        // First add length of finalBitField and then each item as input
        inputs[2] = Strings.toString(finalBitfield.length);
        for (uint256 i = 0; i < finalBitfield.length; i++) {
            inputs[i + 3] = Strings.toString(finalBitfield[i]);
        }
        BeefyClient.ValidatorProof[] memory _proofs;
        (root, validatorProof, _proofs, mmrLeafProofs, mmrLeaf, leafProofOrder) = abi.decode(
            vm.ffi(inputs),
            (
                bytes32,
                BeefyClient.ValidatorProof,
                BeefyClient.ValidatorProof[],
                bytes32[],
                BeefyClient.MMRLeaf,
                uint256
            )
        );
        // Cache finalValidatorProofs to storage in order to reuse in submitFinal later
        for (uint256 i = 0; i < _proofs.length; i++) {
            finalValidatorProofs.push(_proofs[i]);
        }
        console.log(
            "current validator's merkle root is: %s",
            Strings.toHexString(uint256(root), 32)
        );
    }

    function initialize(uint32 _setId) public {
        currentSetId = _setId;
        nextSetId = _setId + 1;
        BeefyClient.ValidatorSet memory vset = BeefyClient.ValidatorSet(
            currentSetId,
            setSize,
            root
        );
        BeefyClient.ValidatorSet memory nextvset = BeefyClient.ValidatorSet(
            nextSetId,
            setSize,
            root
        );
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

        BeefyClient.Commitment memory commitment = BeefyClient.Commitment(
            blockNumber,
            setId,
            payload
        );
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
        BeefyClient.Commitment memory commitment = BeefyClient.Commitment(
            blockNumber,
            setId,
            payload
        );
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

        BeefyClient.Commitment memory commitment = BeefyClient.Commitment(
            blockNumber,
            setId,
            payload
        );
        // invalid bitfield here
        bitfield[0] = 0;
        cheats.expectRevert(BeefyClient.InvalidBitfield.selector);
        beefyClient.submitFinal(commitment, bitfield, finalValidatorProofs);
    }

    function testSubmitFailWithInvalidSignatureProof() public {
        initialize(setId);

        //bad signature in proof
        bytes32 originalSignatureS = validatorProof.s;
        validatorProof.s = bytes32("Hello");
        cheats.expectRevert(BeefyClient.InvalidSignature.selector);
        beefyClient.submitInitial(commitHash, bitfield, validatorProof);

        //bad account in proof
        validatorProof.s = originalSignatureS;
        validatorProof.account = address(0x0);
        cheats.expectRevert(BeefyClient.InvalidValidatorProof.selector);
        beefyClient.submitInitial(commitHash, bitfield, validatorProof);
    }

    function testSubmitWithHandover() public {
        //initialize with previous set
        initialize(setId - 1);

        beefyClient.submitInitialWithHandover(commitHash, bitfield, validatorProof);

        // mine random delay blocks
        cheats.roll(block.number + randaoCommitDelay);

        // set difficulty as PrevRandao
        cheats.difficulty(difficulty);

        beefyClient.commitPrevRandao(commitHash);

        BeefyClient.Commitment memory commitment = BeefyClient.Commitment(
            blockNumber,
            setId,
            payload
        );
        beefyClient.submitFinalWithHandover(
            commitment,
            bitfield,
            finalValidatorProofs,
            mmrLeaf,
            mmrLeafProofs,
            leafProofOrder
        );
        assertEq(beefyClient.latestBeefyBlock(), blockNumber);
    }

    function minimumSignatureThreshold(uint256 validatorSetLen) internal pure returns (uint256) {
        if (validatorSetLen <= 10) {
            return validatorSetLen - (validatorSetLen - 1) / 3;
        } else if (validatorSetLen < 342) {
            return 10;
        } else if (validatorSetLen < 683) {
            return 11;
        } else if (validatorSetLen < 1366) {
            return 12;
        } else if (validatorSetLen < 2731) {
            return 13;
        } else if (validatorSetLen < 5462) {
            return 14;
        } else if (validatorSetLen < 10923) {
            return 15;
        } else if (validatorSetLen < 21846) {
            return 16;
        } else {
            return 17;
        }
    }
}
