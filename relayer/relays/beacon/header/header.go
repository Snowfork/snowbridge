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
	"github.com/snowfork/snowbridge/relayer/relays/beacon/protocol"
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
	cache    *cache.BeaconCache
	writer   parachain.ChainWriter
	syncer   *syncer.Syncer
	protocol *protocol.Protocol
}

func New(writer parachain.ChainWriter, client api.BeaconAPI, setting config.SpecSettings, store store.BeaconStore, protocol *protocol.Protocol) Header {
	return Header{
		cache:    cache.New(setting.SlotsInEpoch, setting.EpochsPerSyncCommitteePeriod),
		writer:   writer,
		syncer:   syncer.New(client, store, protocol),
		protocol: protocol,
	}
}

func (h *Header) Sync(ctx context.Context, eg *errgroup.Group) error {
	lastFinalizedHeaderState, err := h.writer.GetLastFinalizedHeaderState()
	if err != nil {
		return fmt.Errorf("fetch parachain last finalized header state: %w", err)
	}
	latestSyncedPeriod := h.protocol.ComputeSyncPeriodAtSlot(lastFinalizedHeaderState.BeaconSlot)

	log.WithFields(log.Fields{
		"last_finalized_hash":   lastFinalizedHeaderState.BeaconBlockRoot,
		"last_finalized_slot":   lastFinalizedHeaderState.BeaconSlot,
		"last_finalized_period": latestSyncedPeriod,
	}).Info("set cache: Current state")
	h.cache.SetLastSyncedFinalizedState(lastFinalizedHeaderState.BeaconBlockRoot, lastFinalizedHeaderState.BeaconSlot)
	h.cache.SetInitialCheckpointSlot(lastFinalizedHeaderState.InitialCheckpointSlot)
	h.cache.AddCheckPointSlots([]uint64{lastFinalizedHeaderState.BeaconSlot})

	// Special handling here for the initial checkpoint to sync the next sync committee which is not included in initial
	// checkpoint.
	if h.isInitialSyncPeriod() {
		err = h.SyncCommitteePeriodUpdate(ctx, latestSyncedPeriod)
		if err != nil {
			return fmt.Errorf("sync next committee for initial sync period: %w", err)
		}
	}

	log.Info("starting to sync finalized headers")

	ticker := time.NewTicker(time.Minute * 1)

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
	update, err := h.syncer.GetSyncCommitteePeriodUpdate(period, h.cache.Finalized.LastSyncedSlot)
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
	if uint64(update.Payload.FinalizedHeader.Slot) > h.cache.Finalized.LastSyncedSlot {
		diff := uint64(update.Payload.FinalizedHeader.Slot) - h.cache.Finalized.LastSyncedSlot
		log.WithFields(log.Fields{"diff": diff, "last_finalized_slot": h.cache.Finalized.LastSyncedSlot, "new_finalized_slot": uint64(update.Payload.FinalizedHeader.Slot)}).Info("checking max latency")
		if diff > h.protocol.Settings.SlotsInEpoch*h.protocol.Settings.EpochsPerSyncCommitteePeriod {
			log.Info("syncing an interim update")
			err = h.syncInterimFinalizedUpdate(ctx, h.cache.Finalized.LastSyncedSlot, uint64(update.Payload.FinalizedHeader.Slot))
			if err != nil {
				return fmt.Errorf("sync interim finalized header update: %w", err)
			}
		}
	} else {
		log.Info("interim update not required")
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
	lastUpdatedPeriod := h.protocol.ComputeSyncPeriodAtSlot(lastFinalizedHeaderState.BeaconSlot)
	if period != lastUpdatedPeriod {
		return ErrSyncCommitteeNotImported
	}
	h.cache.SetLastSyncedFinalizedState(update.FinalizedHeaderBlockRoot, uint64(update.Payload.FinalizedHeader.Slot))
	h.cache.AddCheckPoint(update.FinalizedHeaderBlockRoot, update.BlockRootsTree, uint64(update.Payload.FinalizedHeader.Slot))

	return nil
}

func (h *Header) SyncFinalizedHeader(ctx context.Context) error {
	// When the chain has been processed up until now, keep getting finalized block updates and send that to the parachain
	finalizedHeader, err := h.syncer.GetFinalizedHeader()
	if err != nil {
		return fmt.Errorf("fetch finalized header update from Ethereum beacon client: %w", err)
	}

	log.WithFields(log.Fields{
		"slot": finalizedHeader.Slot,
	}).Info("syncing finalized header from Ethereum beacon client")

	currentSyncPeriod := h.protocol.ComputeSyncPeriodAtSlot(uint64(finalizedHeader.Slot))
	lastSyncedPeriod := h.protocol.ComputeSyncPeriodAtSlot(h.cache.Finalized.LastSyncedSlot)

	if lastSyncedPeriod < currentSyncPeriod {
		err = h.syncLaggingSyncCommitteePeriods(ctx, lastSyncedPeriod, currentSyncPeriod)
		if err != nil {
			return fmt.Errorf("sync lagging sync committee periods: %w", err)
		}
	}

	return nil
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

func (h *Header) syncInterimFinalizedUpdate(ctx context.Context, lastSyncedSlot, newCheckpointSlot uint64) error {
	// Calculate the range that the interim finalized header update may be in
	minSlot := newCheckpointSlot - h.protocol.SlotsPerHistoricalRoot
	maxSlot := lastSyncedSlot + h.protocol.SlotsPerHistoricalRoot

	finalizedUpdate, err := h.syncer.GetFinalizedUpdateAtAttestedSlot(minSlot, maxSlot, false)
	if err != nil {
		return fmt.Errorf("get interim checkpoint to update chain (last synced slot %d, new slot: %d): %w", lastSyncedSlot, newCheckpointSlot, err)
	}

	log.WithField("slot", finalizedUpdate.Payload.FinalizedHeader.Slot).Info("syncing an interim update to on-chain")

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
	lastSyncedPeriod := h.protocol.ComputeSyncPeriodAtSlot(h.cache.Finalized.LastSyncedSlot)
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
		err = h.populateCheckPointCacheWithDataFromChain(slot)
		if err != nil {
			// There should always be a checkpoint onchain with the range of the sync committee period slots
			return checkpoint, fmt.Errorf("find checkpoint on-chain for slot %d: %w", slot, err)
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

func (h *Header) getNextHeaderUpdateBySlot(slot uint64) (scale.HeaderUpdatePayload, error) {
	slot = slot + 1
	return h.getHeaderUpdateBySlot(slot)
}

func (h *Header) populateCheckPointCacheWithDataFromChain(slot uint64) error {
	checkpointSlot := h.protocol.CalculateNextCheckpointSlot(slot)

	lastFinalizedHeaderState, err := h.writer.GetLastFinalizedHeaderState()
	if err != nil {
		return fmt.Errorf("get last finalized header for the checkpoint: %w", err)
	}

	if slot > lastFinalizedHeaderState.BeaconSlot {
		return ErrBeaconHeaderNotFinalized
	}

	if checkpointSlot < lastFinalizedHeaderState.BeaconSlot {
		historicState, err := h.findLatestCheckPoint(slot)
		if err != nil {
			return fmt.Errorf("get history finalized header for the checkpoint: %w", err)
		}
		checkpointSlot = historicState.BeaconSlot
	} else {
		// Setting the checkpoint slot to what is the latest finalized header on-chain, since the checkpoint should
		// not be after the latest finalized header on-chain
		checkpointSlot = lastFinalizedHeaderState.BeaconSlot
	}

	err = h.populateFinalizedCheckpoint(checkpointSlot)
	if err != nil {
		return fmt.Errorf("populated local cache with finalized header found on-chain: %w", err)
	}

	return nil
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

func (h *Header) FetchExecutionProof(blockRoot common.Hash) (scale.ProofPayload, error) {
	header, err := h.syncer.Client.GetHeaderByBlockRoot(blockRoot)
	if err != nil {
		return scale.ProofPayload{}, fmt.Errorf("get beacon header by blockRoot: %w", err)
	}
	lastFinalizedHeaderState, err := h.writer.GetLastFinalizedHeaderState()
	if err != nil {
		return scale.ProofPayload{}, fmt.Errorf("fetch last finalized header state: %w", err)
	}

	// There is a finalized header on-chain that will be able to verify the header containing the message.
	if header.Slot <= lastFinalizedHeaderState.BeaconSlot {
		headerUpdate, err := h.getHeaderUpdateBySlot(header.Slot)
		if err != nil {
			return scale.ProofPayload{}, fmt.Errorf("get header update by slot with ancestry proof: %w", err)
		}

		return scale.ProofPayload{
			HeaderPayload:    headerUpdate,
			FinalizedPayload: nil,
		}, nil
	}

	// The latest finalized header on-chain is older than the header containing the message, so we need to sync the
	// finalized header with the message.
	finalizedHeader, err := h.syncer.GetFinalizedHeader()
	if err != nil {
		return scale.ProofPayload{}, err
	}

	// If the header is not finalized yet, we can't do anything further.
	if header.Slot > uint64(finalizedHeader.Slot) {
		return scale.ProofPayload{}, ErrBeaconHeaderNotFinalized
	}

	var finalizedUpdate scale.Update
	// If we import the last finalized header, the gap between the finalized headers would be too large, so import
	// a slightly older header.
	if lastFinalizedHeaderState.BeaconSlot+h.protocol.SlotsPerHistoricalRoot < uint64(finalizedHeader.Slot) {
		finalizedUpdate, err = h.syncer.GetFinalizedUpdateAtAttestedSlot(header.Slot, lastFinalizedHeaderState.BeaconSlot+h.protocol.SlotsPerHistoricalRoot, false)
		if err != nil {
			return scale.ProofPayload{}, fmt.Errorf("get finalized update at attested slot: %w", err)
		}
	} else {
		finalizedUpdate, err = h.syncer.GetFinalizedUpdate()
		if err != nil {
			return scale.ProofPayload{}, fmt.Errorf("get finalized update: %w", err)
		}
	}

	checkpoint := cache.Proof{
		FinalizedBlockRoot: finalizedUpdate.FinalizedHeaderBlockRoot,
		BlockRootsTree:     finalizedUpdate.BlockRootsTree,
		Slot:               uint64(finalizedUpdate.Payload.FinalizedHeader.Slot),
	}
	headerUpdate, err := h.syncer.GetHeaderUpdate(blockRoot, &checkpoint)

	return scale.ProofPayload{
		HeaderPayload:    headerUpdate,
		FinalizedPayload: &finalizedUpdate,
	}, nil

}

func (h *Header) isInitialSyncPeriod() bool {
	initialPeriod := h.protocol.ComputeSyncPeriodAtSlot(h.cache.InitialCheckpointSlot)
	lastFinalizedPeriod := h.protocol.ComputeSyncPeriodAtSlot(h.cache.Finalized.LastSyncedSlot)
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

	syncCommitteePeriod := h.protocol.Settings.SlotsInEpoch * h.protocol.Settings.EpochsPerSyncCommitteePeriod
	slotPeriodIndex := slot / syncCommitteePeriod

	for index := startIndex; index >= endIndex; index-- {
		beaconRoot, err := h.writer.GetFinalizedBeaconRootByIndex(uint32(index))
		if err != nil {
			return beaconState, fmt.Errorf("GetFinalizedBeaconRootByIndex %d, error: %w", index, err)
		}
		beaconState, err = h.writer.GetFinalizedHeaderStateByBlockRoot(beaconRoot)
		if err != nil {
			return beaconState, fmt.Errorf("GetFinalizedHeaderStateByBlockRoot %s, error: %w", beaconRoot.Hex(), err)
		}
		statePeriodIndex := beaconState.BeaconSlot / syncCommitteePeriod
		if beaconState.BeaconSlot < slot {
			break
		}
		// Found the beaconState
		if beaconState.BeaconSlot > slot && beaconState.BeaconSlot < slot+syncCommitteePeriod && slotPeriodIndex == statePeriodIndex {
			break
		}
	}
	if beaconState.BeaconSlot > slot && beaconState.BeaconSlot < slot+syncCommitteePeriod {
		return beaconState, nil
	}

	return beaconState, fmt.Errorf("no checkpoint on chain for slot %d", slot)
}
