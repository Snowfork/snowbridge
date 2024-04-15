package header

import (
	"context"
	"errors"
	"fmt"
	"time"

	"github.com/snowfork/snowbridge/relayer/chain/parachain"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/cache"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/config"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/api"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/scale"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/state"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/store"

	"github.com/ethereum/go-ethereum/common"
	log "github.com/sirupsen/logrus"
	"golang.org/x/sync/errgroup"
)

var ErrFinalizedHeaderUnchanged = errors.New("finalized header unchanged")
var ErrFinalizedHeaderNotImported = errors.New("finalized header not imported")
var ErrSyncCommitteeNotImported = errors.New("sync committee not imported")
var ErrSyncCommitteeLatency = errors.New("sync committee latency found")
var ErrExecutionHeaderNotImported = errors.New("execution header not imported")
var ErrBeaconHeaderNotFinalized = errors.New("beacon header not finalized")

type Header struct {
	cache                        *cache.BeaconCache
	writer                       parachain.ChainWriter
	syncer                       *syncer.Syncer
	store                        store.BeaconStore
	slotsInEpoch                 uint64
	epochsPerSyncCommitteePeriod uint64
}

func New(writer parachain.ChainWriter, client api.BeaconAPI, setting config.SpecSettings, store store.BeaconStore) Header {
	return Header{
		cache:                        cache.New(setting.SlotsInEpoch, setting.EpochsPerSyncCommitteePeriod),
		writer:                       writer,
		syncer:                       syncer.New(client, setting, store),
		store:                        store,
		slotsInEpoch:                 setting.SlotsInEpoch,
		epochsPerSyncCommitteePeriod: setting.EpochsPerSyncCommitteePeriod,
	}
}

