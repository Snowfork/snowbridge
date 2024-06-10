// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.25;

import {Strings} from "openzeppelin/utils/Strings.sol";
import {Test} from "forge-std/Test.sol";
import {console} from "forge-std/console.sol";
import {stdJson} from "forge-std/StdJson.sol";

import {BeefyClient} from "../src/BeefyClient.sol";
import {BeefyClientMock} from "./mocks/BeefyClientMock.sol";
import {ScaleCodec} from "../src/utils/ScaleCodec.sol";
import {Bitfield} from "../src/utils/Bitfield.sol";

contract BeefyClientTestV2 is Test {
    using stdJson for string;

    BeefyClientMock beefyClient;
    uint8 randaoCommitDelay;
    uint8 randaoCommitExpiration;
    uint256 minNumRequiredSignatures;
    uint32 prevRandao;
    bytes2 mmrRootID = bytes2("mh");

    function setUp() public {
        string memory root = vm.projectRoot();
        string memory beefyCheckpointFile = string.concat(root, "/beefy-state.json");
        string memory beefyCheckpointRaw = vm.readFile(beefyCheckpointFile);
        uint64 startBlock = uint64(beefyCheckpointRaw.readUint(".startBlock"));

        BeefyClient.ValidatorSet memory current = BeefyClient.ValidatorSet(
            uint128(beefyCheckpointRaw.readUint(".current.id")),
            uint128(beefyCheckpointRaw.readUint(".current.length")),
            beefyCheckpointRaw.readBytes32(".current.root")
        );
        BeefyClient.ValidatorSet memory next = BeefyClient.ValidatorSet(
            uint128(beefyCheckpointRaw.readUint(".next.id")),
            uint128(beefyCheckpointRaw.readUint(".next.length")),
            beefyCheckpointRaw.readBytes32(".next.root")
        );

        randaoCommitDelay = uint8(vm.envOr("RANDAO_COMMIT_DELAY", uint256(3)));
        randaoCommitExpiration = uint8(vm.envOr("RANDAO_COMMIT_EXP", uint256(8)));
        minNumRequiredSignatures = uint8(vm.envOr("MINIMUM_REQUIRED_SIGNATURES", uint256(16)));
        prevRandao = uint32(vm.envOr("PREV_RANDAO", uint256(377)));

        beefyClient = new BeefyClientMock(randaoCommitDelay, randaoCommitExpiration, minNumRequiredSignatures);
        beefyClient.initialize_public(startBlock, current, next);
    }

    function testSubmit() public {
        string memory beefyCommitmentFile = string.concat(vm.projectRoot(), "/test/data/initial-commitment.json");

        string memory beefyCommitmentRaw = vm.readFile(beefyCommitmentFile);

        uint32 blockNumber = uint32(beefyCommitmentRaw.readUint(".Commitment.BlockNumber"));
        uint64 validatorSetID = uint32(beefyCommitmentRaw.readUint(".Commitment.ValidatorSetID"));
        bytes32 mmrRoot = beefyCommitmentRaw.readBytes32(".Commitment.Payload[0].Data");

        BeefyClient.PayloadItem[] memory payload = new BeefyClient.PayloadItem[](1);
        payload[0] = BeefyClient.PayloadItem(mmrRootID, abi.encodePacked(mmrRoot));

        BeefyClient.Commitment memory commitment = BeefyClient.Commitment(blockNumber, validatorSetID, payload);

        uint256[] memory bitfield = beefyCommitmentRaw.readUintArray(".Bitfield");

        BeefyClient.ValidatorProof memory proof = BeefyClient.ValidatorProof(
            uint8(beefyCommitmentRaw.readUint(".Proof.V")),
            beefyCommitmentRaw.readBytes32(".Proof.R"),
            beefyCommitmentRaw.readBytes32(".Proof.S"),
            beefyCommitmentRaw.readUint(".Proof.Index"),
            beefyCommitmentRaw.readAddress(".Proof.Account"),
            beefyCommitmentRaw.readBytes32Array(".Proof.Proof")
        );

        beefyClient.submitInitial(commitment, bitfield, proof);

        // // mine random delay blocks
        // vm.roll(block.number + randaoCommitDelay);

        // vm.prevrandao(bytes32(uint256(prevRandao)));
        // beefyClient.commitPrevRandao(commitHash);

        // beefyClient.submitFinal(
        //     commitment, bitfield, finalValidatorProofs, emptyLeaf, emptyLeafProofs, emptyLeafProofOrder
        // );

        // assertEq(beefyClient.latestBeefyBlock(), blockNumber);
        // assertEq(beefyClient.getValidatorCounter(false, finalValidatorProofs[0].index), 1);
        // assertEq(beefyClient.getValidatorCounter(true, finalValidatorProofs[0].index), 0);
    }
}
