package beacon

import (
	"context"

	"github.com/sirupsen/logrus"
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
	lightClientSnapshot, err := r.syncer.InitialSync("0xed94aec726c5158606f33b5c599f8bf14c9a88d1722fe1f3c327ddb882c219fc")
	if err != nil {
		logrus.WithError(err).Error("unable to do intial beacon chain sync")

		return err
	}

	writer := NewParachainWriter(
		r.paraconn,
	)

	writer.WritePayload(ctx, &ParachainPayload{
		InitialSync: &InitialSync{
			Header:                     lightClientSnapshot.Header,
			CurrentSyncCommittee:       lightClientSnapshot.CurrentSyncCommittee,
			CurrentSyncCommitteeBranch: lightClientSnapshot.CurrentSyncCommitteeBranch,
			Genesis:                    lightClientSnapshot.Genesis,
		},
	})

	err = r.syncer.SyncCommitteePeriodUpdates(lightClientSnapshot.Header.Slot)
	if err != nil {
		logrus.WithError(err).Error("unable to sync committee updates")

		return err
	}

	// When the chain has been processed up until now, keep getting finalized block updates and send that to the parachain
	err = r.syncer.FinalizedBlockUpdate()
	if err != nil {
		logrus.WithError(err).Error("unable to sync finalized header")

		return err
	}

	return nil
}
