package beefy

import (
	"context"
	"fmt"

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
	err := relay.relaychainConn.Connect(ctx)
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

	requests, err := relay.polkadotListener.Start(ctx, eg, initialState.LatestBeefyBlock, initialState.CurrentValidatorSetID)
	if err != nil {
		return fmt.Errorf("initialize polkadot listener: %w", err)
	}

	err = relay.ethereumWriter.Start(ctx, eg, requests)
	if err != nil {
		return fmt.Errorf("start ethereum writer: %w", err)
	}

	return nil
}

func (relay *Relay) OneShotSync(ctx context.Context, blockNumber uint64, generateLog bool) error {
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
	if blockNumber <= state.LatestBeefyBlock {
		log.WithFields(log.Fields{
			"validatorSetID": state.CurrentValidatorSetID,
			"beefyBlock":     state.LatestBeefyBlock,
			"relayBlock":     blockNumber,
		}).Info("Relay block already synced, just ignore")
		return nil
	}

	// generate beefy update for that specific relay block
	task, err := relay.polkadotListener.generateBeefyUpdate(blockNumber)
	if err != nil {
		return fmt.Errorf("fail to generate next beefy request: %w", err)
	}

	// Ignore commitment earlier than LatestBeefyBlock which is outdated
	if task.SignedCommitment.Commitment.BlockNumber <= uint32(state.LatestBeefyBlock) {
		log.WithFields(log.Fields{
			"latestBeefyBlock":      state.LatestBeefyBlock,
			"currentValidatorSetID": state.CurrentValidatorSetID,
			"nextValidatorSetID":    state.NextValidatorSetID,
			"blockNumberToSync":     task.SignedCommitment.Commitment.BlockNumber,
		}).Info("Commitment outdated, just ignore")
		return nil
	}
	if task.SignedCommitment.Commitment.ValidatorSetID > state.NextValidatorSetID {
		log.WithFields(log.Fields{
			"latestBeefyBlock":      state.LatestBeefyBlock,
			"currentValidatorSetID": state.CurrentValidatorSetID,
			"nextValidatorSetID":    state.NextValidatorSetID,
			"validatorSetIDToSync":  task.SignedCommitment.Commitment.ValidatorSetID,
		}).Warn("Task unexpected, wait for mandatory updates to catch up first")
		return nil
	}

	// Submit the task
	if task.SignedCommitment.Commitment.ValidatorSetID == state.CurrentValidatorSetID {
		task.ValidatorsRoot = state.CurrentValidatorSetRoot
	} else {
		task.ValidatorsRoot = state.NextValidatorSetRoot
	}
	task.GenerateLog = generateLog
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
