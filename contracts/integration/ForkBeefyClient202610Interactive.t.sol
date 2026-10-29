// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.34;

import {BeefyClient} from "../src/BeefyClient.sol";
import {ForkBeefyClient202610Base} from "./ForkBeefyClient202610Base.sol";

/// @notice The mainnet submitInitial / commitPrevRandao / submitFinal sequence for relay block
///         33,599,510, replayed against the new BeefyClient with the same commitment, bitfield,
///         MMR leaf and PREVRANDAO.
contract ForkBeefyClient202610InteractiveTest is ForkBeefyClient202610Base {
    // Block of submitFinal tx 0xc02ad755…, so the production ticket still holds the captured
    // PREVRANDAO.
    uint256 private constant SUBMIT_FINAL_BLOCK = 26_276_737;

    struct Replay {
        BeefyClient.Commitment commitment;
        uint256[] bitfield;
        BeefyClient.ValidatorProof initialProof;
        FinalArgs original;
        BeefyClient.ValidatorProof[] signers;
        bytes32 commitmentHash;
    }

    BeefyClient private client;
    uint256 private prevRandao;
    uint256 private randaoDelay;

    function setUp() public {
        forkAt(SUBMIT_FINAL_BLOCK);
        client = deployCheckpointedClient();

        Replay memory r = load();
        (uint64 ticketBlock,, uint32 required, uint256 randao,) =
            BeefyClient(OLD_BEEFY_CLIENT).tickets(ticketID(RELAYER, r.commitmentHash));
        assertEq(required, 28);
        assertGt(randao, 0);
        prevRandao = randao;
        // commitPrevRandao landed one block before submitFinal.
        randaoDelay = block.number - 1 - ticketBlock;
        assertGe(randaoDelay, RANDAO_COMMIT_DELAY);
        assertLe(randaoDelay, RANDAO_COMMIT_DELAY + RANDAO_COMMIT_EXPIRATION);
    }

    function load() private view returns (Replay memory r) {
        (r.commitment, r.bitfield, r.initialProof) = abi.decode(
            readFixture("tx-int-submitInitial.hex"),
            (BeefyClient.Commitment, uint256[], BeefyClient.ValidatorProof)
        );
        r.original = decodeFinalArgs(readFixture("tx-int-submitFinal.hex"));
        r.signers = decodeFinalArgs(readFixture("beefy-33599510-all-signers.hex")).proofs;
        r.commitmentHash = client.computeCommitmentHash(r.commitment);
        assertEq(r.original.proofs.length, 28);
    }

    function submitFinal(Replay memory r, BeefyClient.ValidatorProof[] memory proofs) private {
        vm.prank(RELAYER);
        client.submitFinal(
            r.commitment,
            r.bitfield,
            proofs,
            r.original.leaf,
            r.original.leafProof,
            r.original.leafProofOrder
        );
    }

    /// @dev The original proofs, plus the block's stored signatures for the picks they lack.
    function completeProofs(Replay memory r, uint256[] memory finalBitfield)
        private
        pure
        returns (BeefyClient.ValidatorProof[] memory proofs)
    {
        BeefyClient.ValidatorProof[] memory original =
            selectProofs(r.original.proofs, finalBitfield);
        uint256[] memory missing = new uint256[](finalBitfield.length);
        for (uint256 i = 0; i < finalBitfield.length; i++) {
            missing[i] = finalBitfield[i];
        }
        for (uint256 i = 0; i < original.length; i++) {
            missing[original[i].index / 256] &= ~(1 << (original[i].index % 256));
        }
        BeefyClient.ValidatorProof[] memory added = selectProofs(r.signers, missing);

        proofs = new BeefyClient.ValidatorProof[](original.length + added.length);
        for (uint256 i = 0; i < original.length; i++) {
            proofs[i] = original[i];
        }
        for (uint256 i = 0; i < added.length; i++) {
            proofs[original.length + i] = added[i];
        }
    }

    /// The new minimum gives 20 + log2(600) + 1 = 31 signatures.
    function testSubmitFinalNeeds31Signatures() public {
        Replay memory r = load();

        vm.prank(RELAYER);
        client.submitInitial(r.commitment, r.bitfield, r.initialProof);
        (,, uint32 required,,) = client.tickets(ticketID(RELAYER, r.commitmentHash));
        assertEq(required, 31);

        vm.roll(block.number + randaoDelay);
        vm.prevrandao(bytes32(prevRandao));
        vm.prank(RELAYER);
        client.commitPrevRandao(r.commitmentHash);

        // Same seed and bitfield as mainnet, so the original 28 proofs are the first picks.
        vm.prank(RELAYER);
        uint256[] memory finalBitfield = client.createFinalBitfield(r.commitmentHash, r.bitfield);
        assertEq(countSet(finalBitfield), 31);
        assertEq(selectProofs(r.original.proofs, finalBitfield).length, 28);
        vm.expectRevert(BeefyClient.InvalidValidatorProofLength.selector);
        submitFinal(r, r.original.proofs);

        BeefyClient.ValidatorProof[] memory proofs = completeProofs(r, finalBitfield);
        assertEq(proofs.length, 31);
        uint256 gasBefore = gasleft();
        submitFinal(r, proofs);
        emit log_named_uint("submitFinal gas (31 proofs)", gasBefore - gasleft());
        assertEq(client.latestBeefyBlock(), r.commitment.blockNumber);
    }

    /// Reusing the initial validator signature in a second ticket raises the requirement.
    function testSecondUseNeeds33Signatures() public {
        Replay memory r = load();

        vm.prank(makeAddr("first-relayer"));
        client.submitInitial(r.commitment, r.bitfield, r.initialProof);
        vm.prank(RELAYER);
        client.submitInitial(r.commitment, r.bitfield, r.initialProof);
        (,, uint32 required,,) = client.tickets(ticketID(RELAYER, r.commitmentHash));
        assertEq(required, 33);

        vm.roll(block.number + randaoDelay);
        vm.prevrandao(bytes32(prevRandao));
        vm.prank(RELAYER);
        client.commitPrevRandao(r.commitmentHash);

        vm.prank(RELAYER);
        uint256[] memory finalBitfield = client.createFinalBitfield(r.commitmentHash, r.bitfield);
        BeefyClient.ValidatorProof[] memory proofs = completeProofs(r, finalBitfield);
        assertEq(proofs.length, 33);
        submitFinal(r, proofs);
        assertEq(client.latestBeefyBlock(), r.commitment.blockNumber);
    }
}