func (h *Header) Sync(ctx context.Context, eg *errgroup.Group) error {
	lastFinalizedHeaderState, err := h.writer.GetLastFinalizedHeaderState()
	if err != nil {
		return fmt.Errorf("fetch parachain last finalized header state: %w", err)
	}
	latestSyncedPeriod := h.syncer.ComputeSyncPeriodAtSlot(lastFinalizedHeaderState.BeaconSlot)

	log.WithFields(log.Fields{
		"last_finalized_hash":   lastFinalizedHeaderState.BeaconBlockRoot,
		"last_finalized_slot":   lastFinalizedHeaderState.BeaconSlot,
		"last_finalized_period": latestSyncedPeriod,
	}).Info("set cache: Current state")
	h.cache.SetLastSyncedFinalizedState(lastFinalizedHeaderState.BeaconBlockRoot, lastFinalizedHeaderState.BeaconSlot)
	h.cache.SetInitialCheckpointSlot(lastFinalizedHeaderState.InitialCheckpointSlot)
	h.cache.AddCheckPointSlots([]uint64{lastFinalizedHeaderState.BeaconSlot})

	log.Info("starting to sync finalized headers")

	ticker := time.NewTicker(time.Second * 10)

	eg.Go(func() error {
		for {
			err = h.SyncHeaders(ctx)
			logFields := log.Fields{
				"finalized_header": h.cache.Finalized.LastSyncedHash,
				"finalized_slot":   h.cache.Finalized.LastSyncedSlot,
			}
			switch {
			case errors.Is(err, ErrFinalizedHeaderUnchanged):
				log.WithFields(logFields).Info("not importing unchanged header")
			case errors.Is(err, ErrFinalizedHeaderNotImported):
				log.WithFields(logFields).WithError(err).Warn("Not importing header this cycle")
			case errors.Is(err, ErrSyncCommitteeNotImported):
				log.WithFields(logFields).WithError(err).Warn("SyncCommittee not imported")
			case errors.Is(err, ErrSyncCommitteeLatency):
				log.WithFields(logFields).WithError(err).Warn("SyncCommittee latency found")
			case errors.Is(err, ErrExecutionHeaderNotImported):
				log.WithFields(logFields).WithError(err).Warn("ExecutionHeader not imported")
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

	// If the gap between the last two finalized headers is more than the sync committee period, sync an interim
	// finalized header
	maxLatency := h.cache.Finalized.LastSyncedSlot + (h.slotsInEpoch * h.epochsPerSyncCommitteePeriod)
	if maxLatency < uint64(update.Payload.FinalizedHeader.Slot) {
		err = h.syncInterimFinalizedUpdate(ctx, h.cache.Finalized.LastSyncedSlot)
		if err != nil {
			return fmt.Errorf("sync interim finalized header update: %w", err)
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

	// Only update cache when SyncCommitteeUpdate import succeeded and period updated as expected
	lastFinalizedHeaderState, err := h.writer.GetLastFinalizedHeaderState()
	if err != nil {
		return fmt.Errorf("fetch last finalized header state: %w", err)
	}
	lastUpdatedPeriod := h.syncer.ComputeSyncPeriodAtSlot(lastFinalizedHeaderState.BeaconSlot)
	if period != lastUpdatedPeriod {
		return ErrSyncCommitteeNotImported
	}
	h.cache.SetLastSyncedFinalizedState(update.FinalizedHeaderBlockRoot, uint64(update.Payload.FinalizedHeader.Slot))
	h.cache.AddCheckPoint(update.FinalizedHeaderBlockRoot, update.BlockRootsTree, uint64(update.Payload.FinalizedHeader.Slot))

	return nil
}

func (h *Header) SyncFinalizedHeader(ctx context.Context) error {
	// When the chain has been processed up until now, keep getting finalized block updates and send that to the parachain
	update, err := h.syncer.GetFinalizedUpdate()
	if err != nil {
		return fmt.Errorf("fetch finalized header update from Ethereum beacon client: %w", err)
	}

	log.WithFields(log.Fields{
		"slot":      update.Payload.FinalizedHeader.Slot,
		"blockRoot": update.FinalizedHeaderBlockRoot,
	}).Info("syncing finalized header from Ethereum beacon client")

	currentSyncPeriod := h.syncer.ComputeSyncPeriodAtSlot(uint64(update.Payload.AttestedHeader.Slot))
	lastSyncedPeriod := h.syncer.ComputeSyncPeriodAtSlot(h.cache.Finalized.LastSyncedSlot)

	if lastSyncedPeriod < currentSyncPeriod {
		err = h.syncLaggingSyncCommitteePeriods(ctx, lastSyncedPeriod, currentSyncPeriod)
		if err != nil {
			return fmt.Errorf("sync lagging sync committee periods: %w", err)
		}
	}

	return h.updateFinalizedHeaderOnchain(ctx, update)
}

// Write the provided finalized header update (possibly containing a sync committee) on-chain and check if it was
// imported successfully. Update the cache if it has and add the finalized header to the checkpoint cache.
func (h *Header) updateFinalizedHeaderOnchain(ctx context.Context, update scale.Update) error {
	err := h.writer.WriteToParachainAndWatch(ctx, "EthereumBeaconClient.submit", update.Payload)
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

	// If the finalized header import succeeded, we add it to this cache.
	h.cache.SetLastSyncedFinalizedState(update.FinalizedHeaderBlockRoot, uint64(update.Payload.FinalizedHeader.Slot))
	h.cache.AddCheckPoint(update.FinalizedHeaderBlockRoot, update.BlockRootsTree, uint64(update.Payload.FinalizedHeader.Slot))
	return nil
}

func (h *Header) SyncHeaders(ctx context.Context) error {
	finalizedUpdate, err := h.syncer.Client.GetLatestFinalizedUpdate()
	if err != nil {
		return fmt.Errorf("fetch finalized update: %w", err)
	}

	finalizedHeader, err := finalizedUpdate.Data.FinalizedHeader.Beacon.ToScale()
	if err != nil {
		return fmt.Errorf("convert finalized header to scale: %w", err)
	}

	hasChanged, err := h.syncer.HasFinalizedHeaderChanged(finalizedHeader, h.cache.Finalized.LastSyncedHash)
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

	return nil
}

func (h *Header) syncInterimFinalizedUpdate(ctx context.Context, lastSyncedSlot uint64) error {
	checkpointSlot := h.syncer.CalculateNextCheckpointSlot(lastSyncedSlot)
	finalizedUpdate, err := h.syncer.GetFinalizedUpdateAtAttestedSlot(checkpointSlot, lastSyncedSlot)
	if err != nil {
		return fmt.Errorf("get interim checkpoint to update chain (checkpoint slot %d, original slot: %d): %w", checkpointSlot, lastSyncedSlot, err)
	}

	err = h.updateFinalizedHeaderOnchain(ctx, finalizedUpdate)
	if err != nil {
		return fmt.Errorf("update interim finalized header on-chain: %w", err)
	}

	return nil
}

func (h *Header) syncLaggingSyncCommitteePeriods(ctx context.Context, latestSyncedPeriod, currentSyncPeriod uint64) error {
	// sync for all missing periods
	periodsToSync := []uint64{}
	for i := latestSyncedPeriod + 1; i <= currentSyncPeriod; i++ {
		periodsToSync = append(periodsToSync, i)
	}

	// Special handling here for the initial checkpoint to sync the next sync committee which is not included in initial
	// checkpoint.
	if h.isInitialSyncPeriod() {
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

	// If Latency found between LastSyncedSyncCommitteePeriod and currentSyncPeriod in Ethereum beacon client
	// just return error so to exit ASAP to allow ExecutionUpdate to catch up
	lastSyncedPeriod := h.syncer.ComputeSyncPeriodAtSlot(h.cache.Finalized.LastSyncedSlot)
	if lastSyncedPeriod < currentSyncPeriod {
		return ErrSyncCommitteeLatency
	}

	return nil
}

func (h *Header) populateFinalizedCheckpoint(slot uint64) error {
	finalizedHeader, err := h.syncer.Client.GetHeaderBySlot(slot)
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
		return fmt.Errorf("get finalized header state by block root: %w", err)
	}
	if onChainFinalizedHeader.BeaconSlot != slot {
		return fmt.Errorf("on chain finalized header inconsistent at slot %d", slot)
	}

	blockRootsProof, err := h.syncer.GetBlockRoots(slot)
	if err != nil && !errors.Is(err, syncer.ErrBeaconStateAvailableYet) {
		return fmt.Errorf("fetch block roots for slot %d: %w", slot, err)
	}

	log.Info("populating checkpoint")

	h.cache.AddCheckPoint(blockRoot, blockRootsProof.Tree, slot)

	return nil
}

// Find the closest finalized checkpoint for a given slot. If a checkpoint cannot be found in the local cache, look
// for a checkpoint that can be used on-chain. There should always be a checkpoint on-chain because on-chain we
// verify that there is not large gap than the sync committee period range.
func (h *Header) populateClosestCheckpoint(slot uint64) (cache.Proof, error) {
	var checkpoint cache.Proof
	checkpoint, err := h.cache.GetClosestCheckpoint(slot)

	switch {
	case errors.Is(cache.FinalizedCheckPointNotAvailable, err) || errors.Is(cache.FinalizedCheckPointNotPopulated, err):
		checkpointSlot := checkpoint.Slot
		if checkpointSlot == 0 {
			checkpointSlot, err = h.populateCheckPointCacheWithDataFromChain(slot)
			if err != nil {
				// There should always be a checkpoint onchain with the range of the sync committee period slots
				return checkpoint, fmt.Errorf("find checkpoint on-chain: %w", err)
			}
		}

		checkpoint, err = h.cache.GetClosestCheckpoint(slot)
		if err != nil {
			return checkpoint, fmt.Errorf("get closest checkpoint after populating finalized header: %w", err)
		}

		log.WithFields(log.Fields{"slot": slot, "checkpoint": checkpoint}).Info("checkpoint after populating finalized header")

		return checkpoint, nil
	case err != nil:
		return checkpoint, fmt.Errorf("get closest checkpoint: %w", err)
	}

	return checkpoint, nil
}

func (h *Header) populateCheckPointCacheWithDataFromChain(slot uint64) (uint64, error) {
	checkpointSlot := h.syncer.CalculateNextCheckpointSlot(slot)

	lastFinalizedHeaderState, err := h.writer.GetLastFinalizedHeaderState()
	if err != nil {
		return 0, fmt.Errorf("get last finalized header for the checkpoint: %w", err)
	}

	if slot > lastFinalizedHeaderState.BeaconSlot {
		return 0, ErrBeaconHeaderNotFinalized
	}

	if checkpointSlot < lastFinalizedHeaderState.BeaconSlot {
		historicState, err := h.findLatestCheckPoint(slot)
		if err != nil {
			return 0, fmt.Errorf("get history finalized header for the checkpoint: %w", err)
		}
		checkpointSlot = historicState.BeaconSlot
	} else {
		// Setting the checkpoint slot to what is the latest finalized header on-chain, since the checkpoint should
		// not be after the latest finalized header on-chain
		checkpointSlot = lastFinalizedHeaderState.BeaconSlot
	}

	err = h.populateFinalizedCheckpoint(checkpointSlot)
	if err != nil {
		return 0, fmt.Errorf("populated local cache with finalized header found on-chain: %w", err)
	}

	return 0, nil
}

func (h *Header) getHeaderUpdateBySlot(slot uint64) (scale.HeaderUpdatePayload, error) {
	header, err := h.syncer.FindBeaconHeaderWithBlockIncluded(slot)
	if err != nil {
		return scale.HeaderUpdatePayload{}, fmt.Errorf("get next beacon header with block included: %w", err)
	}
	checkpoint, err := h.populateClosestCheckpoint(header.Slot)
	if err != nil {
		return scale.HeaderUpdatePayload{}, fmt.Errorf("populate closest checkpoint: %w", err)
	}
	blockRoot, err := header.HashTreeRoot()
	if err != nil {
		return scale.HeaderUpdatePayload{}, fmt.Errorf("header hash tree root: %w", err)
	}
	return h.syncer.GetHeaderUpdate(blockRoot, &checkpoint)
}

func (h *Header) FetchExecutionProof(blockRoot common.Hash) (scale.HeaderUpdatePayload, error) {
	var headerUpdate scale.HeaderUpdatePayload
	header, err := h.syncer.Client.GetHeader(blockRoot)
	if err != nil {
		return headerUpdate, fmt.Errorf("get beacon header by blockRoot: %w", err)
	}
	lastFinalizedHeaderState, err := h.writer.GetLastFinalizedHeaderState()
	if err != nil {
		return headerUpdate, fmt.Errorf("fetch last finalized header state: %w", err)
	}

	if header.Slot > lastFinalizedHeaderState.BeaconSlot {
		return headerUpdate, ErrBeaconHeaderNotFinalized
	}
	headerUpdate, err = h.getHeaderUpdateBySlot(header.Slot)
	if err != nil {
		return headerUpdate, fmt.Errorf("get header update by slot with ancestry proof: %w", err)
	}
	return headerUpdate, nil
}

func (h *Header) isInitialSyncPeriod() bool {
	initialPeriod := h.syncer.ComputeSyncPeriodAtSlot(h.cache.InitialCheckpointSlot)
	lastFinalizedPeriod := h.syncer.ComputeSyncPeriodAtSlot(h.cache.Finalized.LastSyncedSlot)
	return initialPeriod == lastFinalizedPeriod
}

func (h *Header) findLatestCheckPoint(slot uint64) (state.FinalizedHeader, error) {
	var beaconState state.FinalizedHeader
	lastIndex, err := h.writer.GetLastFinalizedStateIndex()
	if err != nil {
		return beaconState, fmt.Errorf("GetLastFinalizedStateIndex error: %w", err)
	}
	startIndex := uint64(lastIndex)
	endIndex := uint64(0)
	if uint64(lastIndex) > h.epochsPerSyncCommitteePeriod {
		endIndex = endIndex - h.epochsPerSyncCommitteePeriod
	}

	syncCommitteePeriod := h.slotsInEpoch * h.epochsPerSyncCommitteePeriod

	for index := startIndex; index >= endIndex; index-- {
		beaconRoot, err := h.writer.GetFinalizedBeaconRootByIndex(uint32(index))
		if err != nil {
			return beaconState, fmt.Errorf("GetFinalizedBeaconRootByIndex %d, error: %w", index, err)
		}
		beaconState, err = h.writer.GetFinalizedHeaderStateByBlockRoot(beaconRoot)
		if err != nil {
			return beaconState, fmt.Errorf("GetFinalizedHeaderStateByBlockRoot %s, error: %w", beaconRoot.Hex(), err)
		}
		if beaconState.BeaconSlot < slot {
			break
		}
		if beaconState.BeaconSlot > slot && beaconState.BeaconSlot < slot+syncCommitteePeriod {
			break
		}
	}
	if beaconState.BeaconSlot > slot && beaconState.BeaconSlot < slot+syncCommitteePeriod {
		return beaconState, nil
	}

	return beaconState, fmt.Errorf("no checkpoint on chain for slot %d", slot)
}
