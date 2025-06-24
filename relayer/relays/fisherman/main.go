package fisherman

import (
	"context"
	"fmt"
	"time"

	"golang.org/x/sync/errgroup"

	"github.com/snowfork/snowbridge/relayer/chain/ethereum"
	"github.com/snowfork/snowbridge/relayer/chain/relaychain"
	"github.com/snowfork/snowbridge/relayer/crypto/secp256k1"
	"github.com/snowfork/snowbridge/relayer/crypto/sr25519"

	log "github.com/sirupsen/logrus"
)

type Relay struct {
	config            *Config
	relaychainConn    *relaychain.Connection
	ethereumConnBeefy *ethereum.Connection
	beefyListener     *BeefyListener
}

func NewRelay(config *Config, keypair *secp256k1.Keypair, keypair2 *sr25519.Keypair) (*Relay, error) {
	log.Info("Creating fisherman worker")

	relaychainWriterConn := relaychain.NewConnection(config.Source.Polkadot.Endpoint, keypair2.AsKeyringPair())

	ethereumConnBeefy := ethereum.NewConnection(&config.Source.Ethereum, keypair)

	beefyListener := NewBeefyListener(
		&config.Source,
		&config.Schedule,
		ethereumConnBeefy,
		relaychainWriterConn,
		nil,
		nil,
		nil,
	)

	return &Relay{
		config:            config,
		relaychainConn:    relaychainWriterConn,
		ethereumConnBeefy: ethereumConnBeefy,
		beefyListener:     beefyListener,
	}, nil
}

func (relay *Relay) Start(ctx context.Context, eg *errgroup.Group) error {
	err := relay.ethereumConnBeefy.Connect(ctx)
	if err != nil {
		return fmt.Errorf("unable to connect to ethereum: beefy: %w", err)
	}

	err = relay.relaychainConn.ConnectWithHeartBeat(ctx, 30*time.Second)
	if err != nil {
		return err
	}

	log.Info("Starting beefy listener (fisherman)")
	err = relay.beefyListener.Start(ctx, eg)
	if err != nil {
		return err
	}

	log.Info("Starting equivocation fisherman")

	return nil
}
