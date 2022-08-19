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
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/scale"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/writer"
)

var ErrFinalizedHeaderUnchanged = errors.New("finalized header unchanged")

type Header struct {
	cache  *cache.BeaconCache
	writer *writer.ParachainWriter
	syncer *syncer.Syncer
}

func New(cache *cache.BeaconCache, writer *writer.ParachainWriter, beaconEndpoint string, slotsInEpoch uint64, epochsPerSyncCommitteePeriod uint64) Header {
	syncer := syncer.New(beaconEndpoint, slotsInEpoch, epochsPerSyncCommitteePeriod)

	return Header{cache, writer, syncer}
}

func (h *Header) Sync(ctx context.Context) (<-chan uint64, error) {
	latestSyncedPeriod, err := h.writer.GetLastSyncedSyncCommitteePeriod()
	if err != nil {
		return nil, fmt.Errorf("fetch last sync commitee: %w", err)
	}

	h.cache.SetLastSyncedSyncCommitteePeriod(latestSyncedPeriod)

	log.WithField("period", latestSyncedPeriod).Info("set cache: last beacon synced sync committee period")

	periodsToSync, err := h.syncer.GetSyncPeriodsToFetch(latestSyncedPeriod)
	if err != nil {
		return nil, fmt.Errorf("check sync committee periods to be fetched: %w", err)
	}

	log.WithFields(log.Fields{
		"periods": periodsToSync,
	}).Info("sync committee periods to be synced")

	for _, period := range periodsToSync {
		err := h.SyncCommitteePeriodUpdate(ctx, period)
		if err != nil {
			return nil, err
		}
	}

	lastFinalizedHeader, err := h.writer.GetLastStoredFinalizedHeader()
	if err != nil {
		return nil, fmt.Errorf("fetch last finalized header: %w", err)
	}

	lastFinalizedSlot, err := h.writer.GetLastStoredFinalizedHeaderSlot()
	if err != nil {
		return nil, fmt.Errorf("fetch last finalized header slot: %w", err)
	}

	h.cache.FinalizedHeaders = append(h.cache.FinalizedHeaders, lastFinalizedHeader)

	log.WithFields(log.Fields{
		"hash": lastFinalizedHeader,
		"slot": lastFinalizedSlot,
	}).Info("set cache: last finalized header")

	log.Info("starting to sync finalized headers")

	ticker := time.NewTicker(time.Second * 20)
	done := make(chan bool)

	finalizedHeader := make(chan uint64)

	go func() {
		for {
			lastBlockNumber, err := h.SyncHeaders(ctx)
			switch {
			case errors.Is(err, ErrFinalizedHeaderUnchanged):
				log.WithField("finalized_header", h.cache.LastFinalizedHeader()).Info("finalized header unchanged")
			case err != nil:
				log.WithError(err).Error("error while syncing headers")

				return
			default:
				finalizedHeader <- lastBlockNumber
			}

			select {
			case <-done:
				return
			case <-ticker.C:
				continue
			}
		}
	}()

	return finalizedHeader, nil
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

	err = h.writer.WriteToParachain(ctx, "EthereumBeaconClient.sync_committee_period_update", syncCommitteeUpdate)
	if err != nil {
		return err
	}

	h.cache.SetLastSyncedSyncCommitteePeriod(period)

	return nil
}

func (h *Header) SyncFinalizedHeader(ctx context.Context) (syncer.FinalizedHeaderUpdate, common.Hash, error) {
	// When the chain has been processed up until now, keep getting finalized block updates and send that to the parachain
	finalizedHeaderUpdate, blockRoot, err := h.syncer.GetFinalizedUpdate()
	if err != nil {
		return syncer.FinalizedHeaderUpdate{}, common.Hash{}, fmt.Errorf("fetch finalized header update: %w", err)
	}

	if syncer.IsInHashArray(h.cache.FinalizedHeaders, blockRoot) {
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

	err = h.writer.WriteToParachain(ctx, "EthereumBeaconClient.import_finalized_header", finalizedHeaderUpdate)
	if err != nil {
		return syncer.FinalizedHeaderUpdate{}, common.Hash{}, fmt.Errorf("write to parachain: %w", err)
	}

	h.cache.FinalizedHeaders = append(h.cache.FinalizedHeaders, blockRoot)

	return finalizedHeaderUpdate, blockRoot, err
}

func (h *Header) SyncHeader(ctx context.Context, blockRoot common.Hash, syncAggregate scale.SyncAggregate) (syncer.HeaderUpdate, error) {
	headerUpdate, err := h.syncer.GetHeaderUpdate(blockRoot)
	if err != nil {
		return syncer.HeaderUpdate{}, fmt.Errorf("fetch header update: %w", err)
	}

	/*logrus.WithFields(logrus.Fields{
		"beaconBlockRoot":    blockRoot,
		"executionBlockRoot": headerUpdate.Block.Body.ExecutionPayload.BlockHash.Hex(),
		"slot":               headerUpdate.Block.Slot,
	}).Info("Syncing header between last two finalized headers")*/

	headerUpdate.SyncAggregate = syncAggregate

	err = h.writer.WriteToParachain(ctx, "EthereumBeaconClient.import_execution_header", headerUpdate)
	if err != nil {
		return syncer.HeaderUpdate{}, fmt.Errorf("write to parachain: %w", err)
	}

	h.cache.HeadersMap[blockRoot] = uint64(headerUpdate.Block.Slot)

	return headerUpdate, nil
}

func (h *Header) SyncHeaders(ctx context.Context) (uint64, error) {
	secondLastFinalizedHeader := h.cache.LastFinalizedHeader()

	finalizedHeader, finalizedHeaderBlockRoot, err := h.SyncFinalizedHeader(ctx)
	if err != nil {
		return 0, err
	}

	lastFinalizedHeader := h.cache.LastFinalizedHeader()

	if lastFinalizedHeader == secondLastFinalizedHeader {
		return 0, ErrFinalizedHeaderUnchanged
	}

	log.WithFields(log.Fields{
		"secondLastHash": secondLastFinalizedHeader,
		"lastHash":       lastFinalizedHeader,
	}).Info("starting to back-fill headers")

	blockRoot := common.HexToHash(finalizedHeader.FinalizedHeader.ParentRoot.Hex())

	prevSyncAggregate, err := h.syncer.GetSyncAggregate(finalizedHeaderBlockRoot)
	if err != nil {
		return 0, fmt.Errorf("fetch sync aggregate: %w", err)
	}

	foundFinalizedHeaderAncestor := false
	ancestorFinalizedHeader := common.Hash{}

	for secondLastFinalizedHeader != blockRoot {
		headerUpdate, err := h.SyncHeader(ctx, blockRoot, prevSyncAggregate)
		if err != nil {
			return 0, err
		}

		blockRoot = common.HexToHash(headerUpdate.Block.ParentRoot.Hex())
		if !foundFinalizedHeaderAncestor {
			ancestorFinalizedHeader = blockRoot
			foundFinalizedHeaderAncestor = true
		}

		prevSyncAggregate = headerUpdate.Block.Body.SyncAggregate
	}

	// Import the execution header for the second last finalized header too.
	_, err = h.SyncHeader(ctx, blockRoot, prevSyncAggregate)
	if err != nil {
		return 0, err
	}

	// Use the block just before the last finalized header, because the finalized header is not imported in this chunk
	executionBlockNumber, err := h.syncer.GetExecutionBlockHash(ancestorFinalizedHeader)
	if err != nil {
		return 0, err
	}

	return executionBlockNumber, nil
}
