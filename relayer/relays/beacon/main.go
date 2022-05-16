package beacon

import (
	"context"
	"errors"
	"time"

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

		r.SyncCommitteePeriodUpdate(ctx, period)
	}

	logrus.Info("Done with sync committee updates")

	logrus.Info("Starting to sync finalized headers")

	r.SyncFinalizedHeader(ctx)

	finalizedHeaderTicker := time.NewTicker(time.Minute * 5)
	doneFinalizedHeader := make(chan bool)

	go func() error {
		for {
			select {
			case <-doneFinalizedHeader:
				return nil
			case <-finalizedHeaderTicker.C:
				r.SyncFinalizedHeader(ctx)
			}
		}
	}()

	headUpdateTicker := time.NewTicker(time.Minute * 5)
	doneHeadUpdate := make(chan bool)

	go func() error {
		for {
			select {
			case <-doneHeadUpdate:
				return nil
			case <-headUpdateTicker.C:
				r.SyncHeader(ctx)
			}
		}
	}()

	return nil
}

func (r *Relay) InitialSync(ctx context.Context) (syncer.InitialSync, error) {
	initialSync, err := r.syncer.InitialSync("0xed94aec726c5158606f33b5c599f8bf14c9a88d1722fe1f3c327ddb882c219fc")
	if err != nil {
		logrus.WithError(err).Error("unable to do intial beacon chain sync")

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

func (r *Relay) SyncFinalizedHeader(ctx context.Context) error {
	logrus.Info("Syncing finalized header")

	// When the chain has been processed up until now, keep getting finalized block updates and send that to the parachain
	finalizedHeaderUpdate, err := r.syncer.GetFinalizedBlockUpdate()
	if err != nil {
		logrus.WithError(err).Error("unable to sync finalized header")

		return err
	}

	if syncer.IsInArray(r.syncer.Cache.FinalizedHeaders, uint64(finalizedHeaderUpdate.FinalizedHeader.Slot)) {
		logrus.Info("Finalized header has been synced already, skipped")

		return nil
	}

	currentSyncPeriod := syncer.ComputeSyncPeriodAtSlot(uint64(finalizedHeaderUpdate.AttestedHeader.Slot))

	if !syncer.IsInArray(r.syncer.Cache.SyncCommitteePeriodsSynced, currentSyncPeriod) {
		logrus.WithField("period", currentSyncPeriod).Info("Sync period rolled over, getting sync committee update")

		r.SyncCommitteePeriodUpdate(ctx, currentSyncPeriod)

		r.syncer.Cache.AddSyncCommitteePeriod(currentSyncPeriod)
	}

	err = r.writer.WriteToParachain(ctx, "import_finalized_header", finalizedHeaderUpdate)
	if err != nil {
		logrus.WithError(err).Error("unable to write to parachain")

		return err
	}

	r.syncer.Cache.FinalizedHeaders = append(r.syncer.Cache.FinalizedHeaders, uint64(finalizedHeaderUpdate.FinalizedHeader.Slot))

	return err
}

func (r *Relay) SyncHeader(ctx context.Context) error {
	logrus.Info("Syncing head update")

	headerUpdate, err := r.syncer.GetHeaderUpdate()
	if err != nil {
		logrus.WithError(err).Error("unable to sync latest header")

		return err
	}

	if syncer.IsInArray(r.syncer.Cache.Headers, uint64(headerUpdate.AttestedHeader.Slot)) {
		logrus.Info("latest header has been synced already, skipped")

		return nil
	}

	currentSyncPeriod := syncer.ComputeSyncPeriodAtSlot(uint64(headerUpdate.AttestedHeader.Slot))

	if !syncer.IsInArray(r.syncer.Cache.SyncCommitteePeriodsSynced, currentSyncPeriod) {
		logrus.WithField("period", currentSyncPeriod).Info("Sync period rolled over, getting sync committee update")

		r.SyncCommitteePeriodUpdate(ctx, currentSyncPeriod)

		r.syncer.Cache.AddSyncCommitteePeriod(currentSyncPeriod)
	}

	err = r.writer.WriteToParachain(ctx, "import_header", headerUpdate)
	if err != nil {
		logrus.WithError(err).Error("unable to write to parachain")

		return err
	}

	r.syncer.Cache.Headers = append(r.syncer.Cache.Headers, uint64(headerUpdate.AttestedHeader.Slot))

	return err
}
