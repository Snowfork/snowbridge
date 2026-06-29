// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.34;

import {BeefyClient} from "../src/BeefyClient.sol";
import {BeefyClientTest} from "./BeefyClient.t.sol";

/// @dev Tests for non-consecutive ("skip-ahead") validator set updates, gated by an
/// Ethereum-clock trusting period. Reuses the fixtures and helpers from BeefyClientTest.
///
/// Fixture facts (test/data): the signed commitment is for validatorSetID == setId, the validator
/// merkle root is `root`, and the MMR leaf attests nextAuthoritySetID == setId + 1 with a
/// *different* root (a genuine era change). So initializing current/next a few sessions behind
/// `setId` (all sharing `root`) turns the same fixture into a skip-ahead within a stable era.
contract BeefyClientSkipAheadTest is BeefyClientTest {
    /// @dev Interactive skip-ahead: a commitment from a later session inside a stable era is
    /// accepted against the current set, fast-forwards the id (root preserved), loads next from
    /// the leaf — and must NOT refresh the trusting-window anchor (no ratchet).
    function testSkipAheadInteractiveAdvancesWithoutRatchet() public {
        // current = setId-3, next = setId-2, both sharing `root`. Commitment is from setId.
        BeefyClient.Commitment memory commitment = initialize(setId - 3);
        uint64 anchorBefore = beefyClient.currentSetActivatedAt();

        // Advance wall-clock (still well within the window) so that an (incorrect) re-anchor
        // would be observably different from `anchorBefore`.
        vm.warp(block.timestamp + 1 hours);

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

        // No ratchet: a skip must not move the anchor forward.
        assertEq(
            beefyClient.currentSetActivatedAt(), anchorBefore, "skip must not re-anchor window"
        );
    }

    /// @dev Once the current set is older than the trusting period it may be unbonded, so a skip
    /// must be refused with a precise TrustingPeriodExpired signal (Fiat-Shamir path).
    function testSkipAheadRevertsWhenTrustingPeriodExpiredFiatShamir() public {
        BeefyClient.Commitment memory commitment = initialize(setId - 3);
        uint64 anchor = beefyClient.currentSetActivatedAt();

        // Jump just past the window.
        vm.warp(uint256(anchor) + beefyClient.trustingPeriod() + 1);

        vm.expectRevert(BeefyClient.TrustingPeriodExpired.selector);
        beefyClient.submitFiatShamir(
            commitment, bitfield, fiatShamirValidatorProofs, mmrLeaf, mmrLeafProofs, leafProofOrder
        );
    }

    /// @dev Same gate on the interactive path (submitInitial).
    function testSkipAheadRevertsWhenTrustingPeriodExpiredInteractive() public {
        BeefyClient.Commitment memory commitment = initialize(setId - 3);
        uint64 anchor = beefyClient.currentSetActivatedAt();

        vm.warp(uint256(anchor) + beefyClient.trustingPeriod() + 1);

        vm.expectRevert(BeefyClient.TrustingPeriodExpired.selector);
        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[0]);
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

    /// @dev A genuine (consecutive) handover is fresh evidence the new set is active, so it MUST
    /// re-anchor the trusting window — the mechanism that keeps skips available across long eras.
    function testHandoverReanchorsTrustingWindow() public {
        BeefyClient.Commitment memory commitment = initialize(setId - 1); // handover path
        uint64 anchorBefore = beefyClient.currentSetActivatedAt();

        vm.warp(block.timestamp + 2 hours);
        beefyClient.submitFiatShamir(
            commitment, bitfield, fiatShamirValidatorProofs, mmrLeaf, mmrLeafProofs, leafProofOrder
        );

        assertEq(beefyClient.latestBeefyBlock(), blockNumber);
        assertEq(
            beefyClient.currentSetActivatedAt(),
            uint64(block.timestamp),
            "handover re-anchors to now"
        );
        assertGt(beefyClient.currentSetActivatedAt(), anchorBefore, "anchor moved forward");
    }
}
