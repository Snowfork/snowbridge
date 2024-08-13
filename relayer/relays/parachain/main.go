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
	ethereumConnWriter    *ethereum.Connection
	ethereumConnBeefy     *ethereum.Connection
	ethereumChannelWriter *EthereumWriter
	beefyListener         *BeefyListener
}

func NewRelay(config *Config, keypair *secp256k1.Keypair) (*Relay, error) {
	log.Info("Creating worker")

	parachainConn := parachain.NewConnection(config.Source.Parachain.Endpoint, nil)
	relaychainConn := relaychain.NewConnection(config.Source.Polkadot.Endpoint)

	ethereumConnWriter := ethereum.NewConnection(&config.Sink.Ethereum, keypair)
	ethereumConnBeefy := ethereum.NewConnection(&config.Source.Ethereum, keypair)

	// channel for messages from beefy listener to ethereum writer
	var tasks = make(chan *Task, 1)

	ethereumChannelWriter, err := NewEthereumWriter(
		&config.Sink,
		ethereumConnWriter,
		tasks,
	)
	if err != nil {
		return nil, err
	}

	beefyListener := NewBeefyListener(
		&config.Source,
		&config.Schedule,
		ethereumConnBeefy,
		relaychainConn,
		parachainConn,
		tasks,
	)

	return &Relay{
		config:                config,
		parachainConn:         parachainConn,
		relaychainConn:        relaychainConn,
		ethereumConnWriter:    ethereumConnWriter,
		ethereumConnBeefy:     ethereumConnBeefy,
		ethereumChannelWriter: ethereumChannelWriter,
		beefyListener:         beefyListener,
	}, nil
}

func (relay *Relay) Start(ctx context.Context, eg *errgroup.Group) error {
	err := relay.parachainConn.Connect(ctx)
	if err != nil {
		return err
	}

	err = relay.ethereumConnWriter.Connect(ctx)
	if err != nil {
		return fmt.Errorf("unable to connect to ethereum: writer: %w", err)
	}

	err = relay.ethereumConnBeefy.Connect(ctx)
	if err != nil {
		return fmt.Errorf("unable to connect to ethereum: beefy: %w", err)
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

	log.Info("Current relay's ID:", relay.config.Schedule.ID)

	return nil
}
