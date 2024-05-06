package parachain

import (
	"context"
	"fmt"

	"golang.org/x/sync/errgroup"

	"github.com/snowfork/snowbridge/relayer/chain/ethereum"
	"github.com/snowfork/snowbridge/relayer/chain/parachain"
	"github.com/snowfork/snowbridge/relayer/chain/relaychain"
	"github.com/snowfork/snowbridge/relayer/crypto/secp256k1"

	log "github.com/sirupsen/logrus"
)

type Relay struct {
	config                *Config
	parachainConn         *parachain.Connection
	relaychainConn        *relaychain.Connection
	ethereumBeefyConn     *ethereum.Connection
	ethereumWriterConn    *ethereum.Connection
	ethereumChannelWriter *EthereumWriter
	beefyListener         *BeefyListener
}

func NewRelay(config *Config, keypair *secp256k1.Keypair) (*Relay, error) {
	log.Info("Creating worker")

	parachainConn := parachain.NewConnection(config.Source.Parachain.Endpoint, nil)
	relaychainConn := relaychain.NewConnection(config.Source.Polkadot.Endpoint)

	// TODO: This is used by both the source & sink. They should use separate connections
	ethereumConnBeefy := ethereum.NewConnection(&config.Sink.Ethereum, keypair)
	ethereumWriterConn := ethereum.NewConnection(&config.Sink.Ethereum, keypair)

	// channel for messages from beefy listener to ethereum writer
	var tasks = make(chan *Task, 1)

	ethereumChannelWriter, err := NewEthereumWriter(
		&config.Sink,
		ethereumWriterConn,
		tasks,
	)
	if err != nil {
		return nil, err
	}

	beefyListener := NewBeefyListener(
		&config.Source,
		ethereumConnBeefy,
		relaychainConn,
		parachainConn,
		tasks,
	)

	return &Relay{
		config:                config,
		parachainConn:         parachainConn,
		relaychainConn:        relaychainConn,
		ethereumBeefyConn:     ethereumConnBeefy,
		ethereumWriterConn:    ethereumWriterConn,
		ethereumChannelWriter: ethereumChannelWriter,
		beefyListener:         beefyListener,
	}, nil
}

func (relay *Relay) Start(ctx context.Context, eg *errgroup.Group) error {
	err := relay.parachainConn.Connect(ctx)
	if err != nil {
		return err
	}

	err = relay.ethereumBeefyConn.Connect(ctx)
	if err != nil {
		return fmt.Errorf("connect to ethereum for beefy: %w", err)
	}

	err = relay.ethereumWriterConn.Connect(ctx)
	if err != nil {
		return fmt.Errorf("connect to ethereum for writer: %w", err)
	}

	err = relay.relaychainConn.Connect(ctx)
	if err != nil {
		return err
	}

	log.Info("Starting beefy listener")
	err = relay.beefyListener.Start(ctx, eg)
	if err != nil {
		return err
	}

	log.Info("Starting ethereum writer")
	err = relay.ethereumChannelWriter.Start(ctx, eg)
	if err != nil {
		return err
	}

	return nil
}
