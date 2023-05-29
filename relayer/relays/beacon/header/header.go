package header

import (
	"context"
	"errors"
	"fmt"
	"time"

	"github.com/snowfork/snowbridge/relayer/relays/beacon/config"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/scale"

	log "github.com/sirupsen/logrus"
	"github.com/snowfork/snowbridge/relayer/chain/parachain"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/cache"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/state"
	"golang.org/x/sync/errgroup"
)

var ErrFinalizedHeaderUnchanged = errors.New("finalized header unchanged")
var ErrFinalizedHeaderNotImported = errors.New("finalized header not imported")
var ErrSyncCommitteeNotImported = errors.New("sync committee header not imported")
var ErrSyncCommitteeLatency = errors.New("sync committee latency found")

type Header struct {
	cache  *cache.BeaconCache
	writer *parachain.ParachainWriter
	syncer *syncer.Syncer
}

func New(writer *parachain.ParachainWriter, beaconEndpoint string, slotsInEpoch, epochsPerSyncCommitteePeriod uint64, maxSlotsPerHistoricalRoot int, activeSpec config.ActiveSpec) Header {
	return Header{
		cache:  cache.New(slotsInEpoch, epochsPerSyncCommitteePeriod),
		writer: writer,
		syncer: syncer.New(beaconEndpoint, slotsInEpoch, epochsPerSyncCommitteePeriod, maxSlotsPerHistoricalRoot, activeSpec),
	}
}

func (h *Header) Sync(ctx context.Context, eg *errgroup.Group) error {
	lastFinalizedHeaderState, err := h.writer.GetLastFinalizedHeaderState()
	if err != nil {
		return fmt.Errorf("fetch parachain last finalized header state: %w", err)
	}

	latestSyncedPeriod := h.syncer.ComputeSyncPeriodAtSlot(lastFinalizedHeaderState.BeaconSlot)
	log.WithFields(log.Fields{
		"hash":   lastFinalizedHeaderState.BeaconBlockRoot,
		"slot":   lastFinalizedHeaderState.BeaconSlot,
		"period": latestSyncedPeriod,
	}).Info("set cache: last finalized state")
	h.cache.SetLastSyncedSyncCommitteePeriod(latestSyncedPeriod)
	h.cache.SetLastSyncedFinalizedState(lastFinalizedHeaderState.BeaconBlockRoot, lastFinalizedHeaderState.BeaconSlot)
	h.cache.SetInitialCheckpointSlot(lastFinalizedHeaderState.InitialCheckpointSlot)
	h.cache.AddCheckPointSlots([]uint64{lastFinalizedHeaderState.BeaconSlot})

	// syncLaggingExecutionHeaders so to allow ExecutionHeader to catch up
	executionHeaderState, err := h.writer.GetLastExecutionHeaderState()
	if err != nil {
		return fmt.Errorf("fetch last execution hash: %w", err)
	}
	log.WithFields(log.Fields{
		"hash": executionHeaderState.BeaconBlockRoot,
		"from": executionHeaderState.BeaconSlot,
		"to":   lastFinalizedHeaderState.BeaconSlot,
	}).Info("starting to sync from last execution state to last finalized state")
	err = h.syncLaggingExecutionHeaders(ctx, lastFinalizedHeaderState, executionHeaderState)
	if err != nil {
		return fmt.Errorf("sync lagging execution headers: %w", err)
	}

	log.Info("starting to sync finalized headers")

	ticker := time.NewTicker(time.Second * 10)

	eg.Go(func() error {
		for {
			err = h.SyncHeadersFromFinalized(ctx)
			logFields := log.Fields{
				"finalized_header": h.cache.Finalized.LastSyncedHash,
				"slot":             h.cache.Finalized.LastSyncedSlot,
			}
			switch {
			case errors.Is(err, ErrFinalizedHeaderUnchanged):
				log.WithFields(logFields).Info("not importing unchanged header")
			case errors.Is(err, syncer.ErrBeaconStateAvailableYet):
				log.WithFields(logFields).WithError(err).Warn("beacon state not available for finalized state yet")
			case err != nil:
				return err
			}

			select {
			case <-ctx.Done():
				return nil
			case <-ticker.C:
				continue
			}
		}
	})

	return nil
}

