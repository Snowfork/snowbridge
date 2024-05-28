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

	relayer := Relay{
		config:           config,
		relaychainConn:   relaychainConn,
		ethereumConn:     ethereumConn,
		polkadotListener: polkadotListener,
		ethereumWriter:   ethereumWriter,
	}
	polkadotListener.relayer = &relayer
	return &relayer, nil
}

func (relay *Relay) Start(ctx context.Context, eg *errgroup.Group) error {
	err := relay.relaychainConn.Connect(ctx)
	if err != nil {
		return fmt.Errorf("create relaychain connection: %w", err)
	}

	err = relay.ethereumConn.Connect(ctx)
	if err != nil {
		return fmt.Errorf("create ethereum connection: %w", err)
	}

	currentState, err := relay.CurrentState(ctx)
	if err != nil {
		return fmt.Errorf("fetch BeefyClient current state: %w", err)
	}
	log.WithFields(log.Fields{
		"currentState": currentState,
	}).Info("Retrieved current BeefyClient state")

	requests, err := relay.polkadotListener.Start(ctx, eg, currentState)
	if err != nil {
		return fmt.Errorf("initialize polkadot listener: %w", err)
	}

	err = relay.ethereumWriter.Start(ctx, eg, requests)
	if err != nil {
		return fmt.Errorf("initialize ethereum writer: %w", err)
	}

	return nil
}

func (relay *Relay) CurrentState(ctx context.Context) (BeefyState, error) {
	var currentState BeefyState
	address := common.HexToAddress(relay.config.Sink.Contracts.BeefyClient)
	beefyClient, err := contracts.NewBeefyClient(address, relay.ethereumConn.Client())
	if err != nil {
		return currentState, err
	}

	callOpts := bind.CallOpts{
		Context: ctx,
	}

	latestBeefyBlock, err := beefyClient.LatestBeefyBlock(&callOpts)
	if err != nil {
		return currentState, err
	}

	currentValidatorSet, err := beefyClient.CurrentValidatorSet(&callOpts)
	if err != nil {
		return currentState, err
	}

	nextValidatorSet, err := beefyClient.NextValidatorSet(&callOpts)
	if err != nil {
		return currentState, err
	}

	currentState = BeefyState{
		LatestBeefyBlock:        latestBeefyBlock,
		CurrentValidatorSetId:   currentValidatorSet.Id.Uint64(),
		CurrentValidatorSetRoot: currentValidatorSet.Root,
		NextValidatorSetId:      nextValidatorSet.Id.Uint64(),
		NextValidatorSetRoot:    nextValidatorSet.Root,
	}

	return currentState, nil
}
