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
    uint256 minimumSignatureSamples;
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
    BeefyClient.ValidatorProof[] finalValidatorProofs;
    BeefyClient.ValidatorProof[] finalValidatorProofs3SignatureCount;
    bytes32[] mmrLeafProofs;
    BeefyClient.MMRLeaf mmrLeaf;
    uint256 leafProofOrder;
    BeefyClient.MMRLeaf emptyLeaf;
    bytes32[] emptyLeafProofs;
    uint256 emptyLeafProofOrder;
    bytes2 mmrRootID = bytes2("mh");
    string bitFieldFile0SignatureCount;
    string bitFieldFile3SignatureCount;

    function setUp() public {
        randaoCommitDelay = uint8(vm.envOr("RANDAO_COMMIT_DELAY", uint256(3)));
        randaoCommitExpiration = uint8(vm.envOr("RANDAO_COMMIT_EXP", uint256(8)));
        minimumSignatureSamples = uint8(vm.envOr("BEEFY_MINIMUM_SIGNATURE_SAMPLES", uint256(16)));
        prevRandao = uint32(vm.envOr("PREV_RANDAO", uint256(377)));

        string memory beefyCommitmentFile = string.concat(vm.projectRoot(), "/test/data/beefy-commitment.json");

        string memory beefyCommitmentRaw = vm.readFile(beefyCommitmentFile);

        bitFieldFile0SignatureCount = string.concat(vm.projectRoot(), "/test/data/beefy-final-bitfield-0.json");
        bitFieldFile3SignatureCount = string.concat(vm.projectRoot(), "/test/data/beefy-final-bitfield-3.json");

        blockNumber = uint32(beefyCommitmentRaw.readUint(".params.commitment.blockNumber"));
        setId = uint32(beefyCommitmentRaw.readUint(".params.commitment.validatorSetID"));
        commitHash = beefyCommitmentRaw.readBytes32(".commitmentHash");
        mmrRoot = beefyCommitmentRaw.readBytes32(".params.commitment.payload[0].data");
        mmrLeafProofs = beefyCommitmentRaw.readBytes32Array(".params.leafProof");
        leafProofOrder = beefyCommitmentRaw.readUint(".params.leafProofOrder");
        decodeMMRLeaf(beefyCommitmentRaw);

        string memory beefyValidatorSetFile = string.concat(vm.projectRoot(), "/test/data/beefy-validator-set.json");
        string memory beefyValidatorSetRaw = vm.readFile(beefyValidatorSetFile);
        setSize = uint32(beefyValidatorSetRaw.readUint(".validatorSetSize"));
        root = beefyValidatorSetRaw.readBytes32(".validatorRoot");
        bitSetArray = beefyValidatorSetRaw.readUintArray(".participants");
        absentBitSetArray = beefyValidatorSetRaw.readUintArray(".absentees");

        console.log("current validator's merkle root is: %s", Strings.toHexString(uint256(root), 32));

        beefyClient = new BeefyClientMock(randaoCommitDelay, randaoCommitExpiration, minimumSignatureSamples);

        bitfield = beefyClient.createInitialBitfield(bitSetArray, setSize);
        absentBitfield = beefyClient.createInitialBitfield(absentBitSetArray, setSize);

        string memory finalProofFile0SignatureCount =
            string.concat(vm.projectRoot(), "/test/data/beefy-final-proof-0.json");
        string memory finalProofRaw0SignatureCount = vm.readFile(finalProofFile0SignatureCount);
        loadFinalProofs(finalProofRaw0SignatureCount, finalValidatorProofs);

        string memory finalProofFile3SignatureCount =
            string.concat(vm.projectRoot(), "/test/data/beefy-final-proof-3.json");
        string memory finalProofRaw3SignatureCount = vm.readFile(finalProofFile3SignatureCount);
        loadFinalProofs(finalProofRaw3SignatureCount, finalValidatorProofs3SignatureCount);
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

    function loadFinalProofs(string memory finalProofRaw, BeefyClient.ValidatorProof[] storage finalProofs) internal {
        bytes memory proofRaw = finalProofRaw.readBytes(".finalValidatorsProofRaw");
        BeefyClient.ValidatorProof[] memory proofs = abi.decode(proofRaw, (BeefyClient.ValidatorProof[]));
        for (uint256 i = 0; i < proofs.length; i++) {
            finalProofs.push(proofs[i]);
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
    function regenerateBitField(string memory bitfieldFile, uint256 samples) internal {
        console.log("print initialBitField");
        printBitArray(bitfield);
        prevRandao = uint32(vm.envOr("PREV_RANDAO", prevRandao));
        finalBitfield = Bitfield.subsample(prevRandao, bitfield, samples, setSize);
        console.log("print finalBitField");
        printBitArray(finalBitfield);

        string memory finalBitFieldRaw = "";
        finalBitFieldRaw = finalBitFieldRaw.serialize("finalBitFieldRaw", abi.encode(finalBitfield));

        string memory finaliBitFieldStr = "";
        finaliBitFieldStr = finaliBitFieldStr.serialize("finalBitField", finalBitfield);

        string memory output = finalBitFieldRaw.serialize("final", finaliBitFieldStr);

        vm.writeJson(output, bitfieldFile);
    }

    function decodeMMRLeaf(string memory beefyCommitmentRaw) internal {
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

        beefyClient.submitFinal(
            commitment, bitfield, finalValidatorProofs, emptyLeaf, emptyLeafProofs, emptyLeafProofOrder
        );

        assertEq(beefyClient.latestBeefyBlock(), blockNumber);
        return commitment;
    }

    function testSubmitWith3SignatureCount() public returns (BeefyClient.Commitment memory) {
        BeefyClient.Commitment memory commitment = initialize(setId);

        // Signature count is 0 for the first submitInitial.
        assertEq(beefyClient.getValidatorCounter(false, finalValidatorProofs[0].index), 0);
        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[0]);

        // Signature count is now 1 after a second submitInitial.
        assertEq(beefyClient.getValidatorCounter(false, finalValidatorProofs[0].index), 1);
        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[0]);

        // Signature count is still 1 because we use another validator.
        assertEq(beefyClient.getValidatorCounter(false, finalValidatorProofs[1].index), 0);
        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[1]);
        assertEq(beefyClient.getValidatorCounter(false, finalValidatorProofs[1].index), 1);

        // Signature count is now 2 after a third submitInitial.
        assertEq(beefyClient.getValidatorCounter(false, finalValidatorProofs[0].index), 2);
        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[0]);

        // Signature count is now 3 after a forth submitInitial.
        assertEq(beefyClient.getValidatorCounter(false, finalValidatorProofs[0].index), 3);
        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[0]);

        // mine random delay blocks
        vm.roll(block.number + randaoCommitDelay);

        commitPrevRandao();

        createFinalProofs();

        beefyClient.submitFinal(
            commitment, bitfield, finalValidatorProofs3SignatureCount, emptyLeaf, emptyLeafProofs, emptyLeafProofOrder
        );

        assertEq(beefyClient.latestBeefyBlock(), blockNumber);
        assertEq(beefyClient.getValidatorCounter(false, finalValidatorProofs[0].index), 4);
        assertEq(beefyClient.getValidatorCounter(true, finalValidatorProofs[0].index), 0);
        assertEq(beefyClient.getValidatorCounter(false, finalValidatorProofs[1].index), 1);
        assertEq(beefyClient.getValidatorCounter(true, finalValidatorProofs[1].index), 0);
        return commitment;
    }

    function testSubmitFailWithInvalidValidatorProofWhenNotProvidingSignatureCount() public {
        BeefyClient.Commitment memory commitment = initialize(setId);

        // Signature count is 0 for the first submitInitial.
        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[0]);

        // Signature count is now 1 after a second submitInitial.
        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[0]);

        // mine random delay blocks
        vm.roll(block.number + randaoCommitDelay);

        commitPrevRandao();

        createFinalProofs();

        // make an invalid signature
        finalValidatorProofs[0].r = 0xb5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c;
        vm.expectRevert(BeefyClient.InvalidValidatorProof.selector);
        beefyClient.submitFinal(
            commitment, bitfield, finalValidatorProofs, emptyLeaf, emptyLeafProofs, emptyLeafProofOrder
        );
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
        beefyClient.submitFinal(
            commitment, bitfield, finalValidatorProofs, emptyLeaf, emptyLeafProofs, emptyLeafProofOrder
        );
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
        beefyClient.submitFinal(
            commitment, bitfield, finalValidatorProofs, emptyLeaf, emptyLeafProofs, emptyLeafProofOrder
        );
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
        beefyClient.submitFinal(
            commitment, bitfield, finalValidatorProofs, emptyLeaf, emptyLeafProofs, emptyLeafProofOrder
        );
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
        beefyClient.submitFinal(
            commitment, bitfield, finalValidatorProofs, emptyLeaf, emptyLeafProofs, emptyLeafProofOrder
        );
    }

    function testSubmitFailWithoutPrevRandao() public {
        BeefyClient.Commitment memory commitment = initialize(setId);

        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[0]);

        // reverted without commit PrevRandao
        vm.expectRevert(BeefyClient.PrevRandaoNotCaptured.selector);
        beefyClient.submitFinal(
            commitment, bitfield, finalValidatorProofs, emptyLeaf, emptyLeafProofs, emptyLeafProofOrder
        );
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

        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[0]);

        vm.roll(block.number + randaoCommitDelay);

        commitPrevRandao();

        createFinalProofs();

        beefyClient.submitFinal(commitment, bitfield, finalValidatorProofs, mmrLeaf, mmrLeafProofs, leafProofOrder);
        assertEq(beefyClient.latestBeefyBlock(), blockNumber);
    }

    function testSubmitWithHandoverCountersAreCopiedCorrectly() public {
        //initialize with previous set
        BeefyClient.Commitment memory commitment = initialize(setId - 1);

        // submit with the first validator
        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[1]);
        assertEq(beefyClient.getValidatorCounter(false, finalValidatorProofs[1].index), 0);
        assertEq(beefyClient.getValidatorCounter(true, finalValidatorProofs[1].index), 1);

        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[0]);
        assertEq(beefyClient.getValidatorCounter(false, finalValidatorProofs[0].index), 0);
        assertEq(beefyClient.getValidatorCounter(true, finalValidatorProofs[0].index), 1);

        vm.roll(block.number + randaoCommitDelay);

        commitPrevRandao();

        createFinalProofs();

        beefyClient.submitFinal(commitment, bitfield, finalValidatorProofs, mmrLeaf, mmrLeafProofs, leafProofOrder);
        assertEq(beefyClient.latestBeefyBlock(), blockNumber);
        assertEq(beefyClient.getValidatorCounter(false, finalValidatorProofs[0].index), 1);
        assertEq(beefyClient.getValidatorCounter(true, finalValidatorProofs[0].index), 0);
        assertEq(beefyClient.getValidatorCounter(false, finalValidatorProofs[1].index), 1);
        assertEq(beefyClient.getValidatorCounter(true, finalValidatorProofs[1].index), 0);
    }

    function testSubmitWithHandoverAnd3SignatureCount() public {
        //initialize with previous set
        BeefyClient.Commitment memory commitment = initialize(setId - 1);

        // Signature count is 0 for the first submitInitial.
        assertEq(beefyClient.getValidatorCounter(true, finalValidatorProofs[0].index), 0);
        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[0]);

        // Signature count is now 1 after a second submitInitial.
        assertEq(beefyClient.getValidatorCounter(true, finalValidatorProofs[0].index), 1);
        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[0]);

        // Signature count is still 1 because we use another validator.
        assertEq(beefyClient.getValidatorCounter(true, finalValidatorProofs[1].index), 0);
        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[1]);
        assertEq(beefyClient.getValidatorCounter(true, finalValidatorProofs[1].index), 1);

        // Signature count is now 2 after a third submitInitial.
        assertEq(beefyClient.getValidatorCounter(true, finalValidatorProofs[0].index), 2);
        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[0]);

        // Signature count is now 3 after a forth submitInitial.
        assertEq(beefyClient.getValidatorCounter(true, finalValidatorProofs[0].index), 3);
        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[0]);

        vm.roll(block.number + randaoCommitDelay);

        commitPrevRandao();

        createFinalProofs();

        beefyClient.submitFinal(
            commitment, bitfield, finalValidatorProofs3SignatureCount, mmrLeaf, mmrLeafProofs, leafProofOrder
        );
        assertEq(beefyClient.latestBeefyBlock(), blockNumber);
        assertEq(beefyClient.getValidatorCounter(false, finalValidatorProofs[0].index), 4);
        assertEq(beefyClient.getValidatorCounter(true, finalValidatorProofs[0].index), 0);
        assertEq(beefyClient.getValidatorCounter(false, finalValidatorProofs[1].index), 1);
        assertEq(beefyClient.getValidatorCounter(true, finalValidatorProofs[1].index), 0);
    }

    function testSubmitWithHandoverFailWithInvalidValidatorProofWhenNotProvidingSignatureCount() public {
        //initialize with previous set
        BeefyClient.Commitment memory commitment = initialize(setId - 1);

        // Signature count is 0 for the first submitInitial.
        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[0]);

        // Signature count is now 1 after a second submitInitial.
        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[0]);

        vm.roll(block.number + randaoCommitDelay);

        commitPrevRandao();

        createFinalProofs();

        vm.expectRevert(BeefyClient.InvalidValidatorProof.selector);
        beefyClient.submitFinal(commitment, bitfield, finalValidatorProofs, mmrLeaf, mmrLeafProofs, leafProofOrder);
    }

    function testSubmitWithHandoverFailWithoutPrevRandao() public {
        //initialize with previous set
        BeefyClient.Commitment memory commitment = initialize(setId - 1);

        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[0]);

        vm.expectRevert(BeefyClient.PrevRandaoNotCaptured.selector);
        beefyClient.submitFinal(commitment, bitfield, finalValidatorProofs, mmrLeaf, mmrLeafProofs, leafProofOrder);
    }

    function testSubmitWithHandoverFailStaleCommitment() public {
        BeefyClient.Commitment memory commitment = initialize(setId - 1);
        beefyClient.setLatestBeefyBlock(blockNumber);

        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[0]);

        vm.roll(block.number + randaoCommitDelay);

        commitPrevRandao();

        createFinalProofs();

        vm.expectRevert(BeefyClient.StaleCommitment.selector);
        beefyClient.submitFinal(commitment, bitfield, finalValidatorProofs, mmrLeaf, mmrLeafProofs, leafProofOrder);
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
        beefyClient.submitFinal(commitment, bitfield, finalValidatorProofs, mmrLeaf, mmrLeafProofs, leafProofOrder);
    }

    function testSubmitWithHandoverFailWithInvalidValidatorSet() public {
        //initialize with previous set
        BeefyClient.Commitment memory commitment = initialize(setId - 1);

        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[0]);

        vm.roll(block.number + randaoCommitDelay);
        commitPrevRandao();

        createFinalProofs();

        //reinitialize with next validator set
        initialize(setId + 1);
        //submit will be reverted with InvalidCommitment
        vm.expectRevert(BeefyClient.InvalidCommitment.selector);
        beefyClient.submitFinal(commitment, bitfield, finalValidatorProofs, mmrLeaf, mmrLeafProofs, leafProofOrder);
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
        beefyClient.submitFinal(_commitment, bitfield, finalValidatorProofs, mmrLeaf, mmrLeafProofs, leafProofOrder);
    }

    function testSubmitFailWithInvalidMMRLeaf() public {
        //initialize with previous set
        BeefyClient.Commitment memory commitment = initialize(setId - 1);

        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[0]);

        vm.roll(block.number + randaoCommitDelay);

        vm.prevrandao(bytes32(uint256(prevRandao)));

        beefyClient.commitPrevRandao(commitHash);

        createFinalProofs();

        //construct nextAuthoritySetID with a wrong value
        mmrLeaf.nextAuthoritySetID = setId;
        //submit will be reverted with InvalidMMRLeaf
        vm.expectRevert(BeefyClient.InvalidMMRLeaf.selector);
        beefyClient.submitFinal(commitment, bitfield, finalValidatorProofs, mmrLeaf, mmrLeafProofs, leafProofOrder);
    }

    function testSubmitFailWithInvalidMMRLeafProof() public {
        //initialize with previous set
        BeefyClient.Commitment memory commitment = initialize(setId - 1);

        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[0]);

        vm.roll(block.number + randaoCommitDelay);

        commitPrevRandao();

        createFinalProofs();

        //construct parentNumber with a wrong value
        mmrLeaf.parentNumber = 1;
        //submit will be reverted with InvalidMMRLeafProof
        vm.expectRevert(BeefyClient.InvalidMMRLeafProof.selector);
        beefyClient.submitFinal(commitment, bitfield, finalValidatorProofs, mmrLeaf, mmrLeafProofs, leafProofOrder);
    }

    function testSubmitFailWithNotEnoughClaims() public {
        BeefyClient.Commitment memory commitment = initialize(setId);
        uint256[] memory initialBits = absentBitfield;
        Bitfield.set(initialBits, finalValidatorProofs[0].index);
        printBitArray(initialBits);
        vm.expectRevert(BeefyClient.NotEnoughClaims.selector);
        beefyClient.submitInitial(commitment, initialBits, finalValidatorProofs[0]);
    }

    function testRegenerateBitField() public {
        // Generate a bitfield for signature count 0.
        uint256 samples = beefyClient.signatureSamples_public(setSize, 0);
        regenerateBitField(bitFieldFile0SignatureCount, samples);
        // Generate a bitfield for signature count 3.
        samples = beefyClient.signatureSamples_public(setSize, 3);
        regenerateBitField(bitFieldFile3SignatureCount, samples);
    }

    function testSignatureSampleWithUseMajorityIfVsetIsSmallerThanMinSignatures() public {
        assertEq(7, beefyClient.signatureSamples_public(9, 0), "dynamicSignatures incorrect.");
        assertEq(11, beefyClient.signatureSamples_public(16, 0), "dynamicSignatures incorrect.");
        assertEq(16, beefyClient.signatureSamples_public(17, 0), "dynamicSignatures incorrect.");
    }

    function testSignatureSampleNeverYieldsASampleGreaterThanTheValidatorSetLength() public {
        assertEq(17, beefyClient.signatureSamples_public(18, 0), "dynamicSignatures incorrect.");
        assertEq(18, beefyClient.signatureSamples_public(19, 0), "dynamicSignatures incorrect.");
        assertEq(18, beefyClient.signatureSamples_public(20, 0), "dynamicSignatures incorrect.");
        assertEq(19, beefyClient.signatureSamples_public(21, 0), "dynamicSignatures incorrect.");
        assertEq(20, beefyClient.signatureSamples_public(30, 0), "dynamicSignatures incorrect.");

        assertEq(29, beefyClient.signatureSamples_public(30, 10), "dynamicSignatures incorrect.");
        assertEq(30, beefyClient.signatureSamples_public(30, 17), "dynamicSignatures incorrect.");
    }

    function testSignatureSamplingRanges() public {
        assertEq(25, beefyClient.signatureSamples_public(setSize, 0), "dynamicSignatures incorrect.");
        assertEq(26, beefyClient.signatureSamples_public(setSize, 1), "dynamicSignatures incorrect.");
        assertEq(28, beefyClient.signatureSamples_public(setSize, 2), "dynamicSignatures incorrect.");
        assertEq(30, beefyClient.signatureSamples_public(setSize, 3), "dynamicSignatures incorrect.");
        assertEq(30, beefyClient.signatureSamples_public(setSize, 4), "dynamicSignatures incorrect.");
        assertEq(32, beefyClient.signatureSamples_public(setSize, 5), "dynamicSignatures incorrect.");
        assertEq(32, beefyClient.signatureSamples_public(setSize, 6), "dynamicSignatures incorrect.");
        assertEq(32, beefyClient.signatureSamples_public(setSize, 8), "dynamicSignatures incorrect.");
        assertEq(34, beefyClient.signatureSamples_public(setSize, 9), "dynamicSignatures incorrect.");
        assertEq(34, beefyClient.signatureSamples_public(setSize, 12), "dynamicSignatures incorrect.");
        assertEq(34, beefyClient.signatureSamples_public(setSize, 16), "dynamicSignatures incorrect.");
        assertEq(36, beefyClient.signatureSamples_public(setSize, 17), "dynamicSignatures incorrect.");
        assertEq(36, beefyClient.signatureSamples_public(setSize, 24), "dynamicSignatures incorrect.");
        assertEq(36, beefyClient.signatureSamples_public(setSize, 32), "dynamicSignatures incorrect.");
        assertEq(38, beefyClient.signatureSamples_public(setSize, 33), "dynamicSignatures incorrect.");
        assertEq(38, beefyClient.signatureSamples_public(setSize, 48), "dynamicSignatures incorrect.");
        assertEq(38, beefyClient.signatureSamples_public(setSize, 64), "dynamicSignatures incorrect.");
        assertEq(40, beefyClient.signatureSamples_public(setSize, 65), "dynamicSignatures incorrect.");
        assertEq(40, beefyClient.signatureSamples_public(setSize, 96), "dynamicSignatures incorrect.");
        assertEq(40, beefyClient.signatureSamples_public(setSize, 128), "dynamicSignatures incorrect.");
        assertEq(42, beefyClient.signatureSamples_public(setSize, 129), "dynamicSignatures incorrect.");
        assertEq(42, beefyClient.signatureSamples_public(setSize, 192), "dynamicSignatures incorrect.");
        assertEq(42, beefyClient.signatureSamples_public(setSize, 256), "dynamicSignatures incorrect.");
        assertEq(44, beefyClient.signatureSamples_public(setSize, 257), "dynamicSignatures incorrect.");
        assertEq(44, beefyClient.signatureSamples_public(setSize, 384), "dynamicSignatures incorrect.");
        assertEq(44, beefyClient.signatureSamples_public(setSize, 512), "dynamicSignatures incorrect.");
        assertEq(46, beefyClient.signatureSamples_public(setSize, 513), "dynamicSignatures incorrect.");
        assertEq(46, beefyClient.signatureSamples_public(setSize, 768), "dynamicSignatures incorrect.");
        assertEq(46, beefyClient.signatureSamples_public(setSize, 1024), "dynamicSignatures incorrect.");
        assertEq(48, beefyClient.signatureSamples_public(setSize, 1025), "dynamicSignatures incorrect.");
        assertEq(48, beefyClient.signatureSamples_public(setSize, 1536), "dynamicSignatures incorrect.");
        assertEq(48, beefyClient.signatureSamples_public(setSize, 2048), "dynamicSignatures incorrect.");
        assertEq(50, beefyClient.signatureSamples_public(setSize, 2049), "dynamicSignatures incorrect.");
        assertEq(50, beefyClient.signatureSamples_public(setSize, 3072), "dynamicSignatures incorrect.");
        assertEq(50, beefyClient.signatureSamples_public(setSize, 4096), "dynamicSignatures incorrect.");
        assertEq(52, beefyClient.signatureSamples_public(setSize, 4097), "dynamicSignatures incorrect.");
        assertEq(52, beefyClient.signatureSamples_public(setSize, 6144), "dynamicSignatures incorrect.");
        assertEq(52, beefyClient.signatureSamples_public(setSize, 8192), "dynamicSignatures incorrect.");
        assertEq(54, beefyClient.signatureSamples_public(setSize, 8193), "dynamicSignatures incorrect.");
        assertEq(54, beefyClient.signatureSamples_public(setSize, 12288), "dynamicSignatures incorrect.");
        assertEq(54, beefyClient.signatureSamples_public(setSize, 16384), "dynamicSignatures incorrect.");
        assertEq(56, beefyClient.signatureSamples_public(setSize, 16385), "dynamicSignatures incorrect.");
        assertEq(56, beefyClient.signatureSamples_public(setSize, 24576), "dynamicSignatures incorrect.");
        assertEq(56, beefyClient.signatureSamples_public(setSize, 32768), "dynamicSignatures incorrect.");
        assertEq(58, beefyClient.signatureSamples_public(setSize, 32769), "dynamicSignatures incorrect.");
        assertEq(58, beefyClient.signatureSamples_public(setSize, 49152), "dynamicSignatures incorrect.");
        assertEq(58, beefyClient.signatureSamples_public(setSize, 65535), "dynamicSignatures incorrect.");
    }
}
