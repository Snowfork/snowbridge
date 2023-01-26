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

    function expectRevert(bytes calldata msg) external;

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

    function setUp() public {
        randaoCommitDelay = 3;
        randaoCommitExpiration = 8;
        difficulty = 377;

        beefyClient = new BeefyClient(randaoCommitDelay, randaoCommitExpiration);

        // allocate input command array
        string[] memory inputs = new string[](10);
        inputs[0] = "test/beefy/validator-set.ts";
        // type of command
        inputs[1] = "GenerateInitialSet";
        // generate initial fixture data with ffi
        (blockNumber, setId, setSize, bitSetArray, commitHash, payload) = abi.decode(
            vm.ffi(inputs),
            (uint32, uint32, uint32, uint256[], bytes32, BeefyClient.Payload)
        );
        bitfield = Bitfield.createBitfield(bitSetArray, setSize);

        // To avoid another round of ffi in multiple tests
        // except for the initial merkle root and proof for validators
        // precalculate finalBitfield and finalValidatorProofs
        finalBitfield = Bitfield.randomNBitsWithPriorCheck(
            difficulty,
            bitfield,
            minimumSignatureThreshold(setSize),
            setSize
        );

        inputs[1] = "GenerateProofs";
        //length of finalBitField
        inputs[2] = Strings.toString(finalBitfield.length);
        for (uint i = 0; i < finalBitfield.length; i++) {
            inputs[i + 3] = Strings.toString(finalBitfield[i]);
        }
        BeefyClient.ValidatorProof[] memory _proofs;
        (root, validatorProof, _proofs, mmrLeafProofs, mmrLeaf) = abi.decode(
            vm.ffi(inputs),
            (
                bytes32,
                BeefyClient.ValidatorProof,
                BeefyClient.ValidatorProof[],
                bytes32[],
                BeefyClient.MMRLeaf
            )
        );
        // cache to storage to reuse later in submitFinal
        for (uint i = 0; i < _proofs.length; i++) {
            finalValidatorProofs.push(_proofs[i]);
        }
        console.log(
            "current validator's merkle root is: %s",
            Strings.toHexString(uint256(root), 32)
        );
    }

    function testSubmit() public {
        currentSetId = setId;
        nextSetId = setId + 1;
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

        beefyClient.submitInitial(commitHash, bitfield, validatorProof);

        // mine 3 blocks
        cheats.roll(block.number + randaoCommitDelay);
        // set difficulty as PrevRandao
        cheats.difficulty(difficulty);

        beefyClient.commitPrevRandao(commitHash);

        beefyClient.createFinalBitfield(commitHash, bitfield);

        BeefyClient.Commitment memory commitment = BeefyClient.Commitment(
            blockNumber,
            setId,
            payload
        );
        bytes32 encodedCommitmentHash = keccak256(encodeCommitment(commitment));
        assertEq(encodedCommitmentHash, commitHash);
        beefyClient.submitFinal(commitment, bitfield, finalValidatorProofs);
        assertEq(beefyClient.latestBeefyBlock(), blockNumber);
    }

    function testSubmitWithHandover() public {
        currentSetId = setId - 1;
        nextSetId = setId;
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

        beefyClient.submitInitialWithHandover(commitHash, bitfield, validatorProof);

        // mine 3 blocks
        cheats.roll(block.number + randaoCommitDelay);
        // set difficulty as PrevRandao
        cheats.difficulty(difficulty);

        beefyClient.commitPrevRandao(commitHash);

        beefyClient.createFinalBitfield(commitHash, bitfield);

        BeefyClient.Commitment memory commitment = BeefyClient.Commitment(
            blockNumber,
            setId,
            payload
        );
        // BeefyClient.MMRLeaf memory mmrLeaf = BeefyClient.MMRLeaf(
        //     0,
        //     370,
        //     0x2a74fc1410a321daefc1ae17adc69048db56f4d37660e7af042289480de59897,
        //     38,
        //     3,
        //     0x42b63941ec636f52303b3c33f53349830d8a466e9456d25d22b28f4bb0ad0365,
        //     0xc992465982e7733f5f91c60f6c7c5d4433298c10b348487081f2356d80a0133f
        // );
        uint256 leafProofOrder = 0;
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

    function encodeCommitment(
        BeefyClient.Commitment memory commitment
    ) internal pure returns (bytes memory) {
        return
            bytes.concat(
                commitment.payload.prefix,
                commitment.payload.mmrRootHash,
                commitment.payload.suffix,
                ScaleCodec.encodeU32(commitment.blockNumber),
                ScaleCodec.encodeU64(commitment.validatorSetID)
            );
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
