package beacon

import (
	"context"
	"errors"
	"time"

	"github.com/snowfork/snowbridge/relayer/relays/beacon/syncer/scale"

	"github.com/ethereum/go-ethereum/common"

	"github.com/sirupsen/logrus"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/chain/parachain"
	"github.com/snowfork/snowbridge/relayer/crypto/sr25519"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/syncer"
	"golang.org/x/sync/errgroup"
)

type Relay struct {
	config   *Config
	syncer   *syncer.Syncer
	keypair  *sr25519.Keypair
	paraconn *parachain.Connection
	writer   *ParachainWriter
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
	r.paraconn = parachain.NewConnection(r.config.Sink.Parachain.Endpoint, r.keypair.AsKeyringPair())
	r.syncer = syncer.New(r.config.Source.Beacon.Endpoint)

	err := r.paraconn.Connect(ctx)
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

	return r.Sync(ctx)
}

func (r *Relay) Sync(ctx context.Context) error {
	initialSync, err := r.InitialSync(ctx)
	if err != nil {
		return err
	}

	r.syncer.Cache.SyncCommitteePeriodsSynced, err = r.syncer.GetSyncPeriodsToFetch(uint64(initialSync.Header.Slot))
	if err != nil {
		logrus.WithError(err).Error("unable check sync committee periods to be fetched")

		return err
	}

	logrus.WithFields(logrus.Fields{
		"periods": r.syncer.Cache.SyncCommitteePeriodsSynced,
	}).Info("Sync committee periods that needs fetching")

	for _, period := range r.syncer.Cache.SyncCommitteePeriodsSynced {
		logrus.WithFields(logrus.Fields{
			"period": period,
		}).Info("Fetch sync committee period update")

		err := r.SyncCommitteePeriodUpdate(ctx, period)
		if err != nil {
			return err
		}
	}

	logrus.Info("Done with sync committee updates")

	logrus.Info("Starting to sync finalized headers")

	_, _, err = r.SyncFinalizedHeader(ctx)
	if err != nil {
		return err
	}

	ticker := time.NewTicker(time.Second * 20)
	done := make(chan bool)

	go func() {
		err := func() error {
			for {
				select {
				case <-done:
					return nil
				case <-ticker.C:
					secondLastFinalizedHeader := r.syncer.Cache.LastFinalizedHeader()

					finalizedHeader, finalizedHeaderBlockRoot, err := r.SyncFinalizedHeader(ctx)
					if err != nil {
						return err
					}

					lastFinalizedHeader := r.syncer.Cache.LastFinalizedHeader()

					if lastFinalizedHeader == secondLastFinalizedHeader {
						continue
					}

					logrus.WithFields(logrus.Fields{
						"secondLastHash": secondLastFinalizedHeader,
						"lastHash":       lastFinalizedHeader,
					}).Info("Starting to back-fill headers")

					blockRoot := common.HexToHash(finalizedHeader.FinalizedHeader.ParentRoot.Hex())

					prevSyncAggregate, err := r.syncer.GetSyncAggregate(finalizedHeaderBlockRoot)
					if err != nil {
						logrus.WithError(err).Error("Unable to get sync aggregate")

						continue
					}

					for secondLastFinalizedHeader != blockRoot {
						headerUpdate, err := r.SyncHeader(ctx, blockRoot, prevSyncAggregate)
						if err != nil {
							return err
						}

						blockRoot = common.HexToHash(headerUpdate.Block.ParentRoot.Hex())
						prevSyncAggregate = headerUpdate.Block.Body.SyncAggregate
					}

				}
			}
		}()
		if err != nil {
			logrus.WithError(err).Error("Error while syncing headers")
		}
	}()

	return nil
}

func (r *Relay) InitialSync(ctx context.Context) (syncer.InitialSync, error) {
	initialSync, err := r.syncer.InitialSync("0x73504113348a42e26c7ac8835fc0397524d05c2ac0b11748a28bc47ad54a475c")
	if err != nil {
		logrus.WithError(err).Error("unable to do initial beacon chain sync")

		return syncer.InitialSync{}, err
	}

	err = r.writer.WriteToParachain(ctx, "initial_sync", initialSync)
	if err != nil {
		logrus.WithError(err).Error("unable to write to parachain")

		return syncer.InitialSync{}, err
	}

	logrus.Info("intial sync written to parachain")

	return initialSync, nil
}

func (r *Relay) SyncCommitteePeriodUpdate(ctx context.Context, period uint64) error {
	syncCommitteeUpdate, err := r.syncer.GetSyncCommitteePeriodUpdate(period, period)

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

	return r.writer.WriteToParachain(ctx, "sync_committee_period_update", syncCommitteeUpdate)
}

func (r *Relay) SyncFinalizedHeader(ctx context.Context) (syncer.FinalizedHeaderUpdate, common.Hash, error) {
	// When the chain has been processed up until now, keep getting finalized block updates and send that to the parachain
	finalizedHeaderUpdate, blockRoot, err := r.syncer.GetFinalizedUpdate()
	if err != nil {
		logrus.WithError(err).Error("unable to sync finalized header")

		return syncer.FinalizedHeaderUpdate{}, common.Hash{}, err
	}

	if syncer.IsInArray(r.syncer.Cache.FinalizedHeaderSlots, uint64(finalizedHeaderUpdate.FinalizedHeader.Slot)) {
		logrus.WithFields(logrus.Fields{
			"slot":      finalizedHeaderUpdate.FinalizedHeader.Slot,
			"blockRoot": blockRoot,
		}).Info("Finalized header has been synced already, skipping.")

		return syncer.FinalizedHeaderUpdate{}, common.Hash{}, err
	}

	logrus.WithFields(logrus.Fields{
		"slot":      finalizedHeaderUpdate.FinalizedHeader.Slot,
		"blockRoot": blockRoot,
	}).Info("Syncing finalized header at slot")

	currentSyncPeriod := syncer.ComputeSyncPeriodAtSlot(uint64(finalizedHeaderUpdate.AttestedHeader.Slot))

	if !syncer.IsInArray(r.syncer.Cache.SyncCommitteePeriodsSynced, currentSyncPeriod) {
		logrus.WithField("period", currentSyncPeriod).Info("Sync period rolled over, getting sync committee update")

		err := r.SyncCommitteePeriodUpdate(ctx, currentSyncPeriod)
		if err != nil {
			return syncer.FinalizedHeaderUpdate{}, common.Hash{}, err
		}

		r.syncer.Cache.AddSyncCommitteePeriod(currentSyncPeriod)
	}

	err = r.writer.WriteToParachain(ctx, "import_finalized_header", finalizedHeaderUpdate)
	if err != nil {
		logrus.WithError(err).Error("unable to write to parachain")

		return syncer.FinalizedHeaderUpdate{}, common.Hash{}, err
	}

	r.syncer.Cache.FinalizedHeaders = append(r.syncer.Cache.FinalizedHeaders, blockRoot)
	r.syncer.Cache.FinalizedHeaderSlots = append(r.syncer.Cache.FinalizedHeaderSlots, uint64(finalizedHeaderUpdate.FinalizedHeader.Slot))

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

	err = r.writer.WriteToParachain(ctx, "import_execution_header", headerUpdate)
	if err != nil {
		logrus.WithError(err).Error("unable to write to parachain")

		return syncer.HeaderUpdate{}, err
	}

	r.syncer.Cache.HeadersMap[blockRoot] = uint64(headerUpdate.Block.Slot)

	return headerUpdate, nil
}
