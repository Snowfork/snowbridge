package beacon

import (
	"context"

	"github.com/snowfork/snowbridge/relayer/chain/ethereum"
	"github.com/snowfork/snowbridge/relayer/chain/parachain"
	"github.com/snowfork/snowbridge/relayer/crypto/sr25519"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/cache"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/config"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/message"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/writer"
	"golang.org/x/sync/errgroup"
)

type Relay struct {
	config   *config.Config
	keypair  *sr25519.Keypair
	paraconn *parachain.Connection
	ethconn  *ethereum.Connection
	cache    *cache.BeaconCache
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
	specSettings := r.config.GetSpecSettings()

	r.paraconn = parachain.NewConnection(r.config.Sink.Parachain.Endpoint, r.keypair.AsKeyringPair())
	r.ethconn = ethereum.NewConnection(r.config.Source.Ethereum.Endpoint, nil)

	err := r.paraconn.Connect(ctx)
	if err != nil {
		return err
	}

	err = r.ethconn.Connect(ctx)
	if err != nil {
		return err
	}

	writer := writer.NewParachainWriter(
		r.paraconn,
	)

	err = writer.Start(ctx, eg)
	if err != nil {
		return err
	}

	cache := cache.New()

	h := header.New(cache, writer, r.config.Source.Beacon.Endpoint, specSettings.SlotsInEpoch, specSettings.EpochsPerSyncCommitteePeriod)

	m, err := message.New(ctx, eg, cache, writer, &r.config.Source, r.ethconn)
	if err != nil {
		return err
	}

	finalizedBlock, err := h.Sync(ctx)
	if err != nil {
		return err
	}

	go m.SyncMessages(ctx, finalizedBlock)

	return nil
}
