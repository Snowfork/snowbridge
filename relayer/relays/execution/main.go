package execution

import (
	"context"

	"github.com/snowfork/snowbridge/relayer/chain/ethereum"
	"github.com/snowfork/snowbridge/relayer/chain/parachain"
	"github.com/snowfork/snowbridge/relayer/crypto/sr25519"
	"github.com/snowfork/snowbridge/relayer/relays/execution/config"
	"github.com/snowfork/snowbridge/relayer/relays/execution/message"
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

	writer := parachain.NewParachainWriter(
		paraconn,
		r.config.Sink.Parachain.MaxWatchedExtrinsics,
	)

	err = writer.Start(ctx, eg)
	if err != nil {
		return err
	}

	listener := message.NewEthereumListener(
		&r.config.Source,
		ethconn,
	)

	err = listener.Start(ctx, eg)
	if err != nil {
		return err
	}

	messages, err := message.New(
		writer,
		listener,
	)
	if err != nil {
		return err
	}

	err = messages.Sync(ctx, eg)
	if err != nil {
		return err
	}

	return nil
}
