package beacon

import (
	"context"
	"errors"
	"time"

	"github.com/ethereum/go-ethereum/common"
	"github.com/sirupsen/logrus"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/chain/ethereum"
	"github.com/snowfork/snowbridge/relayer/chain/parachain"
	"github.com/snowfork/snowbridge/relayer/crypto/sr25519"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/syncer"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/syncer/scale"
	"golang.org/x/sync/errgroup"
)

type Relay struct {
	config   *Config
	syncer   *syncer.Syncer
	keypair  *sr25519.Keypair
	paraconn *parachain.Connection
	writer   *ParachainWriter
	listener *EthereumListener
	ethconn  *ethereum.Connection
}

func NewRelay(
	config *Config,
	keypair *sr25519.Keypair,
) *Relay {
	return &Relay{
		config:  config,
		keypair: keypair,
	}
}

func (r *Relay) Start(ctx context.Context, eg *errgroup.Group) error {
	specSettings := r.config.GetSpecSettings()

	r.paraconn = parachain.NewConnection(r.config.Sink.Parachain.Endpoint, r.keypair.AsKeyringPair())
	r.syncer = syncer.New(r.config.Source.Beacon.Endpoint, specSettings.SlotsInEpoch, specSettings.EpochsPerSyncCommitteePeriod)
	r.ethconn = ethereum.NewConnection(r.config.Source.Ethereum.Endpoint, nil)

	err := r.paraconn.Connect(ctx)
	if err != nil {
		return err
	}

	err = r.ethconn.Connect(ctx)
	if err != nil {
		return err
	}

	r.writer = NewParachainWriter(
		r.paraconn,
	)

	err = r.writer.Start(ctx, eg)
	if err != nil {
		return err
	}

	r.listener = NewEthereumListener(
		&r.config.Source,
		r.ethconn,
	)

	err = r.listener.Start(ctx, eg)
	if err != nil {
		return err
	}

	err = r.Sync(ctx)
	if err != nil {
		return err
	}

	return nil
}

func (r *Relay) Sync(ctx context.Context) error {
	latestSyncedPeriod, err := r.writer.getLastSyncedSyncCommitteePeriod()
	if err != nil {
		logrus.WithError(err).Error("unable to get last synced sync committee")

		return err
	}

	r.syncer.Cache.SetLastSyncedSyncCommitteePeriod(latestSyncedPeriod)

	logrus.WithField("period", latestSyncedPeriod).Info("set cache: last beacon synced sync committee period")

	periodsToSync, err := r.syncer.GetSyncPeriodsToFetch(latestSyncedPeriod)
	if err != nil {
		logrus.WithError(err).Error("unable to check sync committee periods to be fetched")

		return err
	}

	logrus.WithFields(logrus.Fields{
		"periods": periodsToSync,
	}).Info("sync committee periods to be synced")

	for _, period := range periodsToSync {
		err := r.SyncCommitteePeriodUpdate(ctx, period)
		if err != nil {
			return err
		}
	}

	lastVerifiedMessageBlock, err := r.writer.getLastVerifiedMessageBlock()
	if err != nil {
		logrus.WithError(err).Error("unable to get last synced sync committee")

		return err
	}

	r.syncer.Cache.LastVerifiedMessageBlock = lastVerifiedMessageBlock

	logrus.WithField("blockNumber", lastVerifiedMessageBlock).Info("set cache: last verified message block found")

	lastFinalizedHeader, err := r.writer.getLastStoredFinalizedHeader()
	if err != nil {
		logrus.WithError(err).Error("unable to get last finalized header")

		return err
	}

	lastFinalizedSlot, err := r.writer.getLastStoredFinalizedHeaderSlot()
	if err != nil {
		logrus.WithError(err).Error("unable to get last finalized header slot")

		return err
	}

	r.syncer.Cache.FinalizedHeaders = append(r.syncer.Cache.FinalizedHeaders, lastFinalizedHeader)

	logrus.WithFields(logrus.Fields{
		"hash": lastFinalizedHeader,
		"slot": lastFinalizedSlot,
	}).Info("set cache: last finalized header")

	logrus.Info("starting to sync finalized headers")

	err = r.SyncHeaders(ctx)
	if err != nil {
		logrus.WithError(err).Error("error while syncing headers")
	}

	ticker := time.NewTicker(time.Second * 20)
	done := make(chan bool)

	go func() {
		for {
			select {
			case <-done:
				return
			case <-ticker.C:
				err := r.SyncHeaders(ctx)
				if err != nil {
					logrus.WithError(err).Error("error while syncing headers")
				}
			}
		}
	}()

	return nil
}

