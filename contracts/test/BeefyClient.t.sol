// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

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
    uint256 minNumRequiredSignatures;
    uint256 fiatShamirRequiresSignatures;
    uint256 signatureUsageCount;
    uint32 blockNumber;
    uint32 prevRandao;
    uint32 setSize;
    uint32 setId;
    uint128 currentSetId;
    uint128 nextSetId;
    bytes32 commitHash;
    bytes32 root;
    uint256[] bitSetArray;
    uint256[] bitfield;
    bytes32 mmrRoot;
    uint256[] finalBitfield;
    BeefyClient.ValidatorProof[] finalValidatorProofs;
    bytes32[] mmrLeafProofs;
    BeefyClient.MMRLeaf mmrLeaf;
    uint256 leafProofOrder;
    BeefyClient.MMRLeaf emptyLeaf;
    bytes32[] emptyLeafProofs;
    uint256 emptyLeafProofOrder;
    bytes2 mmrRootID = bytes2("mh");
    string bitFieldFile;
    uint256[] fiatShamirFinalBitfield;
    string fiatShamirBitFieldFile;
    BeefyClient.ValidatorProof[] fiatShamirValidatorProofs;

    function setUp() public {
        randaoCommitDelay = uint8(vm.envOr("RANDAO_COMMIT_DELAY", uint256(3)));
        randaoCommitExpiration = uint8(vm.envOr("RANDAO_COMMIT_EXP", uint256(8)));
        minNumRequiredSignatures = uint8(vm.envOr("MINIMUM_REQUIRED_SIGNATURES", uint256(17)));
        fiatShamirRequiresSignatures = vm.envOr("FIAT_SHAMIR_REQUIRED_SIGNATURES", uint256(101));
        signatureUsageCount = vm.envOr("SIGNATURE_USAGE_COUNT", uint256(0));
        prevRandao = uint32(vm.envOr("PREV_RANDAO", uint256(377)));

        string memory beefyCommitmentFile =
            string.concat(vm.projectRoot(), "/test/data/beefy-commitment.json");

        string memory beefyCommitmentRaw = vm.readFile(beefyCommitmentFile);

        bitFieldFile = string.concat(vm.projectRoot(), "/test/data/beefy-final-bitfield.json");
        fiatShamirBitFieldFile =
            string.concat(vm.projectRoot(), "/test/data/beefy-fiat-shamir-bitfield.json");

        blockNumber = uint32(beefyCommitmentRaw.readUint(".params.commitment.blockNumber"));
        setId = uint32(beefyCommitmentRaw.readUint(".params.commitment.validatorSetID"));
        commitHash = beefyCommitmentRaw.readBytes32(".commitmentHash");
        mmrRoot = beefyCommitmentRaw.readBytes32(".params.commitment.payload[0].data");
        mmrLeafProofs = beefyCommitmentRaw.readBytes32Array(".params.leafProof");
        leafProofOrder = beefyCommitmentRaw.readUint(".params.leafProofOrder");
        decodeMMRLeaf(beefyCommitmentRaw);

        string memory beefyValidatorSetFile =
            string.concat(vm.projectRoot(), "/test/data/beefy-validator-set.json");
        string memory beefyValidatorSetRaw = vm.readFile(beefyValidatorSetFile);
        setSize = uint32(beefyValidatorSetRaw.readUint(".validatorSetSize"));
        root = beefyValidatorSetRaw.readBytes32(".validatorRoot");
        bitSetArray = beefyValidatorSetRaw.readUintArray(".participants");

        console.log(
            "current validator's merkle root is: %s", Strings.toHexString(uint256(root), 32)
        );

        beefyClient = new BeefyClientMock(
            randaoCommitDelay,
            randaoCommitExpiration,
            minNumRequiredSignatures,
            fiatShamirRequiresSignatures,
            0,
            BeefyClient.ValidatorSet(0, 0, 0x0),
            BeefyClient.ValidatorSet(1, 0, 0x0)
        );

        bitfield = beefyClient.createInitialBitfield(bitSetArray, setSize);

        string memory finalProofFile =
            string.concat(vm.projectRoot(), "/test/data/beefy-final-proof.json");
        string memory finalProofRaw = vm.readFile(finalProofFile);
        loadFinalProofs(finalProofRaw, finalValidatorProofs);

        string memory fiatShamirProofFile =
            string.concat(vm.projectRoot(), "/test/data/beefy-fiat-shamir-proof.json");
        string memory fiatShamirProofRaw = vm.readFile(fiatShamirProofFile);
        loadFinalProofs(fiatShamirProofRaw, fiatShamirValidatorProofs);
    }

    function initialize(uint32 _setId) public returns (BeefyClient.Commitment memory) {
        currentSetId = _setId;
        nextSetId = _setId + 1;
        BeefyClient.ValidatorSet memory vset =
            BeefyClient.ValidatorSet(currentSetId, setSize, root);
        BeefyClient.ValidatorSet memory nextvset =
            BeefyClient.ValidatorSet(nextSetId, setSize, root);
        beefyClient.initialize_public(0, vset, nextvset);
        BeefyClient.PayloadItem[] memory payload = new BeefyClient.PayloadItem[](1);
        payload[0] = BeefyClient.PayloadItem(mmrRootID, abi.encodePacked(mmrRoot));
        return BeefyClient.Commitment(blockNumber, setId, payload);
    }

    function loadFinalProofs(
        string memory finalProofRaw,
        BeefyClient.ValidatorProof[] storage finalProofs
    ) internal {
        bytes memory proofRaw = finalProofRaw.readBytes(".finalValidatorsProofRaw");
        BeefyClient.ValidatorProof[] memory proofs =
            abi.decode(proofRaw, (BeefyClient.ValidatorProof[]));
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
    function regenerateBitField(string memory bitfieldFile, uint256 numRequiredSignatures)
        internal
    {
        prevRandao = uint32(vm.envOr("PREV_RANDAO", prevRandao));
        finalBitfield = Bitfield.subsample(prevRandao, bitfield, setSize, numRequiredSignatures);

        string memory finalBitFieldRaw = "";
        finalBitFieldRaw =
            finalBitFieldRaw.serialize("finalBitFieldRaw", abi.encode(finalBitfield));

        string memory finaliBitFieldStr = "";
        finaliBitFieldStr = finaliBitFieldStr.serialize("finalBitField", finalBitfield);

        string memory output = finalBitFieldRaw.serialize("final", finaliBitFieldStr);

        vm.writeJson(output, bitfieldFile);
    }

    function decodeMMRLeaf(string memory beefyCommitmentRaw) internal {
        uint8 version = uint8(beefyCommitmentRaw.readUint(".params.leaf.version"));
        uint32 parentNumber = uint32(beefyCommitmentRaw.readUint(".params.leaf.parentNumber"));
        bytes32 parentHash = beefyCommitmentRaw.readBytes32(".params.leaf.parentHash");
        uint64 nextAuthoritySetID =
            uint64(beefyCommitmentRaw.readUint(".params.leaf.nextAuthoritySetID"));
        uint32 nextAuthoritySetLen =
            uint32(beefyCommitmentRaw.readUint(".params.leaf.nextAuthoritySetLen"));
        bytes32 nextAuthoritySetRoot =
            beefyCommitmentRaw.readBytes32(".params.leaf.nextAuthoritySetRoot");
        bytes32 parachainHeadsRoot =
            beefyCommitmentRaw.readBytes32(".params.leaf.parachainHeadsRoot");
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

    function testSubmitHappyPath() public returns (BeefyClient.Commitment memory) {
        BeefyClient.Commitment memory commitment = initialize(setId);

        assertEq(beefyClient.getValidatorCounter(false, finalValidatorProofs[0].index), 0);
        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[0]);
        assertEq(beefyClient.getValidatorCounter(false, finalValidatorProofs[0].index), 1);

        // mine random delay blocks
        vm.roll(block.number + randaoCommitDelay);

        commitPrevRandao();

        createFinalProofs();

        beefyClient.submitFinal(
            commitment,
            bitfield,
            finalValidatorProofs,
            emptyLeaf,
            emptyLeafProofs,
            emptyLeafProofOrder
        );

        assertEq(beefyClient.latestBeefyBlock(), blockNumber);
        assertEq(beefyClient.getValidatorCounter(false, finalValidatorProofs[0].index), 1);
        assertEq(beefyClient.getValidatorCounter(true, finalValidatorProofs[0].index), 0);
        return commitment;
    }

    function testSubmitWithOldBlockFailsWithStaleCommitment() public {
        BeefyClient.Commitment memory commitment = initialize(setId);
        beefyClient.setLatestBeefyBlock(commitment.blockNumber + 1);
        vm.expectRevert(BeefyClient.StaleCommitment.selector);
        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[0]);
    }

    function testSubmitWithHandoverAndOldBlockFailsWithStaleCommitment() public {
        BeefyClient.Commitment memory commitment = initialize(setId - 1);
        beefyClient.setLatestBeefyBlock(commitment.blockNumber + 1);
        vm.expectRevert(BeefyClient.StaleCommitment.selector);
        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[0]);
    }

    function testSubmitFailWithInvalidValidatorProofWhenNotProvidingSignatureCount() public {
        BeefyClient.Commitment memory commitment = initialize(setId);

        // Signature count is 0 for the first submitInitial.
        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[0]);

        // Signature count is now 1 after a second submitInitial.
        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[0]);

        // Signature count is now 2 after a third submitInitial.
        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[0]);

        // mine random delay blocks
        vm.roll(block.number + randaoCommitDelay);

        commitPrevRandao();

        createFinalProofs();

        // make an invalid signature
        vm.expectRevert(BeefyClient.InvalidValidatorProofLength.selector);
        beefyClient.submitFinal(
            commitment, bitfield, finalValidatorProofs, mmrLeaf, mmrLeafProofs, leafProofOrder
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
        finalValidatorProofs[0].r =
        0xb5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c;
        vm.expectRevert(BeefyClient.InvalidSignature.selector);
        beefyClient.submitFinal(
            commitment,
            bitfield,
            finalValidatorProofs,
            emptyLeaf,
            emptyLeafProofs,
            emptyLeafProofOrder
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
            commitment,
            bitfield,
            finalValidatorProofs,
            emptyLeaf,
            emptyLeafProofs,
            emptyLeafProofOrder
        );
    }

    function testSubmitFailWithStaleCommitment() public {
        BeefyClient.Commitment memory commitment = initialize(setId);

        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[0]);

        // mine random delay blocks
        vm.roll(block.number + randaoCommitDelay);

        commitPrevRandao();

        createFinalProofs();

        // Simulates another submitFinal incrementing the latestBeefyBlock
        beefyClient.setLatestBeefyBlock(commitment.blockNumber + 1);

        vm.expectRevert(BeefyClient.StaleCommitment.selector);
        beefyClient.submitFinal(
            commitment,
            bitfield,
            finalValidatorProofs,
            emptyLeaf,
            emptyLeafProofs,
            emptyLeafProofOrder
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
            commitment,
            bitfield,
            finalValidatorProofs,
            emptyLeaf,
            emptyLeafProofs,
            emptyLeafProofOrder
        );
    }

    function testSubmitFailWithoutPrevRandao() public {
        BeefyClient.Commitment memory commitment = initialize(setId);

        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[0]);

        // reverted without commit PrevRandao
        vm.expectRevert(BeefyClient.PrevRandaoNotCaptured.selector);
        beefyClient.submitFinal(
            commitment,
            bitfield,
            finalValidatorProofs,
            emptyLeaf,
            emptyLeafProofs,
            emptyLeafProofOrder
        );
    }

    function testSubmitFailForPrevRandaoTooEarlyOrTooLate() public {
        BeefyClient.Commitment memory commitment = initialize(setId);

        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[0]);
        // reverted for commit PrevRandao too early
        vm.expectRevert(BeefyClient.WaitPeriodNotOver.selector);
        commitPrevRandao();

        // ticket deleted if PrevRandao commit is submitted too late
        vm.roll(block.number + randaoCommitDelay + randaoCommitExpiration + 1);
        commitPrevRandao();
        BeefyClient.Ticket memory ticket = beefyClient.getTicket(commitHash);
        assertEq(ticket.prevRandao, 0);
        assertEq(ticket.blockNumber, 0);
        assertEq(ticket.bitfieldHash, bytes32(0));
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

        assertEq(beefyClient.getValidatorCounter(false, finalValidatorProofs[0].index), 0);
        assertEq(beefyClient.getValidatorCounter(true, finalValidatorProofs[0].index), 0);
        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[0]);
        assertEq(beefyClient.getValidatorCounter(false, finalValidatorProofs[0].index), 0);
        assertEq(beefyClient.getValidatorCounter(true, finalValidatorProofs[0].index), 1);

        vm.roll(block.number + randaoCommitDelay);

        commitPrevRandao();

        createFinalProofs();

        beefyClient.submitFinal(
            commitment, bitfield, finalValidatorProofs, mmrLeaf, mmrLeafProofs, leafProofOrder
        );
        assertEq(beefyClient.latestBeefyBlock(), blockNumber);
        assertEq(beefyClient.getValidatorCounter(false, finalValidatorProofs[0].index), 1);
        assertEq(beefyClient.getValidatorCounter(true, finalValidatorProofs[0].index), 0);
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

        beefyClient.submitFinal(
            commitment, bitfield, finalValidatorProofs, mmrLeaf, mmrLeafProofs, leafProofOrder
        );
        assertEq(beefyClient.latestBeefyBlock(), blockNumber);
        assertEq(beefyClient.getValidatorCounter(false, finalValidatorProofs[0].index), 1);
        assertEq(beefyClient.getValidatorCounter(true, finalValidatorProofs[0].index), 0);
        assertEq(beefyClient.getValidatorCounter(false, finalValidatorProofs[1].index), 1);
        assertEq(beefyClient.getValidatorCounter(true, finalValidatorProofs[1].index), 0);
    }

    function testCommitPrevRandaoCalledInSequence() public {
        vm.expectRevert(BeefyClient.InvalidTicket.selector);
        commitPrevRandao();
    }

    function testSubmitWithHandoverFailWithInvalidValidatorProofWhenNotProvidingSignatureCount()
        public
    {
        //initialize with previous set
        BeefyClient.Commitment memory commitment = initialize(setId - 1);

        // Signature count is 0 for the first submitInitial.
        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[0]);

        // Signature count is now 1 after a second submitInitial.
        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[0]);

        // Signature count is now 2 after a third submitInitial.
        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[0]);

        vm.roll(block.number + randaoCommitDelay);

        commitPrevRandao();

        createFinalProofs();

        vm.expectRevert(BeefyClient.InvalidValidatorProofLength.selector);
        beefyClient.submitFinal(
            commitment, bitfield, finalValidatorProofs, mmrLeaf, mmrLeafProofs, leafProofOrder
        );
    }

    function testSubmitWithHandoverFailWithoutPrevRandao() public {
        //initialize with previous set
        BeefyClient.Commitment memory commitment = initialize(setId - 1);

        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[0]);

        vm.expectRevert(BeefyClient.PrevRandaoNotCaptured.selector);
        beefyClient.submitFinal(
            commitment, bitfield, finalValidatorProofs, mmrLeaf, mmrLeafProofs, leafProofOrder
        );
    }

    function testSubmitWithHandoverFailStaleCommitment() public {
        BeefyClient.Commitment memory commitment = initialize(setId - 1);

        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[0]);

        // mine random delay blocks
        vm.roll(block.number + randaoCommitDelay);

        commitPrevRandao();

        createFinalProofs();

        // Simulates another submitFinal incrementing the latestBeefyBlock
        beefyClient.setLatestBeefyBlock(commitment.blockNumber + 1);

        vm.expectRevert(BeefyClient.StaleCommitment.selector);
        beefyClient.submitFinal(
            commitment,
            bitfield,
            finalValidatorProofs,
            emptyLeaf,
            emptyLeafProofs,
            emptyLeafProofOrder
        );
    }

    function testScaleEncodeCommit() public {
        BeefyClient.PayloadItem[] memory _payload = new BeefyClient.PayloadItem[](2);
        _payload[0] = BeefyClient.PayloadItem(bytes2("ab"), hex"000102");
        _payload[1] = BeefyClient.PayloadItem(
            mmrRootID, hex"3ac49cd24778522203e8bf40a4712ea3f07c3803bbd638cb53ebb3564ec13e8c"
        );

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
        beefyClient.submitFinal(
            commitment, bitfield, finalValidatorProofs, mmrLeaf, mmrLeafProofs, leafProofOrder
        );
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
        beefyClient.submitFinal(
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
        BeefyClient.Commitment memory _commitment =
            BeefyClient.Commitment(blockNumber, setId + 1, commitment.payload);
        //submit will be reverted with InvalidTicket
        vm.expectRevert(BeefyClient.InvalidTicket.selector);
        beefyClient.submitFinal(
            _commitment, bitfield, finalValidatorProofs, mmrLeaf, mmrLeafProofs, leafProofOrder
        );
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
        beefyClient.submitFinal(
            commitment, bitfield, finalValidatorProofs, mmrLeaf, mmrLeafProofs, leafProofOrder
        );
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
        beefyClient.submitFinal(
            commitment, bitfield, finalValidatorProofs, mmrLeaf, mmrLeafProofs, leafProofOrder
        );
    }

    function testSubmitFailWithNotEnoughClaims() public {
        BeefyClient.Commitment memory commitment = initialize(setId);
        uint256[] memory bitSetArray2 = bitSetArray;
        // New length is 5 less than two thirds of the validator set
        assembly {
            mstore(bitSetArray2, 5)
        }
        uint256[] memory initialBits = beefyClient.createInitialBitfield(bitSetArray2, setSize);
        Bitfield.set(initialBits, finalValidatorProofs[0].index);
        vm.expectRevert(BeefyClient.InvalidBitfield.selector);
        beefyClient.submitInitial(commitment, initialBits, finalValidatorProofs[0]);
    }

    function testRegenerateBitField() public {
        console.log("validator set size:", setSize);
        console.log("minimum required signatures:", minNumRequiredSignatures);
        console.log("signature usage count:", signatureUsageCount);
        // Generate a bitfield for initialized signature count.
        uint256 numRequiredSignatures = beefyClient.computeNumRequiredSignatures_public(
            setSize, signatureUsageCount, minNumRequiredSignatures
        );
        console.log("computed required signatures", numRequiredSignatures);
        regenerateBitField(bitFieldFile, numRequiredSignatures);
    }

    function testFuzzComputeValidatorSetQuorum(uint128 validatorSetLen) public {
        // There must be atleast 1 validator.
        vm.assume(validatorSetLen > 0);
        // Calculator 1/3 with flooring due to integer division.
        uint256 oneThirdMajority = uint256(validatorSetLen) / 3;
        uint256 result = beefyClient.computeQuorum_public(validatorSetLen);
        assertGt(result, oneThirdMajority, "result is greater than 1/3rd");
        assertLe(result, validatorSetLen, "result is less than validator set length.");
        assertGt(result, 0, "result is greater than zero.");
    }

    function testFuzzSignatureSamplingRanges(uint128 validatorSetLen, uint16 minSignatures)
        public
    {
        // There must be atleast 1 validator.
        vm.assume(validatorSetLen > 0);
        // Min signatures must be less than the amount of validators.
        vm.assume(beefyClient.computeMaxRequiredSignatures_public(validatorSetLen) > minSignatures);

        uint256 result = beefyClient.computeNumRequiredSignatures_public(
            validatorSetLen, signatureUsageCount, minSignatures
        );

        // Calculator 2/3 with flooring due to integer division plus 1.
        uint256 twoThirdsMajority = (uint256(validatorSetLen) * 2) / 3 + 1;
        assertLe(result, twoThirdsMajority, "result is less than or equal to quorum.");
        assertGe(result, minSignatures, "result is greater than or equal to minimum signatures.");
        assertLe(result, validatorSetLen, "result is less than validator set length.");
        assertGt(result, 0, "result is greater than zero.");
    }

    function testSignatureSamplingCases() public {
        uint256 result = beefyClient.computeQuorum_public(1);
        assertEq(1, result, "B");
        result = beefyClient.computeNumRequiredSignatures_public(1, 0, 0);
        assertEq(1, result, "C");
    }

    function testStorageToStorageCopies() public {
        beefyClient.copyCounters();
    }

    function testFuzzInitializationValidation(uint128 currentId, uint128 nextId) public {
        vm.assume(currentId < type(uint128).max);
        vm.assume(currentId + 1 != nextId);
        vm.expectRevert("invalid-constructor-params");
        new BeefyClient(
            randaoCommitDelay,
            randaoCommitExpiration,
            minNumRequiredSignatures,
            fiatShamirRequiresSignatures,
            0,
            BeefyClient.ValidatorSet(currentId, 0, 0x0),
            BeefyClient.ValidatorSet(nextId, 0, 0x0)
        );
    }

    function testRegenerateFiatShamirProofs() public {
        BeefyClient.Commitment memory commitment = initialize(setId);

        fiatShamirFinalBitfield = beefyClient.createFiatShamirFinalBitfield(commitment, bitfield);

        string memory finalBitFieldRaw = "";
        finalBitFieldRaw =
            finalBitFieldRaw.serialize("finalBitFieldRaw", abi.encode(fiatShamirFinalBitfield));

        string memory finaliBitFieldStr = "";
        finaliBitFieldStr = finaliBitFieldStr.serialize("finalBitField", fiatShamirFinalBitfield);

        string memory output = finalBitFieldRaw.serialize("final", finaliBitFieldStr);

        vm.writeJson(output, fiatShamirBitFieldFile);
    }

    function testSubmitFiatShamir() public returns (BeefyClient.Commitment memory) {
        BeefyClient.Commitment memory commitment = initialize(setId);

        beefyClient.submitFiatShamir(
            commitment,
            bitfield,
            fiatShamirValidatorProofs,
            emptyLeaf,
            emptyLeafProofs,
            emptyLeafProofOrder
        );

        assertEq(beefyClient.latestBeefyBlock(), blockNumber);
        return commitment;
    }

    function testSubmitFiatShamirWithHandOver() public {
        //initialize with previous set
        BeefyClient.Commitment memory commitment = initialize(setId - 1);

        beefyClient.submitFiatShamir(
            commitment, bitfield, fiatShamirValidatorProofs, mmrLeaf, mmrLeafProofs, leafProofOrder
        );
        assertEq(beefyClient.latestBeefyBlock(), blockNumber);
    }

    function testSubmitFiatShamirWithRaceCondition()
        public
        returns (BeefyClient.Commitment memory)
    {
        BeefyClient.Commitment memory commitment = initialize(setId);

        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[0]);

        vm.roll(block.number + randaoCommitDelay);

        commitPrevRandao();

        createFinalProofs();

        beefyClient.submitFiatShamir(
            commitment,
            bitfield,
            fiatShamirValidatorProofs,
            emptyLeaf,
            emptyLeafProofs,
            emptyLeafProofOrder
        );

        assertEq(beefyClient.latestBeefyBlock(), blockNumber);

        vm.expectRevert(BeefyClient.StaleCommitment.selector);
        beefyClient.submitFinal(
            commitment,
            bitfield,
            finalValidatorProofs,
            emptyLeaf,
            emptyLeafProofs,
            emptyLeafProofOrder
        );
        return commitment;
    }
}
