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
	ethereumListener *EthereumListener
	polkadotListener *PolkadotListener
	ethereumWriter   *EthereumWriter
	store          *Database
	tasks    chan Task
	ethHeaders       chan chain.Header
}

func NewRelay(config *Config, ethereumKeypair *secp256k1.Keypair) (*Relay, error) {
	log.Info("Relay created")

	dbMessages := make(chan DatabaseCmd)
	store := NewDatabase(dbMessages)

	err := store.Initialize()
	if err != nil {
		return nil, err
	}

	relaychainConn := relaychain.NewConnection(config.Source.Polkadot.Endpoint)
	ethereumConn := ethereum.NewConnection(config.Sink.Ethereum.Endpoint, ethereumKeypair)

	tasks := make(chan Task)
	ethHeaders := make(chan chain.Header)

	ethereumListener := NewEthereumListener(&config.Sink,
		ethereumConn, store, tasks, dbMessages, ethHeaders)

	ethereumWriter := NewEthereumWriter(&config.Sink, ethereumConn,
		store, dbMessages, tasks)

	polkadotListener := NewPolkadotListener(
		config,
		relaychainConn,
		tasks,
	)

	return &Relay{
		config:           config,
		relaychainConn:   relaychainConn,
		ethereumListener: ethereumListener,
		ethereumConn:     ethereumConn,
		ethereumWriter:   ethereumWriter,
		polkadotListener: polkadotListener,
		store:          store,
		tasks:    tasks,
		ethHeaders:       ethHeaders,
	}, nil
}

func (relay *Relay) Start(ctx context.Context, eg *errgroup.Group) error {
	err := relay.store.Start(ctx, eg)
	if err != nil {
		log.WithError(err).Error("Failed to start database")
		return err
	}

	err = relay.relaychainConn.Connect(ctx)
	if err != nil {
		return err
	}

	err = relay.ethereumConn.Connect(ctx)
	if err != nil {
		return err
	}

	latestBeefyBlock, err := relay.ethereumListener.Start(ctx, eg)
	if err != nil {
		return err
	}

	err = relay.polkadotListener.Start(ctx, eg, latestBeefyBlock)
	if err != nil {
		return err
	}

	err = relay.ethereumWriter.Start(ctx, eg)
	if err != nil {
		return err
	}

	return nil
}
