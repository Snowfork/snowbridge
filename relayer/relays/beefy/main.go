package beefy

import (
	"context"

	"golang.org/x/sync/errgroup"

	"github.com/snowfork/snowbridge/relayer/chain"
	"github.com/snowfork/snowbridge/relayer/chain/ethereum"
	"github.com/snowfork/snowbridge/relayer/chain/relaychain"
	"github.com/snowfork/snowbridge/relayer/crypto/secp256k1"

	log "github.com/sirupsen/logrus"
)

type Relay struct {
	config           *Config
	relaychainConn   *relaychain.Connection
	ethereumConn     *ethereum.Connection
	polkadotListener *PolkadotListener
	ethereumWriter   *EthereumWriter
	tasks            chan Task
	ethHeaders       chan chain.Header
}

func NewRelay(config *Config, ethereumKeypair *secp256k1.Keypair) (*Relay, error) {
	log.Info("Relay created")

	relaychainConn := relaychain.NewConnection(config.Source.Polkadot.Endpoint)
	ethereumConn := ethereum.NewConnection(config.Sink.Ethereum.Endpoint, ethereumKeypair)

	tasks := make(chan Task)
	ethHeaders := make(chan chain.Header)

	ethereumWriter := NewEthereumWriter(&config.Sink, ethereumConn, tasks)

	polkadotListener := NewPolkadotListener(
		config,
		relaychainConn,
		tasks,
	)

	return &Relay{
		config:           config,
		relaychainConn:   relaychainConn,
		ethereumConn:     ethereumConn,
		ethereumWriter:   ethereumWriter,
		polkadotListener: polkadotListener,
		tasks:            tasks,
		ethHeaders:       ethHeaders,
	}, nil
}

func (relay *Relay) Start(ctx context.Context, eg *errgroup.Group) error {
	err := relay.relaychainConn.Connect(ctx)
	if err != nil {
		return err
	}

	err = relay.ethereumConn.Connect(ctx)
	if err != nil {
		return err
	}

	latestBeefyBlock, err := relay.ethereumWriter.Start(ctx, eg)
	if err != nil {
		return err
	}

	err = relay.polkadotListener.Start(ctx, eg, latestBeefyBlock)
	if err != nil {
		return err
	}

	return nil
}