func (r *Relay) SyncCommitteePeriodUpdate(ctx context.Context, period uint64) error {
	syncCommitteeUpdate, err := r.syncer.GetSyncCommitteePeriodUpdate(period)

	switch {
	case errors.Is(err, syncer.ErrCommitteeUpdateHeaderInDifferentSyncPeriod):
		{
			logrus.WithField("period", period).Info("committee update and header in different sync periods, skipping")

			return err
		}
	case err != nil:
		{
			logrus.WithError(err).Error("unable check sync committee periods to be fetched")

			return err
		}
	}

	syncCommitteeUpdate.SyncCommitteePeriod = types.NewU64(period)

	logrus.WithFields(logrus.Fields{
		"period": period,
	}).Info("syncing sync committee for period")

	err = r.writer.WriteToParachain(ctx, "EthereumBeaconClient.sync_committee_period_update", syncCommitteeUpdate)
	if err != nil {
		return err
	}

	r.syncer.Cache.SetLastSyncedSyncCommitteePeriod(period)

	return nil
}

func (r *Relay) SyncFinalizedHeader(ctx context.Context) (syncer.FinalizedHeaderUpdate, common.Hash, error) {
	// When the chain has been processed up until now, keep getting finalized block updates and send that to the parachain
	finalizedHeaderUpdate, blockRoot, err := r.syncer.GetFinalizedUpdate()
	if err != nil {
		logrus.WithError(err).Error("unable to sync finalized header")

		return syncer.FinalizedHeaderUpdate{}, common.Hash{}, err
	}

	if syncer.IsInHashArray(r.syncer.Cache.FinalizedHeaders, blockRoot) {
		logrus.WithFields(logrus.Fields{
			"slot":      finalizedHeaderUpdate.FinalizedHeader.Slot,
			"blockRoot": blockRoot,
		}).Info("finalized header has been synced already, skipping.")

		return syncer.FinalizedHeaderUpdate{}, common.Hash{}, err
	}

	logrus.WithFields(logrus.Fields{
		"slot":      finalizedHeaderUpdate.FinalizedHeader.Slot,
		"blockRoot": blockRoot,
	}).Info("syncing finalized header at slot")

	currentSyncPeriod := r.syncer.ComputeSyncPeriodAtSlot(uint64(finalizedHeaderUpdate.AttestedHeader.Slot))

	if r.syncer.Cache.LastSyncedSyncCommitteePeriod < currentSyncPeriod {
		logrus.WithField("period", currentSyncPeriod).Info("sync period rolled over, getting sync committee update")

		err := r.SyncCommitteePeriodUpdate(ctx, currentSyncPeriod)
		if err != nil {
			return syncer.FinalizedHeaderUpdate{}, common.Hash{}, err
		}
	}

	err = r.writer.WriteToParachain(ctx, "EthereumBeaconClient.import_finalized_header", finalizedHeaderUpdate)
	if err != nil {
		logrus.WithError(err).Error("unable to write to parachain")

		return syncer.FinalizedHeaderUpdate{}, common.Hash{}, err
	}

	r.syncer.Cache.FinalizedHeaders = append(r.syncer.Cache.FinalizedHeaders, blockRoot)

	return finalizedHeaderUpdate, blockRoot, err
}

