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

	h.cache.FinalizedHeaders = append(h.cache.FinalizedHeaders, lastFinalizedHeader)
	h.cache.LastAttemptedFinalizedHeader = lastFinalizedHeader

	log.WithFields(log.Fields{
		"hash": lastFinalizedHeader,
		"slot": lastFinalizedSlot,
	}).Info("set cache: last finalized header")

	log.Info("starting to sync finalized headers")

	ticker := time.NewTicker(time.Second * 20)

	basicChannel := make(chan uint64)
	incentivizedChannel := make(chan uint64)

	lastSyncedExecutionBlockNumber := uint64(0)

	eg.Go(func() error {
		defer func() {
			close(basicChannel)
			close(incentivizedChannel)
		}()
		for {
			err := h.SyncHeaders(ctx)
			switch {
			case errors.Is(err, ErrFinalizedHeaderUnchanged):
				log.WithError(err).WithField("finalized_header", h.cache.LastAttemptedFinalizedHeader).Info("not importing unchanged header")
			case errors.Is(err, ErrFinalizedHeaderNotImported):
				log.WithError(err).Warn("Not importing header this cycle")
			case err != nil:
				return err
			default:
				executionBlockNumber, err := h.syncer.GetExecutionBlockHash(h.cache.LastFinalizedHeader())
				if err != nil {
					return fmt.Errorf("fetch execution block hash: %w", err)
				}

				if executionBlockNumber > lastSyncedExecutionBlockNumber {
					lastSyncedExecutionBlockNumber = executionBlockNumber

					log.WithField("block_number", lastSyncedExecutionBlockNumber).Info("sending block number")

					select {
					case basicChannel <- lastSyncedExecutionBlockNumber:
					case <-ctx.Done():
						return ctx.Err()
					}

					select {
					case incentivizedChannel <- lastSyncedExecutionBlockNumber:
					case <-ctx.Done():
						return ctx.Err()
					}
				}
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

	_, err = h.writer.WriteToParachain(ctx, "EthereumBeaconClient.sync_committee_period_update", syncCommitteeUpdate)
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

	err = h.writer.WriteToParachainAndWatch(ctx, "EthereumBeaconClient.import_finalized_header", finalizedHeaderUpdate)
	if err != nil {
		return syncer.FinalizedHeaderUpdate{}, common.Hash{}, fmt.Errorf("write to parachain: %w", err)
	}

	// We need a distinction between finalized headers that we've tried to sync, so that we don't try syncing
	// it over and over again with the same failure.
	h.cache.LastAttemptedFinalizedHeader = blockRoot

	lastStoredHeader, err := h.writer.GetLastStoredFinalizedHeader()
	if err != nil {
		return syncer.FinalizedHeaderUpdate{}, common.Hash{}, fmt.Errorf("fetch last finalized header from parachain: %w", err)
	}

	// If the finalized header import succeeded, we add it to this cache. This cache is used to determine
	// from which last finalized header needs to imported (i.e. start and end finalized blocks, to backfill execution
	// headers in between).
	h.cache.FinalizedHeaders = append(h.cache.FinalizedHeaders, blockRoot)

	if lastStoredHeader != blockRoot {
		return syncer.FinalizedHeaderUpdate{}, common.Hash{}, ErrFinalizedHeaderNotImported
	}

	return finalizedHeaderUpdate, blockRoot, err
}

func (h *Header) SyncHeader(ctx context.Context, blockRoot common.Hash, syncAggregate scale.SyncAggregate) (syncer.HeaderUpdate, error) {
	headerUpdate, err := h.syncer.GetHeaderUpdate(blockRoot)
	if err != nil {
		return syncer.HeaderUpdate{}, fmt.Errorf("fetch header update: %w", err)
	}

	log.WithFields(log.Fields{
		"beaconBlockRoot":    blockRoot,
		"executionBlockRoot": headerUpdate.Block.Body.ExecutionPayload.BlockHash.Hex(),
		"slot":               headerUpdate.Block.Slot,
	}).Info("Syncing header between last two finalized headers")

	headerUpdate.SyncAggregate = syncAggregate

	_, err = h.writer.WriteToParachain(ctx, "EthereumBeaconClient.import_execution_header", headerUpdate)
	if err != nil {
		return syncer.HeaderUpdate{}, fmt.Errorf("write to parachain: %w", err)
	}

	h.cache.HeadersMap[blockRoot] = uint64(headerUpdate.Block.Slot)

	return headerUpdate, nil
}

func (h *Header) SyncHeaders(ctx context.Context) error {
	lastAttemptedFinalizedHeader := h.cache.LastAttemptedFinalizedHeader
	secondLastFinalizedHeader := h.cache.LastFinalizedHeader()

	finalizedHeader, finalizedHeaderBlockRoot, err := h.SyncFinalizedHeader(ctx)
	if err != nil {
		return err
	}

	if finalizedHeaderBlockRoot == lastAttemptedFinalizedHeader {
		return ErrFinalizedHeaderUnchanged
	}

	log.WithFields(log.Fields{
		"secondLastHash": secondLastFinalizedHeader,
		"lastHash":       finalizedHeaderBlockRoot,
	}).Info("starting to back-fill headers")

	blockRoot := common.HexToHash(finalizedHeader.FinalizedHeader.ParentRoot.Hex())
	prevSyncAggregate := finalizedHeader.SyncAggregate

	headerUpdate, err := h.SyncHeader(ctx, finalizedHeaderBlockRoot, prevSyncAggregate)
	if err != nil {
		return err
	}

	prevSyncAggregate = headerUpdate.Block.Body.SyncAggregate

	for secondLastFinalizedHeader != blockRoot {
		headerUpdate, err := h.SyncHeader(ctx, blockRoot, prevSyncAggregate)
		if err != nil {
			return err
		}

		blockRoot = common.HexToHash(headerUpdate.Block.ParentRoot.Hex())
		prevSyncAggregate = headerUpdate.Block.Body.SyncAggregate
	}

	return nil
}
