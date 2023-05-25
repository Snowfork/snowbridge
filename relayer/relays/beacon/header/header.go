package header

import (
	"context"
	"errors"
	"fmt"
	"time"

	"github.com/snowfork/snowbridge/relayer/relays/beacon/config"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/scale"

	"github.com/ethereum/go-ethereum/common"
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

	h.cache.SetLastSyncedSyncCommitteePeriod(latestSyncedPeriod)

	log.WithField("period", latestSyncedPeriod).Info("set cache: last beacon synced sync committee period")

	h.cache.AddCheckPointSlots([]uint64{lastFinalizedHeaderState.BeaconSlot})

	log.WithField("finalizedSlots", h.cache.Finalized.Checkpoints.Slots).Info("set cache: finalized checkpoint slots")

	currentFinalizedHeader, err := h.syncer.GetLatestFinalizedHeader()
	if err != nil {
		return fmt.Errorf("fetch last finalized header state from beacon node: %w", err)
	}

	lastFinalizedHeader := lastFinalizedHeaderState.BeaconBlockRoot
	lastFinalizedSlot := lastFinalizedHeaderState.BeaconSlot
	h.cache.Finalized.LastAttemptedSyncHash = lastFinalizedHeader
	h.cache.Finalized.LastAttemptedSyncSlot = lastFinalizedSlot

	log.WithFields(log.Fields{
		"hash": lastFinalizedHeader,
		"slot": lastFinalizedSlot,
	}).Info("set cache: last finalized header")

	executionHeaderState, err := h.writer.GetLastExecutionHeaderState()
	if err != nil {
		return fmt.Errorf("fetch last execution hash: %w", err)
	}

	// Move syncLaggingExecutionHeaders before syncLaggingSyncCommitteePeriods so to allow ExecutionHeader to catch up
	err = h.syncLaggingExecutionHeaders(ctx, lastFinalizedHeader, lastFinalizedSlot, executionHeaderState)
	if err != nil {
		return fmt.Errorf("sync lagging execution headers: %w", err)
	}

	err = h.syncLaggingSyncCommitteePeriods(ctx, latestSyncedPeriod, uint64(currentFinalizedHeader.Payload.FinalizedHeader.Slot))
	if err != nil {
		return fmt.Errorf("sync lagging sync committee periods: %w", err)
	}

	h.cache.Finalized.LastSyncedHash = lastFinalizedHeader

	log.Info("starting to sync finalized headers")

	ticker := time.NewTicker(time.Second * 10)

	eg.Go(func() error {
		for {
			err := h.SyncHeadersFromFinalized(ctx)
			logFields := log.Fields{
				"finalized_header": h.cache.Finalized.LastAttemptedSyncHash,
				"slot":             h.cache.Finalized.LastAttemptedSyncSlot,
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
	h.cache.Finalized.LastSyncedHash = update.FinalizedHeaderBlockRoot
	h.cache.Finalized.LastSyncedSlot = uint64(update.Payload.FinalizedHeader.Slot)

	h.cache.AddCheckPoint(update.FinalizedHeaderBlockRoot, update.BlockRootsTree, uint64(update.Payload.FinalizedHeader.Slot))

	return nil
}

func (h *Header) SyncFinalizedHeader(ctx context.Context) (scale.Update, error) {
	// When the chain has been processed up until now, keep getting finalized block updates and send that to the parachain
	update, err := h.syncer.GetFinalizedUpdate()
	if err != nil {
		return scale.Update{}, fmt.Errorf("fetch finalized header update: %w", err)
	}

	log.WithFields(log.Fields{
		"slot":      update.Payload.FinalizedHeader.Slot,
		"blockRoot": update.FinalizedHeaderBlockRoot,
	}).Info("syncing finalized header at slot")

	currentSyncPeriod := h.syncer.ComputeSyncPeriodAtSlot(uint64(update.Payload.AttestedHeader.Slot))

	if h.cache.LastSyncedSyncCommitteePeriod < currentSyncPeriod {
		err = h.syncLaggingSyncCommitteePeriods(ctx, h.cache.LastSyncedSyncCommitteePeriod, uint64(update.Payload.AttestedHeader.Slot))
		if err != nil {
			return scale.Update{}, fmt.Errorf("sync lagging sync committee periods: %w", err)
		}
	}

	err = h.writer.WriteToParachainAndWatch(ctx, "EthereumBeaconClient.submit", update.Payload)
	if err != nil {
		return scale.Update{}, fmt.Errorf("write to parachain: %w", err)
	}

	// We need a distinction between finalized headers that we've tried to sync, so that we don't try syncing
	// it over and over again with the same failure.
	h.cache.Finalized.LastAttemptedSyncHash = update.FinalizedHeaderBlockRoot
	h.cache.Finalized.LastAttemptedSyncSlot = uint64(update.Payload.FinalizedHeader.Slot)

	lastFinalizedHeaderState, err := h.writer.GetLastFinalizedHeaderState()
	if err != nil {
		return scale.Update{}, fmt.Errorf("fetch last finalized header state: %w", err)
	}

	lastStoredHeader := lastFinalizedHeaderState.BeaconBlockRoot

	if lastStoredHeader != update.FinalizedHeaderBlockRoot {
		return scale.Update{}, ErrFinalizedHeaderNotImported
	}

	// If the finalized header import succeeded, we add it to this cache. This cache is used to determine
	// from which last finalized header needs to imported (i.e. start and end finalized blocks, to backfill execution
	// headers in between).
	h.cache.Finalized.LastSyncedHash = update.FinalizedHeaderBlockRoot
	h.cache.Finalized.LastSyncedSlot = uint64(update.Payload.FinalizedHeader.Slot)

	h.cache.AddCheckPoint(update.FinalizedHeaderBlockRoot, update.BlockRootsTree, uint64(update.Payload.FinalizedHeader.Slot))

	return update, err
}

func (h *Header) SyncHeader(ctx context.Context, headerUpdate scale.HeaderUpdate, slotsLeft uint64) error {

	blockHash := headerUpdate.Payload.ExecutionHeader.BlockHash.Hex()
	blockNumber := uint64(headerUpdate.Payload.ExecutionHeader.BlockNumber)

	log.WithFields(log.Fields{
		"slot":                 headerUpdate.Payload.Header.Slot,
		"slotsLeftToSync":      slotsLeft,
		"executionBlockRoot":   blockHash,
		"executionBlockNumber": blockNumber,
	}).Info("Syncing header between last two finalized headers")

	err := h.writer.WriteToParachainAndRateLimit(ctx, "EthereumBeaconClient.submit_execution_header", headerUpdate.Payload)
	if err != nil {
		return fmt.Errorf("write to parachain: %w", err)
	}
	return nil
}

func (h *Header) SyncHeadersFromFinalized(ctx context.Context) error {
	lastAttemptedFinalizedHeader := h.cache.Finalized.LastAttemptedSyncHash
	secondLastFinalizedHeader := h.cache.LastFinalizedHeader()

	hasChanged, err := h.syncer.HasFinalizedHeaderChanged(lastAttemptedFinalizedHeader)
	if err != nil {
		return err
	}

	if !hasChanged {
		return ErrFinalizedHeaderUnchanged
	}

	sync, err := h.SyncFinalizedHeader(ctx)
	if err != nil {
		return err
	}
	err = h.SyncHeaders(ctx, secondLastFinalizedHeader, sync.FinalizedHeaderBlockRoot)

	if err != nil {
		return err
	}

	return nil
}

func (h *Header) SyncHeaders(ctx context.Context, fromHeaderBlockRoot, toHeaderBlockRoot common.Hash) error {
	fromHeader, err := h.syncer.Client.GetHeader(fromHeaderBlockRoot)
	if err != nil {
		return err
	}

	toHeader, err := h.syncer.Client.GetHeader(toHeaderBlockRoot)
	if err != nil {
		return err
	}

	fromSlot := fromHeader.Slot
	toSlot := toHeader.Slot
	totalSlots := toSlot - fromSlot

	log.WithFields(log.Fields{
		"fromHeader": fromHeaderBlockRoot,
		"fromSlot":   fromSlot,
		"fromEpoch":  h.syncer.ComputeEpochAtSlot(fromSlot),
		"toHeader":   toHeaderBlockRoot,
		"toSlot":     toSlot,
		"toEpoch":    h.syncer.ComputeEpochAtSlot(toSlot),
		"totalSlots": totalSlots,
	}).Info("starting to back-fill headers")

	headersToSync := []scale.HeaderUpdate{}

	currentSlot := fromSlot + 1 // start syncing at next block after last synced block

	checkpoint, err := h.getClosestCheckpoint(currentSlot)
	if err != nil {
		return fmt.Errorf("get closest checkpoint: %w", err)
	}

	headerUpdate, err := h.syncer.GetNextHeaderUpdateBySlotWithAncestryProof(currentSlot, checkpoint)
	if err != nil {
		return fmt.Errorf("get next header update by slot with ancestry proof: %w", err)
	}

	for currentSlot <= toSlot {
		epoch := h.syncer.ComputeEpochAtSlot(currentSlot)

		currentSyncPeriod := h.syncer.ComputeSyncPeriodAtSlot(currentSlot)

		if currentSyncPeriod > h.cache.LastSyncedSyncCommitteePeriod {
			err = h.syncLaggingSyncCommitteePeriods(ctx, h.cache.LastSyncedSyncCommitteePeriod, currentSlot)
			if err != nil {
				return fmt.Errorf("sync lagging sync committee periods: %w", err)
			}
		}

		// start of new epoch, sync headers of last epoch
		if h.syncer.IsStartOfEpoch(currentSlot) {
			log.WithFields(log.Fields{
				"epoch": epoch - 1,
			}).Debug("syncing header in epoch")
			for _, header := range headersToSync {
				err := h.SyncHeader(ctx, header, toSlot-uint64(header.Payload.Header.Slot))
				if err != nil {
					return fmt.Errorf("sync execution header: %w", err)
				}
			}

			// new epoch, start with clean array
			headersToSync = []scale.HeaderUpdate{}
		}

		nextSlot := currentSlot + 1

		log.WithFields(log.Fields{
			"currentSlot": currentSlot,
			"nextSlot":    nextSlot,
		}).Info("fetching next header at slot")

		var nextHeaderUpdate scale.HeaderUpdate
		// If this is the last slot we need to sync, don't fetch the ancestry proof for the next slot
		// because its finalized header won't be synced yet. We still need to fetch the next block for the
		// sync aggregate though.
		if currentSlot == toSlot {
			nextHeaderUpdate, err = h.syncer.GetNextHeaderUpdateBySlot(nextSlot)
			if err != nil {
				return fmt.Errorf("get next header update by slot: %w", err)
			}
		} else {
			checkpoint, err = h.getClosestCheckpoint(nextSlot)
			if err != nil {
				return fmt.Errorf("get closest checkpoint: %w", err)
			}

			// To get the sync witness for the current synced header. This header
			// will be used as the next update.
			nextHeaderUpdate, err = h.syncer.GetNextHeaderUpdateBySlotWithAncestryProof(nextSlot, checkpoint)
			if err != nil {
				return fmt.Errorf("get next header update by slot with ancestry proof: %w", err)
			}
		}

		headersToSync = append(headersToSync, headerUpdate)
		headerUpdate = nextHeaderUpdate

		// last slot to be synced, sync headers
		if currentSlot >= toSlot {
			for _, header := range headersToSync {
				err := h.SyncHeader(ctx, header, toSlot-uint64(header.Payload.Header.Slot))
				if err != nil {
					return err
				}
			}
		}
		currentSlot = uint64(nextHeaderUpdate.Payload.Header.Slot)
	}

	return nil
}

// Syncs execution headers from the last synced execution header on the parachain to the current finalized header. Lagging execution headers can occur if the relayer
// stopped while still processing a set of execution headers.
func (h *Header) syncLaggingExecutionHeaders(ctx context.Context, lastFinalizedHeader common.Hash, lastFinalizedSlot uint64, executionHeaderState state.ExecutionHeader) error {
	if executionHeaderState.BlockNumber == 0 {
		log.Info("start of syncing, no execution header lag found")

		return nil
	}

	if executionHeaderState.BeaconSlot >= lastFinalizedSlot {
		log.WithFields(log.Fields{
			"slot":          executionHeaderState.BeaconSlot,
			"blockNumber":   executionHeaderState.BlockNumber,
			"executionHash": executionHeaderState.BlockHash,
		}).Info("execution headers sync up to date with last finalized header")

		return nil
	}

	log.WithFields(log.Fields{
		"executionSlot": executionHeaderState.BeaconSlot,
		"finalizedSlot": lastFinalizedSlot,
		"blockNumber":   executionHeaderState.BlockNumber,
		"executionHash": executionHeaderState.BlockHash,
		"finalizedHash": lastFinalizedHeader,
		"slotsBacklog":  lastFinalizedSlot - executionHeaderState.BeaconSlot,
	}).Info("execution headers sync is not up to date with last finalized header, syncing lagging execution headers")

	err := h.SyncHeaders(ctx, executionHeaderState.BeaconBlockRoot, lastFinalizedHeader)
	if err != nil {
		return fmt.Errorf("sync headers: %w", err)
	}

	return nil
}

func (h *Header) syncLaggingSyncCommitteePeriods(ctx context.Context, latestSyncedPeriod, latestSlot uint64) error {
	periodsToSync, err := h.syncer.GetSyncPeriodsToFetch(latestSyncedPeriod, latestSlot)
	if err != nil {
		return fmt.Errorf("check sync committee periods to be fetched: %w", err)
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
