// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.34;

import {BeefyClient} from "../src/BeefyClient.sol";
import {BeefyClientTest} from "./BeefyClient.t.sol";

/// @dev Skip-ahead tests, reusing BeefyClientTest's fixtures. The fixture commitment is for set
/// `setId` with root `root`, and its leaf announces set `setId + 1` with a different root.
/// Starting current/next a few ids earlier with `root` turns it into a skip.
contract BeefyClientSkipAheadTest is BeefyClientTest {
    /// @dev An interactive skip moves current to the new id, keeps its root, and loads next from
    /// the leaf.
    function testSkipAheadInteractiveAdvances() public {
        // current = setId-3, next = setId-2, both sharing `root`. Commitment is from setId.
        BeefyClient.Commitment memory commitment = initialize(setId - 3);

        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[0]);
        vm.roll(block.number + randaoCommitDelay);
        commitPrevRandao();
        createFinalProofs();
        beefyClient.submitFinal(
            commitment, bitfield, finalValidatorProofs, mmrLeaf, mmrLeafProofs, leafProofOrder
        );

        // MMR root delivered.
        assertEq(beefyClient.latestBeefyBlock(), blockNumber);

        // Current fast-forwarded to the skipped id, membership root preserved.
        (uint128 curId,, bytes32 curRoot,) = beefyClient.currentValidatorSet();
        assertEq(uint256(curId), uint256(setId), "current id advanced to skipped id");
        assertEq(curRoot, root, "current root preserved across skip");

        // Next loaded from the leaf (here a new era => different root).
        (uint128 nextId,, bytes32 nextRoot,) = beefyClient.nextValidatorSet();
        assertEq(uint256(nextId), uint256(mmrLeaf.nextAuthoritySetID), "next id from leaf");
        assertEq(nextRoot, mmrLeaf.nextAuthoritySetRoot, "next root from leaf");
    }

    /// @dev A skip keeps the same validators, so it keeps their usage counters.
    function testSkipAheadPreservesUsageCounters() public {
        // current = setId-3, next = setId-2, both sharing `root`. Commitment is from setId.
        BeefyClient.Commitment memory commitment = initialize(setId - 3);

        // On the skip path submitInitial bumps the *current* set's counters.
        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[1]);
        assertEq(beefyClient.getValidatorCounter(false, finalValidatorProofs[1].index), 1);

        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[0]);
        assertEq(beefyClient.getValidatorCounter(false, finalValidatorProofs[0].index), 1);

        vm.roll(block.number + randaoCommitDelay);
        commitPrevRandao();
        createFinalProofs();
        beefyClient.submitFinal(
            commitment, bitfield, finalValidatorProofs, mmrLeaf, mmrLeafProofs, leafProofOrder
        );
        assertEq(beefyClient.latestBeefyBlock(), blockNumber);

        // Membership is unchanged across the skip, so the counters must still stand.
        assertEq(
            beefyClient.getValidatorCounter(false, finalValidatorProofs[0].index),
            1,
            "skip must not clear current usage counters"
        );
        assertEq(
            beefyClient.getValidatorCounter(false, finalValidatorProofs[1].index),
            1,
            "skip must not clear current usage counters"
        );
    }

    /// @dev No skip while current and next have different roots.
    function testSkipAheadRevertsWhenEraChangePending() public {
        BeefyClient.Commitment memory commitment = initialize(setId - 3);

        // Re-seed so a root change is pending between current and next.
        beefyClient.initialize_public(
            0,
            BeefyClient.ValidatorSet(setId - 3, setSize, root),
            BeefyClient.ValidatorSet(setId - 2, setSize, bytes32(uint256(root) + 1))
        );

        vm.expectRevert(BeefyClient.InvalidCommitment.selector);
        beefyClient.submitFiatShamir(
            commitment, bitfield, fiatShamirValidatorProofs, mmrLeaf, mmrLeafProofs, leafProofOrder
        );
    }

    /// @dev An id at or below the next set is not a skip.
    function testSkipAheadRejectsIdNotAheadOfNext() public {
        // current = setId, next = setId + 1; an id below current is rejected.
        BeefyClient.Commitment memory commitment = initialize(setId);
        commitment.validatorSetID = setId - 1;

        vm.expectRevert(BeefyClient.InvalidCommitment.selector);
        beefyClient.submitFiatShamir(
            commitment, bitfield, fiatShamirValidatorProofs, mmrLeaf, mmrLeafProofs, leafProofOrder
        );
    }

    /// @dev applySkip on its own. The Fiat-Shamir fixtures were generated for set `setId`, but a
    /// skip seeds with the current id, so they can't be replayed through a skip.
    function testApplySkipAdvancesState() public {
        // current = setId-3, next = setId-2, both sharing `root`.
        initialize(setId - 3);

        // Skip to setId with the fixture leaf and proof.
        beefyClient.applySkip_public(setId, mmrRoot, mmrLeaf, mmrLeafProofs, leafProofOrder);

        // Current fast-forwarded to the skipped id; membership root and length preserved.
        (uint128 curId, uint128 curLen, bytes32 curRoot,) = beefyClient.currentValidatorSet();
        assertEq(uint256(curId), uint256(setId), "current id advanced to skipped id");
        assertEq(curRoot, root, "current root preserved across skip");
        assertEq(uint256(curLen), uint256(setSize), "current length preserved across skip");

        // Next loaded from the leaf (here a new era => different root).
        (uint128 nextId,, bytes32 nextRoot,) = beefyClient.nextValidatorSet();
        assertEq(uint256(nextId), uint256(mmrLeaf.nextAuthoritySetID), "next id from leaf");
        assertEq(nextRoot, mmrLeaf.nextAuthoritySetRoot, "next root from leaf");
    }

    /// @dev The leaf must announce the skipped-to id + 1.
    function testSkipAheadRevertsWithInvalidMMRLeaf() public {
        BeefyClient.Commitment memory commitment = initialize(setId - 3);

        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[0]);
        vm.roll(block.number + randaoCommitDelay);
        commitPrevRandao();
        createFinalProofs();

        // Signatures verify, but the leaf announces the skipped-to id itself.
        mmrLeaf.nextAuthoritySetID = setId;
        vm.expectRevert(BeefyClient.InvalidMMRLeaf.selector);
        beefyClient.submitFinal(
            commitment, bitfield, finalValidatorProofs, mmrLeaf, mmrLeafProofs, leafProofOrder
        );
    }

    /// @dev A tampered leaf fails the MMR proof.
    function testSkipAheadRevertsWithInvalidMMRLeafProof() public {
        BeefyClient.Commitment memory commitment = initialize(setId - 3);

        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[0]);
        vm.roll(block.number + randaoCommitDelay);
        commitPrevRandao();
        createFinalProofs();

        // The id check passes, but the leaf no longer matches the proof.
        mmrLeaf.parentNumber = 1;
        vm.expectRevert(BeefyClient.InvalidMMRLeafProof.selector);
        beefyClient.submitFinal(
            commitment, bitfield, finalValidatorProofs, mmrLeaf, mmrLeafProofs, leafProofOrder
        );
    }

    /// @dev One id past `maxSkipAheadSessions` is rejected.
    function testSkipAheadRejectsIdBeyondMaxDistance() public {
        uint32 maxSkip = uint32(beefyClient.maxSkipAheadSessions());
        BeefyClient.Commitment memory commitment = initialize(setId - maxSkip - 1);

        vm.expectRevert(BeefyClient.InvalidCommitment.selector);
        beefyClient.submitFiatShamir(
            commitment, bitfield, fiatShamirValidatorProofs, mmrLeaf, mmrLeafProofs, leafProofOrder
        );

        vm.expectRevert(BeefyClient.InvalidCommitment.selector);
        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[0]);
    }

    /// @dev Exactly `maxSkipAheadSessions` ahead is accepted.
    function testSkipAheadAcceptsIdAtMaxDistance() public {
        uint32 maxSkip = uint32(beefyClient.maxSkipAheadSessions());
        BeefyClient.Commitment memory commitment = initialize(setId - maxSkip);

        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[0]);
        vm.roll(block.number + randaoCommitDelay);
        commitPrevRandao();
        createFinalProofs();
        beefyClient.submitFinal(
            commitment, bitfield, finalValidatorProofs, mmrLeaf, mmrLeafProofs, leafProofOrder
        );

        (uint128 curId,,,) = beefyClient.currentValidatorSet();
        assertEq(uint256(curId), uint256(setId), "current id advanced to skipped id");
    }

    /// @dev A skip ticket is rejected if a handover then maps its id to a set of another length.
    function testSkipAheadTicketRejectedWhenSetLengthChanges() public {
        BeefyClient.Commitment memory commitment = initialize(setId - 3);
        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[0]);
        vm.roll(block.number + randaoCommitDelay);
        commitPrevRandao();
        createFinalProofs();

        // A handover lands: setId is now the next set, one validator larger.
        beefyClient.initialize_public(
            0,
            BeefyClient.ValidatorSet(setId - 1, setSize, root),
            BeefyClient.ValidatorSet(setId, setSize + 1, root)
        );

        vm.expectRevert(BeefyClient.InvalidTicket.selector);
        beefyClient.submitFinal(
            commitment, bitfield, finalValidatorProofs, mmrLeaf, mmrLeafProofs, leafProofOrder
        );
    }

    /// @dev A leaf announcing X + 2 is not from set X.
    function testSkipAheadRevertsWithLeafFromLaterSession() public {
        BeefyClient.Commitment memory commitment = initialize(setId - 3);

        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[0]);
        vm.roll(block.number + randaoCommitDelay);
        commitPrevRandao();
        createFinalProofs();

        mmrLeaf.nextAuthoritySetID = setId + 2;
        vm.expectRevert(BeefyClient.InvalidMMRLeaf.selector);
        beefyClient.submitFinal(
            commitment, bitfield, finalValidatorProofs, mmrLeaf, mmrLeafProofs, leafProofOrder
        );
    }
}
