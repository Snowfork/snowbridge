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

	"github.com/sirupsen/logrus"
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
	err := relay.relaychainConn.Connect(ctx)
	if err != nil {
		return fmt.Errorf("create relaychain connection: %w", err)
	}

	err = relay.ethereumConn.Connect(ctx)
	if err != nil {
		return fmt.Errorf("create ethereum connection: %w", err)
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
	beefyClient, err := contracts.NewBeefyClient(address, relay.ethereumConn.Client())
	if err != nil {
		return 0, 0, err
	}

	callOpts := bind.CallOpts{
		Context: ctx,
	}

	latestBeefyBlock, err := beefyClient.LatestBeefyBlock(&callOpts)
	if err != nil {
		return 0, 0, err
	}

	currentValidatorSet, err := beefyClient.CurrentValidatorSet(&callOpts)
	if err != nil {
		return 0, 0, err
	}

	return latestBeefyBlock, currentValidatorSet.Id.Uint64(), nil
}

func (relay *Relay) SyncUpdate(ctx context.Context, relayBlockNumber uint64) error {
	// Initialize relaychainConn
	err := relay.relaychainConn.Connect(ctx)
	if err != nil {
		return fmt.Errorf("create relaychain connection: %w", err)
	}

	// Initialize ethereumConn
	err = relay.ethereumConn.Connect(ctx)
	if err != nil {
		return fmt.Errorf("create ethereum connection: %w", err)
	}
	err = relay.ethereumWriter.initialize(ctx)
	if err != nil {
		return fmt.Errorf("initialize EthereumWriter: %w", err)
	}

	state, err := relay.ethereumWriter.queryBeefyClientState(ctx)
	if err != nil {
		return fmt.Errorf("query beefy client state: %w", err)
	}
	// Ignore relay block already synced
	if relayBlockNumber <= state.LatestBeefyBlock {
		log.WithFields(log.Fields{
			"validatorSetID": state.CurrentValidatorSetID,
			"beefyBlock":     state.LatestBeefyBlock,
			"relayBlock":     relayBlockNumber,
		}).Info("Relay block already synced, just ignore")
		return nil
	}

	// generate beefy update for that specific relay block
	task, err := relay.polkadotListener.generateBeefyUpdate(relayBlockNumber)
	if err != nil {
		return fmt.Errorf("fail to generate next beefy request: %w", err)
	}

	// Ignore commitment already synced
	if task.SignedCommitment.Commitment.BlockNumber <= uint32(state.LatestBeefyBlock) {
		log.WithFields(logrus.Fields{
			"beefyBlockNumber": task.SignedCommitment.Commitment.BlockNumber,
			"beefyBlockSynced": state.LatestBeefyBlock,
		}).Info("New commitment already synced, just ignore")
		return nil
	}

	// Submit the task
	if task.SignedCommitment.Commitment.ValidatorSetID == state.CurrentValidatorSetID {
		task.ValidatorsRoot = state.CurrentValidatorSetRoot
	} else {
		task.ValidatorsRoot = state.NextValidatorSetRoot
	}
	err = relay.ethereumWriter.submit(ctx, task)
	if err != nil {
		return fmt.Errorf("fail to submit beefy update: %w", err)
	}

	updatedState, err := relay.ethereumWriter.queryBeefyClientState(ctx)
	if err != nil {
		return fmt.Errorf("query beefy client state: %w", err)
	}
	log.WithFields(log.Fields{
		"latestBeefyBlock":      updatedState.LatestBeefyBlock,
		"currentValidatorSetID": updatedState.CurrentValidatorSetID,
		"nextValidatorSetID":    updatedState.NextValidatorSetID,
	}).Info("Sync beefy update success")
	return nil
}
