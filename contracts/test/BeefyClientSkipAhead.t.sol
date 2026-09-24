// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.34;

import {BeefyClient} from "../src/BeefyClient.sol";
import {BeefyClientTest} from "./BeefyClient.t.sol";

/// @dev Tests for non-consecutive ("skip-ahead") validator set updates. Reuses the fixtures and
/// helpers from BeefyClientTest.
///
/// Fixture facts (test/data): the signed commitment is for validatorSetID == setId, the validator
/// merkle root is `root`, and the MMR leaf attests nextAuthoritySetID == setId + 1 with a
/// *different* root (a genuine era change). So initializing current/next a few sessions behind
/// `setId` (all sharing `root`) turns the same fixture into a skip-ahead within a stable era.
contract BeefyClientSkipAheadTest is BeefyClientTest {
    /// @dev Interactive skip-ahead: a commitment from a later session inside a stable era is
    /// accepted against the current set, fast-forwards the id (root preserved), and loads next
    /// from the leaf.
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

    /// @dev A skip carries the *same* validators forward (canSkipAhead requires
    /// current.root == next.root), so the anti-grinding usage counters must survive it. Clearing
    /// them would refund the escalating signature cost that computeNumRequiredSignatures charges
    /// for repeated submitInitial calls — a discount reachable below quorum, unlike the rest of
    /// the skip path. A handover resets the counters because the membership actually changes.
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

    /// @dev A skip is only safe inside a confirmed-stable era. If a root change is already pending
    /// (current.root != next.root), the skip is ambiguous and must be rejected.
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

    /// @dev An id at or below the next set is not a skip; it stays an ordinary InvalidCommitment.
    function testSkipAheadRejectsIdNotAheadOfNext() public {
        // current = setId, next = setId+1; the fixture commitment (== setId) is the normal current
        // case, but a stale id below current must revert InvalidCommitment.
        BeefyClient.Commitment memory commitment = initialize(setId);
        commitment.validatorSetID = setId - 1;

        vm.expectRevert(BeefyClient.InvalidCommitment.selector);
        beefyClient.submitFiatShamir(
            commitment, bitfield, fiatShamirValidatorProofs, mmrLeaf, mmrLeafProofs, leafProofOrder
        );
    }

    /// @dev applySkip in isolation: fast-forwards current to the verified id (root/length
    /// preserved) and loads next from the leaf.
    /// Driven directly because the Fiat-Shamir submit subsample is keyed by the current set id
    /// (createFiatShamirHash), which differs from the skipped-to id, so the cached Fiat-Shamir
    /// proof fixtures can't be replayed against a skip without regenerating them.
    function testApplySkipAdvancesState() public {
        // current = setId-3, next = setId-2, both sharing `root`.
        initialize(setId - 3);

        // Skip ahead to setId, loading next from the (valid) fixture leaf/proof against mmrRoot.
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

    /// @dev applySkip requires the leaf's nextAuthoritySetID to be exactly the skipped-to
    /// commitment id + 1, else InvalidMMRLeaf. Driven through the real
    /// interactive submitFinal so the gate is exercised end-to-end (its subsample is prevRandao-
    /// seeded, so the cached fixtures replay correctly for a skip).
    function testSkipAheadRevertsWithInvalidMMRLeaf() public {
        BeefyClient.Commitment memory commitment = initialize(setId - 3);

        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[0]);
        vm.roll(block.number + randaoCommitDelay);
        commitPrevRandao();
        createFinalProofs();

        // Skip-ahead gates pass, signatures verify, but the leaf claims a non-increasing next id
        // (== the skipped-to id) — applySkip must reject it before advancing.
        mmrLeaf.nextAuthoritySetID = setId;
        vm.expectRevert(BeefyClient.InvalidMMRLeaf.selector);
        beefyClient.submitFinal(
            commitment, bitfield, finalValidatorProofs, mmrLeaf, mmrLeafProofs, leafProofOrder
        );
    }

    /// @dev applySkip verifies the leaf against the commitment's MMR root; a tampered leaf
    /// (here a wrong parentNumber) must fail with InvalidMMRLeafProof.
    function testSkipAheadRevertsWithInvalidMMRLeafProof() public {
        BeefyClient.Commitment memory commitment = initialize(setId - 3);

        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[0]);
        vm.roll(block.number + randaoCommitDelay);
        commitPrevRandao();
        createFinalProofs();

        // id check passes (leaf.nextAuthoritySetID == setId + 1 > setId), but the corrupted leaf
        // no longer matches the proof.
        mmrLeaf.parentNumber = 1;
        vm.expectRevert(BeefyClient.InvalidMMRLeafProof.selector);
        beefyClient.submitFinal(
            commitment, bitfield, finalValidatorProofs, mmrLeaf, mmrLeafProofs, leafProofOrder
        );
    }

    /// @dev A skip may reach at most `maxSkipAheadSessions` past the current set: one further
    /// is an ordinary InvalidCommitment.
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

    /// @dev Exactly `maxSkipAheadSessions` past the current set is still a valid skip.
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

    /// @dev A ticket opened as a skip checks its quorum against the current set. If a handover
    /// then makes the commitment id resolve to a set of a different length, the claimed bitfield
    /// is no longer a proven quorum of that set, so submitFinal must reject the ticket.
    function testSkipAheadTicketRejectedWhenSetLengthChanges() public {
        BeefyClient.Commitment memory commitment = initialize(setId - 3);
        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[0]);
        vm.roll(block.number + randaoCommitDelay);
        commitPrevRandao();
        createFinalProofs();

        // A handover lands in between: setId is now the next set, with a larger membership.
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

    /// @dev Every leaf from set X announces set X + 1, so a leaf announcing anything later is
    /// not from the skipped-to session and must be rejected.
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