func (h *Header) SyncCommitteePeriodUpdate(ctx context.Context, period uint64) error {
	update, err := h.syncer.GetSyncCommitteePeriodUpdate(period)

	switch {
	case errors.Is(err, syncer.ErrCommitteeUpdateHeaderInDifferentSyncPeriod):
		{
			log.WithField("period", period).Info("committee update and header in different sync periods, skipping")

			return err
		}
	case err != nil:
		{
			return fmt.Errorf("fetch sync committee period update: %w", err)
		}
	}

	log.WithFields(log.Fields{
		"finalized_header_slot": update.Payload.FinalizedHeader.Slot,
		"period":                period,
	}).Info("syncing sync committee for period")

	err = h.writer.WriteToParachainAndWatch(ctx, "EthereumBeaconClient.submit", update.Payload)
	if err != nil {
		return err
	}

	lastFinalizedHeaderState, err := h.writer.GetLastFinalizedHeaderState()
	if err != nil {
		return fmt.Errorf("fetch last finalized header state: %w", err)
	}

	lastUpdatedPeriod := h.syncer.ComputeSyncPeriodAtSlot(lastFinalizedHeaderState.BeaconSlot)

	if period != lastUpdatedPeriod {
		return ErrSyncCommitteeNotImported
	}

	h.cache.SetLastSyncedSyncCommitteePeriod(period)
	h.cache.SetLastSyncedFinalizedState(update.FinalizedHeaderBlockRoot, uint64(update.Payload.FinalizedHeader.Slot))
	h.cache.AddCheckPoint(update.FinalizedHeaderBlockRoot, update.BlockRootsTree, uint64(update.Payload.FinalizedHeader.Slot))

	return nil
}

func (h *Header) SyncFinalizedHeader(ctx context.Context) error {
	// When the chain has been processed up until now, keep getting finalized block updates and send that to the parachain
	update, err := h.syncer.GetFinalizedUpdate()
	if err != nil {
		return fmt.Errorf("fetch finalized header update: %w", err)
	}

	log.WithFields(log.Fields{
		"slot":      update.Payload.FinalizedHeader.Slot,
		"blockRoot": update.FinalizedHeaderBlockRoot,
	}).Info("syncing finalized header at slot")

	currentSyncPeriod := h.syncer.ComputeSyncPeriodAtSlot(uint64(update.Payload.AttestedHeader.Slot))

	if h.cache.LastSyncedSyncCommitteePeriod < currentSyncPeriod {
		err = h.syncLaggingSyncCommitteePeriods(ctx, h.cache.LastSyncedSyncCommitteePeriod, currentSyncPeriod)
		if err != nil {
			return fmt.Errorf("sync lagging sync committee periods: %w", err)
		}
	}

	err = h.writer.WriteToParachainAndWatch(ctx, "EthereumBeaconClient.submit", update.Payload)
	if err != nil {
		return fmt.Errorf("write to parachain: %w", err)
	}

	lastFinalizedHeaderState, err := h.writer.GetLastFinalizedHeaderState()
	if err != nil {
		return fmt.Errorf("fetch last finalized header state: %w", err)
	}

	lastStoredHeader := lastFinalizedHeaderState.BeaconBlockRoot

	if lastStoredHeader != update.FinalizedHeaderBlockRoot {
		return ErrFinalizedHeaderNotImported
	}

	// If the finalized header import succeeded, we add it to this cache. This cache is used to determine
	// from which last finalized header needs to imported (i.e. start and end finalized blocks, to backfill execution
	// headers in between).
	h.cache.SetLastSyncedFinalizedState(update.FinalizedHeaderBlockRoot, uint64(update.Payload.FinalizedHeader.Slot))
	h.cache.AddCheckPoint(update.FinalizedHeaderBlockRoot, update.BlockRootsTree, uint64(update.Payload.FinalizedHeader.Slot))
	return nil
}

func (h *Header) SyncHeader(ctx context.Context, headerUpdate scale.HeaderUpdate) error {
	blockHash := headerUpdate.Payload.ExecutionHeader.BlockHash.Hex()
	blockNumber := uint64(headerUpdate.Payload.ExecutionHeader.BlockNumber)

	log.WithFields(log.Fields{
		"slot":                 headerUpdate.Payload.Header.Slot,
		"executionBlockRoot":   blockHash,
		"executionBlockNumber": blockNumber,
	}).Info("Syncing header between last two finalized headers")

	err := h.writer.WriteToParachainAndRateLimit(ctx, "EthereumBeaconClient.submit_execution_header", headerUpdate.Payload)
	if err != nil {
		return fmt.Errorf("write to parachain: %w", err)
	}
	h.cache.SetLastSyncedExecutionSlot(uint64(headerUpdate.Payload.Header.Slot))
	return nil
}