func (r *Relay) SyncHeader(ctx context.Context, blockRoot common.Hash, syncAggregate scale.SyncAggregate) (syncer.HeaderUpdate, error) {
	headerUpdate, err := r.syncer.GetHeaderUpdate(blockRoot)
	if err != nil {
		logrus.WithError(err).Error("unable to sync finalized header")

		return syncer.HeaderUpdate{}, err
	}

	logrus.WithFields(logrus.Fields{
		"beaconBlockRoot":    blockRoot,
		"executionBlockRoot": headerUpdate.Block.Body.ExecutionPayload.BlockHash.Hex(),
		"slot":               headerUpdate.Block.Slot,
	}).Info("Syncing header between last two finalized headers")

	headerUpdate.SyncAggregate = syncAggregate

	err = r.writer.WriteToParachain(ctx, "EthereumBeaconClient.import_execution_header", headerUpdate)
	if err != nil {
		logrus.WithError(err).Error("unable to write to parachain")

		return syncer.HeaderUpdate{}, err
	}

	r.syncer.Cache.HeadersMap[blockRoot] = uint64(headerUpdate.Block.Slot)

	return headerUpdate, nil
}

func (r *Relay) SyncHeaders(ctx context.Context) error {
	secondLastFinalizedHeader := r.syncer.Cache.LastFinalizedHeader()

	finalizedHeader, finalizedHeaderBlockRoot, err := r.SyncFinalizedHeader(ctx)
	if err != nil {
		return err
	}

	lastFinalizedHeader := r.syncer.Cache.LastFinalizedHeader()

	if lastFinalizedHeader == secondLastFinalizedHeader {
		return nil
	}

	logrus.WithFields(logrus.Fields{
		"secondLastHash": secondLastFinalizedHeader,
		"lastHash":       lastFinalizedHeader,
	}).Info("starting to back-fill headers")

	blockRoot := common.HexToHash(finalizedHeader.FinalizedHeader.ParentRoot.Hex())

	prevSyncAggregate, err := r.syncer.GetSyncAggregate(finalizedHeaderBlockRoot)
	if err != nil {
		logrus.WithError(err).Error("Unable to get sync aggregate")

		return err
	}

	for secondLastFinalizedHeader != blockRoot {
		headerUpdate, err := r.SyncHeader(ctx, blockRoot, prevSyncAggregate)
		if err != nil {
			return err
		}

		blockRoot = common.HexToHash(headerUpdate.Block.ParentRoot.Hex())
		prevSyncAggregate = headerUpdate.Block.Body.SyncAggregate
	}

	// Import the execution header for the second last finalized header too.
	_, err = r.SyncHeader(ctx, blockRoot, prevSyncAggregate)
	if err != nil {
		return err
	}

	lastBlockNumber, secondLastBlockNumber, err := r.syncer.GetBlockRange(lastFinalizedHeader, secondLastFinalizedHeader)
	if err != nil {
		return err
	}

	if lastBlockNumber > r.syncer.Cache.LastVerifiedMessageBlock {
		secondLastBlockNumber = r.syncer.Cache.LastVerifiedMessageBlock
	}

	logrus.WithFields(logrus.Fields{
		"start": secondLastBlockNumber,
		"end":   lastBlockNumber - 1,
	}).Info("processing events for block numbers")

	payload, err := r.listener.ProcessEvents(ctx, secondLastBlockNumber, lastBlockNumber-1)
	if err != nil {
		return err
	}

	r.syncer.Cache.LastVerifiedMessageBlock = lastBlockNumber - 1

	return r.writeMessages(ctx, payload)
}

func (r *Relay) writeMessages(ctx context.Context, payload ParachainPayload) error {
	for _, msg := range payload.Messages {
		err := r.writer.WriteToParachain(ctx, msg.Call, msg.Args...)
		if err != nil {
			return err
		}
	}

	return nil
}
