package beefy

import (
	"context"
	"fmt"
	"time"

	"golang.org/x/sync/errgroup"

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
}

func NewRelay(config *Config, ethereumKeypair *secp256k1.Keypair) (*Relay, error) {
	relaychainConn := relaychain.NewConnection(config.Source.Polkadot.Endpoint)
	ethereumConn := ethereum.NewConnection(&config.Sink.Ethereum, ethereumKeypair)

	polkadotListener := NewPolkadotListener(
		&config.Source,
		relaychainConn,
	)

	ethereumWriter := NewEthereumWriter(&config.Sink, ethereumConn)

	log.Info("Beefy relay created")

	return &Relay{
		config:           config,
		relaychainConn:   relaychainConn,
		ethereumConn:     ethereumConn,
		polkadotListener: polkadotListener,
		ethereumWriter:   ethereumWriter,
	}, nil
}

func (relay *Relay) Start(ctx context.Context, eg *errgroup.Group) error {
	err := relay.relaychainConn.ConnectWithHeartBeat(ctx, 30*time.Second)
	if err != nil {
		return fmt.Errorf("create relaychain connection: %w", err)
	}

	err = relay.ethereumConn.Connect(ctx)
	if err != nil {
		return fmt.Errorf("create ethereum connection: %w", err)
	}
	err = relay.ethereumWriter.initialize(ctx)
	if err != nil {
		return fmt.Errorf("initialize ethereum writer: %w", err)
	}

	initialState, err := relay.ethereumWriter.queryBeefyClientState(ctx)
	if err != nil {
		return fmt.Errorf("fetch BeefyClient current state: %w", err)
	}
	log.WithFields(log.Fields{
		"beefyBlock":     initialState.LatestBeefyBlock,
		"validatorSetID": initialState.CurrentValidatorSetID,
	}).Info("Retrieved current BeefyClient state")

	requests, err := relay.polkadotListener.Start(ctx, eg, initialState.LatestBeefyBlock)
	if err != nil {
		return fmt.Errorf("initialize polkadot listener: %w", err)
	}

	err = relay.ethereumWriter.Start(ctx, eg, requests)
	if err != nil {
		return fmt.Errorf("start ethereum writer: %w", err)
	}

	return nil
}