func (h *Header) SyncHeadersFromFinalized(ctx context.Context) error {
	hasChanged, err := h.syncer.HasFinalizedHeaderChanged(h.cache.Finalized.LastSyncedHash)
	if err != nil {
		return err
	}

	if !hasChanged {
		return ErrFinalizedHeaderUnchanged
	}

	err = h.SyncFinalizedHeader(ctx)
	if err != nil {
		return err
	}
	fromSlot := h.cache.LastSyncedExecutionSlot
	if fromSlot <= h.cache.InitialCheckpointSlot {
		fromSlot = h.cache.InitialCheckpointSlot
	}

	err = h.SyncHeaders(ctx, fromSlot, h.cache.Finalized.LastSyncedSlot)

	if err != nil {
		return err
	}

	return nil
}

func (h *Header) SyncHeaders(ctx context.Context, fromSlot, toSlot uint64) error {

	log.WithFields(log.Fields{
		"fromSlot":   fromSlot,
		"fromEpoch":  h.syncer.ComputeEpochAtSlot(fromSlot),
		"toSlot":     toSlot,
		"toEpoch":    h.syncer.ComputeEpochAtSlot(toSlot),
		"totalSlots": toSlot - fromSlot,
	}).Info("starting to back-fill headers")

	var headersToSync []scale.HeaderUpdate

	// start syncing at next block after last synced block
	currentSlot := fromSlot
	headerUpdate, err := h.getNextHeaderUpdateBySlotWithAncestryProof(currentSlot)
	if err != nil {
		return fmt.Errorf("get next header update by slot with ancestry proof: %w", err)
	}

	for currentSlot <= toSlot {
		log.WithFields(log.Fields{
			"currentSlot": currentSlot,
		}).Info("fetching next header at slot")

		var nextHeaderUpdate scale.HeaderUpdate
		// If this is the last slot we need to sync, don't fetch the ancestry proof for the next slot
		// because its finalized header won't be synced yet. We still need to fetch the next block for the
		// sync aggregate though.
		if currentSlot >= toSlot {
			nextHeaderUpdate, err = h.getNextHeaderUpdateBySlot(currentSlot)
			if err != nil {
				return fmt.Errorf("get next header update by slot: %w", err)
			}
		} else {
			// To get the sync witness for the current synced header. This header
			// will be used as the next update.
			nextHeaderUpdate, err = h.getNextHeaderUpdateBySlotWithAncestryProof(currentSlot)
			if err != nil {
				return fmt.Errorf("get next header update by slot with ancestry proof: %w", err)
			}
		}

		headersToSync = append(headersToSync, headerUpdate)
		headerUpdate = nextHeaderUpdate

		// last slot to be synced, sync headers
		if currentSlot >= toSlot {
			for _, header := range headersToSync {
				err := h.SyncHeader(ctx, header)
				if err != nil {
					return err
				}
			}
		}
		currentSlot = uint64(headerUpdate.Payload.Header.Slot)
	}

	return nil
}

// Syncs execution headers from the last synced execution header on the parachain to the current finalized header. Lagging execution headers can occur if the relayer
// stopped while still processing a set of execution headers.
func (h *Header) syncLaggingExecutionHeaders(ctx context.Context, lastFinalizedState state.FinalizedHeader, executionHeaderState state.ExecutionHeader) error {
	fromSlot := executionHeaderState.BeaconSlot
	if fromSlot == 0 {
		fromSlot = lastFinalizedState.InitialCheckpointSlot
	}

	lastFinalizedSlot := lastFinalizedState.BeaconSlot

	if fromSlot >= lastFinalizedSlot {
		log.WithFields(log.Fields{
			"slot":          executionHeaderState.BeaconSlot,
			"blockNumber":   executionHeaderState.BlockNumber,
			"executionHash": executionHeaderState.BlockHash,
			"fromSlot":      fromSlot,
		}).Info("execution headers sync up to date with last finalized header")

		return nil
	}

	log.WithFields(log.Fields{
		"executionSlot": executionHeaderState.BeaconSlot,
		"finalizedSlot": lastFinalizedSlot,
		"blockNumber":   executionHeaderState.BlockNumber,
		"executionHash": executionHeaderState.BlockHash,
		"finalizedHash": lastFinalizedState.BeaconBlockRoot,
		"fromSlot":      fromSlot,
		"slotsBacklog":  lastFinalizedSlot - fromSlot,
	}).Info("execution headers sync is not up to date with last finalized header, syncing lagging execution headers")

	err := h.SyncHeaders(ctx, fromSlot, lastFinalizedState.BeaconSlot)
	if err != nil {
		return fmt.Errorf("sync headers: %w", err)
	}

	return nil
}

