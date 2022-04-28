package beacon

import (
	"context"

	"github.com/sirupsen/logrus"
	"github.com/snowfork/go-substrate-rpc-client/types"
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

type BeaconHeaderScale struct {
	Slot          types.U64
	ProposerIndex types.U64
	ParentRoot    types.H256
	StateRoot     types.H256
	BodyRoot      types.H256
}

func (r *Relay) Start(ctx context.Context, eg *errgroup.Group) error {
	r.syncer = syncer.New(r.config.Source.Beacon.Endpoint, r.config.Source.Beacon.FinalizedUpdateEndpoint)
	r.paraconn = parachain.NewConnection(r.config.Sink.Parachain.Endpoint, r.keypair.AsKeyringPair())

	// Get an initial snapshot of the chain from a verified block
	lightClientSnapshot, err := r.syncer.InitialSync("0xed94aec726c5158606f33b5c599f8bf14c9a88d1722fe1f3c327ddb882c219fc")
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

	header := BeaconHeaderScale{
		Slot:          types.NewU64(lightClientSnapshot.Header.Slot),
		ProposerIndex: types.NewU64(lightClientSnapshot.Header.Slot),
		ParentRoot:    types.NewH256(lightClientSnapshot.Header.ParentRoot.Bytes()),
		StateRoot:     types.NewH256(lightClientSnapshot.Header.ParentRoot.Bytes()),
		BodyRoot:      types.NewH256(lightClientSnapshot.Header.ParentRoot.Bytes()),
	}

	err = writer.WritePayload(ctx, &ParachainPayload{
		InitialSync: &InitialSync{
			Header:                     header,
			CurrentSyncCommittee:       []byte{},
			CurrentSyncCommitteeBranch: []byte{},
			ValidatorsRoot:             types.NewH256([]byte(lightClientSnapshot.ValidatorsRoot)),
		},
	}, eg)
	if err != nil {
		logrus.WithError(err).Error("unable to write to parachain")

		return err
	}

	logrus.Info("wrote payload to parachain")

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
