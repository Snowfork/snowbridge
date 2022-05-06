package beacon

import (
	"context"
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
	r.syncer = syncer.New(r.config.Source.Beacon.Endpoint)
	r.paraconn = parachain.NewConnection(r.config.Sink.Parachain.Endpoint, r.keypair.AsKeyringPair())

	// Get an initial snapshot of the chain from a verified block
	initialSync, err := r.syncer.InitialSync("0xed94aec726c5158606f33b5c599f8bf14c9a88d1722fe1f3c327ddb882c219fc")
	if err != nil {
		logrus.WithError(err).Error("unable to do intial beacon chain sync")

		return err
	}

	err = r.paraconn.Connect(ctx)
	if err != nil {
		return err
	}

	writer := NewParachainWriter(
		r.paraconn,
	)

	err = writer.Start(ctx, eg)
	if err != nil {
		return err
	}

	err = writer.WriteToParachain(ctx, "initial_sync", initialSync)
	if err != nil {
		logrus.WithError(err).Error("unable to write to parachain")

		return err
	}

	logrus.Info("intial sync written to parachain")

	periods, err := r.syncer.GetSyncPeriodsToFetch(uint64(initialSync.Header.Slot))
	if err != nil {
		logrus.WithError(err).Error("unable check sync committee periods to be fetched")

		return err
	}

	logrus.WithFields(logrus.Fields{
		"periods": periods,
	}).Info("Sync committee periods that needs fetching")

	for _, period := range periods {
		logrus.WithFields(logrus.Fields{
			"period": period,
		}).Info("Fetch sync committee period update")
		syncCommitteeUpdate, err := r.syncer.GetSyncCommitteePeriodUpdate(period, period)
		if err != nil {
			logrus.WithError(err).Error("unable check sync committee periods to be fetched")

			return err
		}

		syncCommitteeUpdate.SyncCommitteePeriod = types.NewU64(period)

		err = writer.WriteToParachain(ctx, "sync_committee_period_update", syncCommitteeUpdate)
		if err != nil {
			logrus.WithError(err).Error("unable to write to parachain")

			return err
		}
	}

	// TODO check if period rolled over while updating

	logrus.Info("Done with sync committee updates")

	logrus.Info("Starting to sync finalized headers")

	ticker := time.NewTicker(time.Minute * 5)

	for range ticker.C {
		logrus.Info("Syncing finalized header")

		// When the chain has been processed up until now, keep getting finalized block updates and send that to the parachain
		finalizedHeaderUpdate, err := r.syncer.GetFinalizedBlockUpdate()
		if err != nil {
			logrus.WithError(err).Error("unable to sync finalized header")

			return err
		}

		logrus.Info("Checking if sync period rolled")

		currentSyncPeriod := syncer.ComputeSyncPeriodAtSlot(uint64(finalizedHeaderUpdate.AttestedHeader.Slot))
		
		if syncer.SyncPeriodRolledOver(periods, currentSyncPeriod) {
			logrus.WithField("period", currentSyncPeriod).Info("Sync period rolled over, getting sync committee update")

			syncCommitteeUpdate, err := r.syncer.GetSyncCommitteePeriodUpdate(currentSyncPeriod, currentSyncPeriod)
			if err != nil {
				logrus.WithError(err).Error("unable check sync committee periods to be fetched")

				return err
			}

			syncCommitteeUpdate.SyncCommitteePeriod = types.NewU64(currentSyncPeriod)

			err = writer.WriteToParachain(ctx, "sync_committee_period_update", syncCommitteeUpdate)
			if err != nil {
				logrus.WithError(err).Error("unable to write to parachain")

				return err
			}

			periods = append(periods, currentSyncPeriod)
		}

		err = writer.WriteToParachain(ctx, "import_finalized_header", finalizedHeaderUpdate)
		if err != nil {
			logrus.WithError(err).Error("unable to write to parachain")

			return err
		}
	}

	logrus.Info("Shutting down")

	return nil
}
