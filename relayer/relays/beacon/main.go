package beacon

import (
	"context"

	"github.com/snowfork/snowbridge/relayer/chain/parachain"
	"github.com/snowfork/snowbridge/relayer/crypto/sr25519"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/config"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/api"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/protocol"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/store"

	log "github.com/sirupsen/logrus"
	"golang.org/x/sync/errgroup"
)

type Relay struct {
	config  *config.Config
	keypair *sr25519.Keypair
}

func NewRelay(
	config *config.Config,
	keypair *sr25519.Keypair,
) *Relay {
	return &Relay{
		config:  config,
		keypair: keypair,
	}
}

func (r *Relay) Start(ctx context.Context, eg *errgroup.Group) error {
	specSettings := r.config.Source.Beacon.Spec
	log.WithField("spec", specSettings).Info("spec settings")

	paraconn := parachain.NewConnection(r.config.Sink.Parachain.Endpoint, r.keypair.AsKeyringPair())

	err := paraconn.Connect(ctx)
	if err != nil {
		return err
	}

	writer := parachain.NewParachainWriter(
		paraconn,
		r.config.Sink.Parachain.MaxWatchedExtrinsics,
		r.config.Sink.Parachain.MaxBatchCallSize,
	)

	p := protocol.New(specSettings)

	err = writer.Start(ctx, eg)
	if err != nil {
		return err
	}

	s := store.New(r.config.Source.Beacon.DataStore.Location, r.config.Source.Beacon.DataStore.MaxEntries, *p)
	err = s.Connect()
	if err != nil {
		return err
	}
	defer s.Close()

	beaconAPI := api.NewBeaconClient(r.config.Source.Beacon.Endpoint)
	headers := header.New(
		writer,
		beaconAPI,
		specSettings,
		&s,
		p,
	)

	return headers.Sync(ctx, eg)
}