func (h *Header) syncLaggingSyncCommitteePeriods(ctx context.Context, latestSyncedPeriod, currentSyncPeriod uint64) error {
	periodsToSync, err := h.syncer.GetSyncPeriodsToFetch(latestSyncedPeriod, currentSyncPeriod)
	if err != nil {
		return fmt.Errorf("check sync committee periods to be fetched: %w", err)
	}
	// initialized with latestSyncedPeriod to sync next sync committee
	initialPeriod := h.syncer.ComputeSyncPeriodAtSlot(h.cache.InitialCheckpointSlot)
	lastFinalizedPeriod := h.syncer.ComputeSyncPeriodAtSlot(h.cache.Finalized.LastSyncedSlot)
	if initialPeriod == lastFinalizedPeriod {
		periodsToSync = append([]uint64{latestSyncedPeriod}, periodsToSync...)
	}

	log.WithFields(log.Fields{
		"periods": periodsToSync,
	}).Info("sync committee periods to be synced")

	for _, period := range periodsToSync {
		err := h.SyncCommitteePeriodUpdate(ctx, period)
		if err != nil {
			return err
		}
	}

	if h.cache.LastSyncedSyncCommitteePeriod < currentSyncPeriod {
		return ErrSyncCommitteeLatency
	}

	return nil
}

func (h *Header) populateFinalizedCheckpoint(slot uint64) error {
	finalizedHeader, err := h.syncer.Client.GetHeaderBySlot(slot) // TODO if slot empty get previous slot
	if err != nil {
		return fmt.Errorf("get header by slot: %w", err)
	}

	scaleHeader, err := finalizedHeader.ToScale()
	if err != nil {
		return fmt.Errorf("header to scale: %w", err)
	}

	blockRoot, err := scaleHeader.ToSSZ().HashTreeRoot()
	if err != nil {
		return fmt.Errorf("header hash root: %w", err)
	}

	// Always check slot finalized on chain before populating checkpoint
	onChainFinalizedHeader, err := h.writer.GetFinalizedHeaderStateByBlockRoot(blockRoot)
	if err != nil {
		return err
	}
	if onChainFinalizedHeader.BeaconSlot != slot {
		return fmt.Errorf("on chain finalized header inconsistent at slot %d", slot)
	}

	blockRootsProof, err := h.syncer.GetBlockRoots(slot)
	if err != nil && !errors.Is(err, syncer.ErrBeaconStateAvailableYet) {
		return fmt.Errorf("fetch block roots: %w", err)
	}

	log.Info("populating checkpoint")

	h.cache.AddCheckPoint(blockRoot, blockRootsProof.Tree, slot)

	return nil
}

func (h *Header) getClosestCheckpoint(slot uint64) (cache.Proof, error) {
	checkpoint, err := h.cache.GetClosestCheckpoint(slot)

	switch {
	case errors.Is(cache.FinalizedCheckPointNotAvailable, err) || errors.Is(cache.FinalizedCheckPointNotPopulated, err):
		checkpointSlot := checkpoint.Slot
		if checkpointSlot == 0 {
			checkpointSlot = h.syncer.CalculateNextCheckpointSlot(slot)
			log.WithFields(log.Fields{"calculatedCheckpointSlot": checkpointSlot}).Info("checkpoint slot not available")
		}
		err := h.populateFinalizedCheckpoint(checkpointSlot)
		if err != nil {
			return cache.Proof{}, fmt.Errorf("populate closest checkpoint: %w", err)
		}

		log.Info("populated finalized checkpoint")

		checkpoint, err = h.cache.GetClosestCheckpoint(slot)
		if err != nil {
			return cache.Proof{}, fmt.Errorf("get closest checkpoint after populating finalized header: %w", err)
		}

		log.WithFields(log.Fields{"slot": slot, "checkpoint": checkpoint}).Info("checkpoint after populating finalized header")

		return checkpoint, nil
	case err != nil:
		return cache.Proof{}, fmt.Errorf("get closest checkpoint: %w", err)
	}

	return checkpoint, nil
}

func (h *Header) getNextHeaderUpdateBySlotWithAncestryProof(slot uint64) (scale.HeaderUpdate, error) {
	slot = slot + 1
	checkpoint, err := h.getClosestCheckpoint(slot)
	if err != nil {
		return scale.HeaderUpdate{}, fmt.Errorf("get closest checkpoint: %w", err)
	}
	return h.syncer.GetNextHeaderUpdateBySlotWithAncestryProof(slot, &checkpoint)
}

func (h *Header) getNextHeaderUpdateBySlot(slot uint64) (scale.HeaderUpdate, error) {
	slot = slot + 1
	return h.syncer.GetNextHeaderUpdateBySlotWithAncestryProof(slot, nil)
}
