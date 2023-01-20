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
    bytes32 mmrRootHash;
    uint32 setSize;
    uint32 setId;
    uint128 currentSetId;
    uint128 nextSetId;
    uint32 setIndex;
    bytes32 commitHash;
    bytes32 root;
    uint256[] bitSetArray;
    uint256[] bitfield;
    bytes prefix;
    bytes suffix;
    uint256[] finalBitfield;
    BeefyClient.ValidatorProof validatorProof;
    BeefyClient.ValidatorProof[] finalValidatorProofs;
    bytes32[] mmrLeafProofs;

    function setUp() public {
        randaoCommitDelay = 3;
        randaoCommitExpiration = 8;
        blockNumber = 371;
        difficulty = 377;
        mmrRootHash = 0x482fcbd18294c4b4f339f825537530cfcc678eeea469caa807438d35ace62f04;
        setSize = 300;
        setId = 37;
        setIndex = 0;
        commitHash = 0x243baf0066d021d42716081dad0b30499dad95a300daa269ed8f6f6334d95975;
        prefix = hex"046d6880";
        suffix = hex"";

        mmrLeafProofs = [
            bytes32(0xe8ae8d4c8027764aa0fdae351c30c6085f7822ad6295ae1bd445ee8bef564901),
            bytes32(0xe4d591609cb75673ef8992d1ae6c518ad95d8f924f75249ce43153d01380c79f),
            bytes32(0xb2852e70b508acbda330c6f842d51f4eab82d34b991fe6679d37f2eeedae6ccd),
            bytes32(0x6a83a49e6424b0de032f730064213f4783f2c9f59dab4480f88673a042102ab2),
            bytes32(0xee4688d1831443e4c7f2d47265fd529dd50e41a4c49c5f31a04bf45320f59614)
        ];

        beefyClient = new BeefyClient(randaoCommitDelay, randaoCommitExpiration);

        // allocate input command array with length as 20
        string[] memory inputs = new string[](20);
        inputs[0] = "test/beefy/validator-set.ts";
        // type of command
        inputs[1] = "RandomSubset";
        // validatorSetId
        inputs[2] = Strings.toString(setId);
        // validatorSetSize
        inputs[3] = Strings.toString(setSize);
        // generate random bit set array with ffi
        (bitSetArray) = abi.decode(vm.ffi(inputs), (uint256[]));
        console.logUint(bitSetArray.length);

        bitfield = Bitfield.createBitfield(bitSetArray, setSize);

        // To avoid the slow ffi in submit test precalculate
        // finalBitfield and finalValidatorProofs and save to storage
        finalBitfield = Bitfield.randomNBitsWithPriorCheck(
            difficulty,
            bitfield,
            minimumSignatureThreshold(setSize),
            setSize
        );

        inputs[1] = "FinalProof";
        inputs[4] = Strings.toString(setIndex);
        inputs[5] = Strings.toHexString(uint256(commitHash), 32);
        inputs[6] = Strings.toString(finalBitfield.length);
        for (uint i = 0; i < finalBitfield.length; i++) {
            inputs[i + 7] = Strings.toString(finalBitfield[i]);
        }
        BeefyClient.ValidatorProof[] memory proofs;
        (root, validatorProof, proofs) = abi.decode(
            vm.ffi(inputs),
            (bytes32, BeefyClient.ValidatorProof, BeefyClient.ValidatorProof[])
        );
        console.logBytes32(root);

        // cache to storage and reuse later in submitFinal
        for (uint i = 0; i < proofs.length; i++) {
            finalValidatorProofs.push(proofs[i]);
        }
    }

    function testSubmit() public {
        currentSetId = setId;
        nextSetId = setId + 1;
        BeefyClient.ValidatorSet memory vset = BeefyClient.ValidatorSet(currentSetId, setSize, root);
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

        BeefyClient.Payload memory payload = BeefyClient.Payload(mmrRootHash, prefix, suffix);
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
        BeefyClient.ValidatorSet memory vset = BeefyClient.ValidatorSet(currentSetId, setSize, root);
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

        BeefyClient.Payload memory payload = BeefyClient.Payload(mmrRootHash, prefix, suffix);
        BeefyClient.Commitment memory commitment = BeefyClient.Commitment(
            blockNumber,
            setId,
            payload
        );
        BeefyClient.MMRLeaf memory mmrLeaf = BeefyClient.MMRLeaf(0,370,
        0x2a74fc1410a321daefc1ae17adc69048db56f4d37660e7af042289480de59897,
        38,3,0x42b63941ec636f52303b3c33f53349830d8a466e9456d25d22b28f4bb0ad0365,
        0xc992465982e7733f5f91c60f6c7c5d4433298c10b348487081f2356d80a0133f
        );
        uint256 leafProofOrder = 0;
        beefyClient.submitFinalWithHandover(commitment, bitfield, finalValidatorProofs,mmrLeaf,mmrLeafProofs,leafProofOrder);
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
