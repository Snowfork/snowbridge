package beefy

import (
	"context"
	"fmt"

	"golang.org/x/sync/errgroup"

	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"
	"github.com/snowfork/snowbridge/relayer/chain/ethereum"
	"github.com/snowfork/snowbridge/relayer/chain/relaychain"
	"github.com/snowfork/snowbridge/relayer/contracts"
	"github.com/snowfork/snowbridge/relayer/crypto/secp256k1"

	log "github.com/sirupsen/logrus"
)

type Relay struct {
	config           *Config
	relaychainConn   *relaychain.Connection
	ethereumConn     *ethereum.Connection
	polkadotListener *PolkadotListener
	ethereumWriter   *EthereumWriter
	tasks            chan Request
}

func NewRelay(config *Config, ethereumKeypair *secp256k1.Keypair) (*Relay, error) {
	log.Info("Relay created")

	relaychainConn := relaychain.NewConnection(config.Source.Polkadot.Endpoint)
	ethereumConn := ethereum.NewConnection(config.Sink.Ethereum.Endpoint, ethereumKeypair)

	polkadotListener := NewPolkadotListener(
		&config.Source,
		relaychainConn,
	)

	ethereumWriter := NewEthereumWriter(&config.Sink, ethereumConn)

	return &Relay{
		config:           config,
		relaychainConn:   relaychainConn,
		ethereumConn:     ethereumConn,
		polkadotListener: polkadotListener,
		ethereumWriter:   ethereumWriter,
		tasks:            make(chan Request),
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

	initialBeefyBlock, initialValidatorSetID, err := relay.getInitialState(ctx)
	if err != nil {
		return fmt.Errorf("fetch BeefyClient current state: %w", err)
	}

	log.WithFields(log.Fields{
		"beefyBlock":     initialBeefyBlock,
		"validatorSetID": initialValidatorSetID,
	}).Info("Retrieved current BeefyClient state")

	requests, err := relay.polkadotListener.Start(ctx, eg, initialBeefyBlock, initialValidatorSetID)
	if err != nil {
		return fmt.Errorf("initialize polkadot listener: %w", err)
	}

	err = relay.ethereumWriter.Start(ctx, eg, requests)
	if err != nil {
		return fmt.Errorf("initialize ethereum writer: %w", err)
	}

	return nil
}

func (relay *Relay) getInitialState(ctx context.Context) (uint64, uint64, error) {
	address := common.HexToAddress(relay.config.Sink.Contracts.BeefyClient)
	contract, err := contracts.NewBeefyClient(address, relay.ethereumConn.Client())
	if err != nil {
		return 0, 0, err
	}

	callOpts := bind.CallOpts{
		Context: ctx,
	}

	initialBeefyBlock, err := contract.LatestBeefyBlock(&callOpts)
	if err != nil {
		return 0, 0, err
	}

	initialValidatorSetID, err := contract.CurrentValidatorSet(&callOpts)
	if err != nil {
		return 0, 0, err
	}

	return initialBeefyBlock, initialValidatorSetID.Id.Uint64(), nil
}
