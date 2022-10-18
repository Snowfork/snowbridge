package header

import (
	"context"
	"errors"
	"fmt"
	"time"

	"github.com/ethereum/go-ethereum/common"
	log "github.com/sirupsen/logrus"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/cache"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/state"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/writer"
	"golang.org/x/sync/errgroup"
)

var ErrFinalizedHeaderUnchanged = errors.New("finalized header unchanged")
var ErrFinalizedHeaderNotImported = errors.New("finalized header not imported")

type Header struct {
	cache  *cache.BeaconCache
	writer *writer.ParachainWriter
	syncer *syncer.Syncer
}

func New(writer *writer.ParachainWriter, beaconEndpoint string, slotsInEpoch uint64, epochsPerSyncCommitteePeriod uint64) Header {
	return Header{
		cache:  cache.New(),
		writer: writer,
		syncer: syncer.New(beaconEndpoint, slotsInEpoch, epochsPerSyncCommitteePeriod),
	}
}

func (h *Header) Sync(ctx context.Context, eg *errgroup.Group) (<-chan uint64, <-chan uint64, error) {
	latestSyncedPeriod, err := h.writer.GetLastSyncedSyncCommitteePeriod()
	if err != nil {
		return nil, nil, fmt.Errorf("fetch last sync commitee: %w", err)
	}

	h.cache.SetLastSyncedSyncCommitteePeriod(latestSyncedPeriod)

	log.WithField("period", latestSyncedPeriod).Info("set cache: last beacon synced sync committee period")

	periodsToSync, err := h.syncer.GetSyncPeriodsToFetch(latestSyncedPeriod)
	if err != nil {
		return nil, nil, fmt.Errorf("check sync committee periods to be fetched: %w", err)
	}

	log.WithFields(log.Fields{
		"periods": periodsToSync,
	}).Info("sync committee periods to be synced")

	for _, period := range periodsToSync {
		err := h.SyncCommitteePeriodUpdate(ctx, period)
		if err != nil {
			return nil, nil, err
		}
	}

	lastFinalizedHeader, err := h.writer.GetLastStoredFinalizedHeader()
	if err != nil {
		return nil, nil, fmt.Errorf("fetch last finalized header: %w", err)
	}

	lastFinalizedSlot, err := h.writer.GetLastStoredFinalizedHeaderSlot()
	if err != nil {
		return nil, nil, fmt.Errorf("fetch last finalized header slot: %w", err)
	}

	h.cache.Finalized.Headers = append(h.cache.Finalized.Headers, lastFinalizedHeader)
	h.cache.Finalized.LastAttemptedSyncHash = lastFinalizedHeader
	h.cache.Finalized.LastAttemptedSyncSlot = lastFinalizedSlot

	log.WithFields(log.Fields{
		"hash": lastFinalizedHeader,
		"slot": lastFinalizedSlot,
	}).Info("set cache: last finalized header")

	executionHeaderState, err := h.writer.GetExecutionHeaderState()
	if err != nil {
		return nil, nil, fmt.Errorf("fetch last execution hash: %w", err)
	}

	basicChannel := make(chan uint64)
	incentivizedChannel := make(chan uint64)

	log.Info("starting to sync finalized headers")

	ticker := time.NewTicker(time.Second * 20)

	firstRun := true

	eg.Go(func() error {
		defer func() {
			close(basicChannel)
			close(incentivizedChannel)
		}()
		for {
			// This is in the same goroutine as the normal finalized header sync, otherwise the headers are syced out of order: for the lagging execution headers and
			// new headers. This needs to be in goroutine because otherwise sending a message to the basic and incentizived Go channels don't work.
			if firstRun {
				err = h.syncLaggingExecutionHeaders(ctx, lastFinalizedHeader, lastFinalizedSlot, executionHeaderState, basicChannel, incentivizedChannel)
				if err != nil {
					return fmt.Errorf("sync lagging execution headers: %w", err)
				}
				firstRun = false
			}

			err := h.SyncHeadersFromFinalized(ctx, basicChannel, incentivizedChannel)
			logFields := log.Fields{
				"finalized_header": h.cache.Finalized.LastAttemptedSyncHash,
				"slot":             h.cache.Finalized.LastAttemptedSyncSlot,
			}
			switch {
			case errors.Is(err, ErrFinalizedHeaderUnchanged):
				log.WithFields(logFields).Info("not importing unchanged header")
			case errors.Is(err, ErrFinalizedHeaderNotImported):
				log.WithFields(logFields).WithError(err).Warn("Not importing header this cycle")
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

	return basicChannel, incentivizedChannel, nil
}

func (h *Header) SyncCommitteePeriodUpdate(ctx context.Context, period uint64) error {
	syncCommitteeUpdate, err := h.syncer.GetSyncCommitteePeriodUpdate(period)

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

	syncCommitteeUpdate.SyncCommitteePeriod = types.NewU64(period)

	log.WithFields(log.Fields{
		"period": period,
	}).Info("syncing sync committee for period")

	err = h.writer.WriteToParachainAndWatch(ctx, "EthereumBeaconClient.sync_committee_period_update", syncCommitteeUpdate)
	if err != nil {
		return err
	}

	lastSyncedSyncCommitteePeriod, err := h.writer.GetLastSyncedSyncCommitteePeriod()
	if err != nil {
		return fmt.Errorf("fetch last synced committee periid from parachain: %w", err)
	}

	// Period + 1 because the sync committee update contains the next period's sync committee
	if lastSyncedSyncCommitteePeriod != period+1 {
		return fmt.Errorf("synced committee period %d not imported successfully", lastSyncedSyncCommitteePeriod)
	}

	h.cache.SetLastSyncedSyncCommitteePeriod(period)

	return nil
}

func (h *Header) SyncFinalizedHeader(ctx context.Context, basicChannel, incentivizedChannel chan uint64) (syncer.FinalizedHeaderUpdate, common.Hash, error) {
	// When the chain has been processed up until now, keep getting finalized block updates and send that to the parachain
	finalizedHeaderUpdate, blockRoot, err := h.syncer.GetFinalizedUpdate()
	if err != nil {
		return syncer.FinalizedHeaderUpdate{}, common.Hash{}, fmt.Errorf("fetch finalized header update: %w", err)
	}

	if syncer.IsInHashArray(h.cache.Finalized.Headers, blockRoot) {
		return syncer.FinalizedHeaderUpdate{}, common.Hash{}, ErrFinalizedHeaderUnchanged
	}

	log.WithFields(log.Fields{
		"slot":      finalizedHeaderUpdate.FinalizedHeader.Slot,
		"blockRoot": blockRoot,
	}).Info("syncing finalized header at slot")

	currentSyncPeriod := h.syncer.ComputeSyncPeriodAtSlot(uint64(finalizedHeaderUpdate.AttestedHeader.Slot))

	if h.cache.LastSyncedSyncCommitteePeriod < currentSyncPeriod {
		log.WithField("period", currentSyncPeriod).Info("sync period rolled over, getting sync committee update")

		err := h.SyncCommitteePeriodUpdate(ctx, currentSyncPeriod)
		if err != nil {
			return syncer.FinalizedHeaderUpdate{}, common.Hash{}, err
		}
	}

	err = h.writer.WriteToParachainAndWatch(ctx, "EthereumBeaconClient.import_finalized_header", finalizedHeaderUpdate)
	if err != nil {
		return syncer.FinalizedHeaderUpdate{}, common.Hash{}, fmt.Errorf("write to parachain: %w", err)
	}

	// We need a distinction between finalized headers that we've tried to sync, so that we don't try syncing
	// it over and over again with the same failure.
	h.cache.Finalized.LastAttemptedSyncHash = blockRoot
	h.cache.Finalized.LastAttemptedSyncSlot = uint64(finalizedHeaderUpdate.FinalizedHeader.Slot)

	lastStoredHeader, err := h.writer.GetLastStoredFinalizedHeader()
	if err != nil {
		return syncer.FinalizedHeaderUpdate{}, common.Hash{}, fmt.Errorf("fetch last finalized header from parachain: %w", err)
	}

	// If the finalized header import succeeded, we add it to this cache. This cache is used to determine
	// from which last finalized header needs to imported (i.e. start and end finalized blocks, to backfill execution
	// headers in between).
	h.cache.Finalized.Headers = append(h.cache.Finalized.Headers, blockRoot)

	if lastStoredHeader != blockRoot {
		return syncer.FinalizedHeaderUpdate{}, common.Hash{}, ErrFinalizedHeaderNotImported
	}

	return finalizedHeaderUpdate, blockRoot, err
}

func (h *Header) SyncHeader(ctx context.Context, headerUpdate syncer.HeaderUpdate, slotsLeft uint64) error {
	log.WithFields(log.Fields{
		"slot":                 headerUpdate.Block.Slot,
		"slotsLeftToSync":      slotsLeft,
		"executionBlockRoot":   headerUpdate.Block.Body.ExecutionPayload.BlockHash.Hex(),
		"executionBlockNumber": headerUpdate.Block.Body.ExecutionPayload.BlockNumber,
	}).Info("Syncing header between last two finalized headers")

	err := h.writer.WriteToParachainAndRateLimit(ctx, "EthereumBeaconClient.import_execution_header", headerUpdate)
	if err != nil {
		return fmt.Errorf("write to parachain: %w", err)
	}
	return nil
}

func (h *Header) SyncHeadersFromFinalized(ctx context.Context, basicChannel, incentivizedChannel chan uint64) error {
	lastAttemptedFinalizedHeader := h.cache.Finalized.LastAttemptedSyncHash
	secondLastFinalizedHeader := h.cache.LastFinalizedHeader()

	finalizedHeader, finalizedHeaderBlockRoot, err := h.SyncFinalizedHeader(ctx, basicChannel, incentivizedChannel)
	if err != nil {
		return err
	}

	if finalizedHeaderBlockRoot == lastAttemptedFinalizedHeader {
		return ErrFinalizedHeaderUnchanged
	}

	err = h.SyncHeaders(ctx, secondLastFinalizedHeader, finalizedHeaderBlockRoot, uint64(finalizedHeader.FinalizedHeader.Slot), basicChannel, incentivizedChannel)
	if err != nil {
		return err
	}

	return nil
}

func (h *Header) SyncHeaders(ctx context.Context, fromHeader, toHeader common.Hash, toHeaderSlot uint64, basicChannel, incentivizedChannel chan uint64) error {
	fromHeaderUpdate, err := h.syncer.GetHeaderUpdate(fromHeader)
	if err != nil {
		return err
	}

	toHeaderUpdate, err := h.syncer.GetHeaderUpdate(toHeader)
	if err != nil {
		return err
	}

	fromSlot := uint64(fromHeaderUpdate.Block.Slot)
	toSlot := uint64(toHeaderUpdate.Block.Slot)
	totalSlots := toSlot - fromSlot

	log.WithFields(log.Fields{
		"fromHeader": fromHeader,
		"fromSlot":   fromSlot,
		"fromEpoch":  h.syncer.ComputeEpochAtSlot(fromSlot),
		"toHeader":   toHeader,
		"toSlot":     toSlot,
		"toEpoch":    h.syncer.ComputeEpochAtSlot(toSlot),
		"totalSlots": totalSlots,
	}).Info("starting to back-fill headers")

	headersToSync := []syncer.HeaderUpdate{}

	currentSlot := fromSlot + 1
	for currentSlot <= toSlot {
		epoch := h.syncer.ComputeEpochAtSlot(currentSlot)

		// start of new epoch, sync headers of last epoch
		if float64(epoch) >= (float64(currentSlot) / float64(h.syncer.SlotsInEpoch)) {
			log.WithFields(log.Fields{
				"epoch": h.syncer.ComputeEpochAtSlot(currentSlot) - 1,
			}).Debug("syncing header in epoch")
			for _, header := range headersToSync {
				err := h.SyncHeader(ctx, header, toSlot-uint64(header.Block.Slot))
				if err != nil {
					return err
				}
			}

			if len(headersToSync) > 0 {
				err = h.sendLastBlockNumberMessage(ctx, uint64(headersToSync[len(headersToSync)-1].Block.Body.ExecutionPayload.BlockNumber), basicChannel, incentivizedChannel)
				if err != nil {
					return err
				}
			}

			// new epoch, start with clean array
			headersToSync = []syncer.HeaderUpdate{}
		}

		log.WithFields(log.Fields{
			"slot": currentSlot,
		}).Info("fetching header at slot")

		headerUpdate, err := h.syncer.GetNextHeaderUpdateBySlot(currentSlot)
		if err != nil {
			return err
		}
		// To get the sync witness for the current synced header. This header
		// will be used as the next update.
		nextHeaderUpdate, err := h.syncer.GetNextHeaderUpdateBySlot(currentSlot + 1)
		if err != nil {
			return err
		}

		headerUpdate.SyncAggregate = nextHeaderUpdate.Block.Body.SyncAggregate

		headersToSync = append(headersToSync, headerUpdate)
		headerUpdate = nextHeaderUpdate

		// last slot to be synced, sync headers
		if currentSlot >= toSlot {
			log.WithFields(log.Fields{
				"epoch": h.syncer.ComputeEpochAtSlot(currentSlot),
			}).Debug("syncing last set of headers in epoch")
			for _, header := range headersToSync {
				err := h.SyncHeader(ctx, header, toSlot-uint64(header.Block.Slot))
				if err != nil {
					return err
				}
			}

			if len(headersToSync) > 0 {
				err = h.sendLastBlockNumberMessage(ctx, uint64(headersToSync[len(headersToSync)-1].Block.Body.ExecutionPayload.BlockNumber), basicChannel, incentivizedChannel)
				if err != nil {
					return err
				}
			}
		}

		currentSlot = uint64(nextHeaderUpdate.Block.Slot)
	}

	return nil
}

func (h *Header) sendLastBlockNumberMessage(ctx context.Context, lastBlockNumber uint64, basicChannel, incentivizedChannel chan uint64) error {
	log.WithField("block_number", lastBlockNumber).Info("sending block number")

	select {
	case basicChannel <- lastBlockNumber:
	case <-ctx.Done():
		return ctx.Err()
	}

	select {
	case incentivizedChannel <- lastBlockNumber:
	case <-ctx.Done():
		return ctx.Err()
	}

	return nil
}

// Syncs execution headers from the last synced execution header on the parachain to the current finalized header. Lagging execution headers can occur if the relayer
// stopped while still processing a set of execution headers.
func (h *Header) syncLaggingExecutionHeaders(ctx context.Context, lastFinalizedHeader common.Hash, lastFinalizedSlot uint64, executionHeaderState state.ExecutionHeader, basicChannel, incentivizedChannel chan uint64) error {
	if executionHeaderState.BlockNumber == 0 {
		log.Info("start of syncing, no execution header lag found")

		return nil
	}

	if executionHeaderState.BeaconHeaderSlot >= lastFinalizedSlot {
		log.WithFields(log.Fields{
			"slot":          executionHeaderState.BeaconHeaderSlot,
			"blockNumber":   executionHeaderState.BlockNumber,
			"executionHash": executionHeaderState.BlockHash,
		}).Info("execution headers sync up to date with last finalized header")

		return nil
	}

	log.WithFields(log.Fields{
		"executionSlot": executionHeaderState.BeaconHeaderSlot,
		"finalizedSlot": lastFinalizedSlot,
		"blockNumber":   executionHeaderState.BlockNumber,
		"executionHash": executionHeaderState.BlockHash,
		"finalizedHash": lastFinalizedHeader,
	}).Info("execution headers sync is not up to date with last finalized header, syncing lagging execution headers")

	err := h.SyncHeaders(ctx, executionHeaderState.BeaconHeaderBlockRoot, lastFinalizedHeader, lastFinalizedSlot, basicChannel, incentivizedChannel)
	if err != nil {
		return err
	}

	return nil
}
