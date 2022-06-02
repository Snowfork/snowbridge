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

	finalizedHeader, blockRoot, err := r.SyncFinalizedHeader(ctx)
	if err != nil {
		return err
	}

	prevSyncAggregate, err := r.syncer.GetSyncAggregateForSlot(uint64(finalizedHeader.FinalizedHeader.Slot))
	if err != nil {
		logrus.WithError(err).Error("Unable to get sync aggregate")

		return err
	}

	_, err = r.SyncHeader(ctx, uint64(finalizedHeader.FinalizedHeader.Slot), blockRoot, prevSyncAggregate)
	if err != nil {
		return err
	}

	ticker := time.NewTicker(time.Minute * 1)
	done := make(chan bool)

	go func() {
		err := func() error {
			for {
				select {
				case <-done:
					return nil
				case <-ticker.C:
					secondLastFinalizedHeader := r.syncer.Cache.LastFinalizedHeader()

					_, finalizedHeaderBlockRoot, err := r.SyncFinalizedHeader(ctx)
					if err != nil {
						return err
					}

					lastFinalizedHeader := r.syncer.Cache.LastFinalizedHeader()

					if lastFinalizedHeader == secondLastFinalizedHeader {
						logrus.Info("Still at same finalized header")

						continue
					}

					logrus.WithFields(logrus.Fields{
						"from": secondLastFinalizedHeader,
						"to":   lastFinalizedHeader,
					}).Info("Starting to back-fill headers")

					blockRoot := finalizedHeaderBlockRoot
					prevSyncAggregate, err := r.syncer.GetSyncAggregate(blockRoot)
					if err != nil {
						logrus.WithError(err).Error("Unable to get sync aggregate")

						continue
					}

					if lastFinalizedHeader == secondLastFinalizedHeader {
						logrus.Info("Still at same finalized header")

						continue
					}

					for i := lastFinalizedHeader; i > secondLastFinalizedHeader; i-- {
						headerUpdate, err := r.SyncHeader(ctx, i, blockRoot, prevSyncAggregate)
						if err != nil {
							return err
						}

						blockRoot = common.Hash(headerUpdate.Block.ParentRoot)
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
	initialSync, err := r.syncer.InitialSync("0xed94aec726c5158606f33b5c599f8bf14c9a88d1722fe1f3c327ddb882c219fc")
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
	logrus.Info("Syncing finalized header")

	// When the chain has been processed up until now, keep getting finalized block updates and send that to the parachain
	finalizedHeaderUpdate, blockRoot, err := r.syncer.GetFinalizedUpdate()
	if err != nil {
		logrus.WithError(err).Error("unable to sync finalized header")

		return syncer.FinalizedHeaderUpdate{}, common.Hash{}, err
	}

	if syncer.IsInArray(r.syncer.Cache.FinalizedHeaders, uint64(finalizedHeaderUpdate.FinalizedHeader.Slot)) {
		logrus.Info("Finalized header has been synced already, skipping")

		return syncer.FinalizedHeaderUpdate{}, common.Hash{}, err
	}

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

	r.syncer.Cache.FinalizedHeaders = append(r.syncer.Cache.FinalizedHeaders, uint64(finalizedHeaderUpdate.FinalizedHeader.Slot))

	return finalizedHeaderUpdate, blockRoot, err
}

func (r *Relay) SyncHeader(ctx context.Context, slot uint64, blockRoot common.Hash, syncAggregate scale.SyncAggregate) (syncer.HeaderUpdate, error) {
	logrus.WithFields(logrus.Fields{
		"slot":      slot,
		"blockRoot": blockRoot,
	}).Info("Syncing header at slot")

	headerUpdate, err := r.syncer.GetHeaderUpdate(blockRoot)
	if err != nil {
		logrus.WithError(err).Error("unable to sync finalized header")

		return syncer.HeaderUpdate{}, err
	}

	newTransactions := [][]byte{}
	for i, trans := range headerUpdate.Block.Body.ExecutionPayload.Transactions {
		if i < 100 {
			newTransactions = append(newTransactions, trans)
		}
	}

	headerUpdate.Block.Body.ExecutionPayload.Transactions = newTransactions
	//headerUpdate.SyncAggregate = syncAggregate

	err = r.writer.WriteToParachain(ctx, "import_execution_header", headerUpdate)
	if err != nil {
		logrus.WithError(err).Error("unable to write to parachain")

		return syncer.HeaderUpdate{}, err
	}

	r.syncer.Cache.HeadersMap[blockRoot] = slot

	return headerUpdate, nil
}
