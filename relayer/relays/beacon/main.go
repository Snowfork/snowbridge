package beacon

import (
	"context"

	"github.com/snowfork/snowbridge/relayer/chain/ethereum"
	"github.com/snowfork/snowbridge/relayer/chain/parachain"
	"github.com/snowfork/snowbridge/relayer/crypto/sr25519"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/config"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/message"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/writer"
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
	specSettings := r.config.GetSpecSettings()

	paraconn := parachain.NewConnection(r.config.Sink.Parachain.Endpoint, r.keypair.AsKeyringPair())
	ethconn := ethereum.NewConnection(r.config.Source.Ethereum.Endpoint, nil)

	err := paraconn.Connect(ctx)
	if err != nil {
		return err
	}

	err = ethconn.Connect(ctx)
	if err != nil {
		return err
	}

	writer := writer.NewParachainWriter(
		paraconn,
	)

	err = writer.Start(ctx, eg)
	if err != nil {
		return err
	}

	headers := header.New(
		writer,
		r.config.Source.Beacon.Endpoint,
		specSettings.SlotsInEpoch,
		specSettings.EpochsPerSyncCommitteePeriod,
	)

	messages, err := message.New(
		ctx,
		eg,
		writer,
		&r.config.Source,
		ethconn,
	)
	if err != nil {
		return err
	}

	basicChannel, incentivizedChannel, err := headers.Sync(ctx, eg)
	if err != nil {
		return err
	}

	err = messages.SyncBasic(ctx, eg, basicChannel)
	if err != nil {
		return err
	}

	err = messages.SyncIncentivized(ctx, eg, incentivizedChannel)
	if err != nil {
		return err
	}

	return nil
}
