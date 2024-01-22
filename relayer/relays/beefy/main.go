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

	initialBeefyBlock, initialValidatorSetID, err := relay.getCurrentState(ctx)
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

func (relay *Relay) getCurrentState(ctx context.Context) (uint64, uint64, error) {
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

func (relay *Relay) Initialize(ctx context.Context) error {
	err := relay.relaychainConn.Connect(ctx)
	if err != nil {
		return fmt.Errorf("create relaychain connection: %w", err)
	}

	err = relay.ethereumConn.Connect(ctx)
	if err != nil {
		return fmt.Errorf("create ethereum connection: %w", err)
	}
	return nil
}

func (relay *Relay) SyncUpdate(ctx context.Context, blockNumber uint64) error {
	initialBeefyBlock, initialValidatorSetID, err := relay.getCurrentState(ctx)
	if err != nil {
		return fmt.Errorf("fetch BeefyClient current state: %w", err)
	}
	if blockNumber > initialBeefyBlock {
		update, err := relay.polkadotListener.generateNextBeefyUpdate(blockNumber)
		if err != nil {
			return fmt.Errorf("fail to generate next beefy request: %w", err)
		}
		validatorSetID := update.SignedCommitment.Commitment.ValidatorSetID
		if validatorSetID != initialValidatorSetID && validatorSetID != initialValidatorSetID+1 {
			return fmt.Errorf("commitment has unexpected validatorSetID: blockNumber=%v validatorSetID=%v ValidatorSetIDInBeefyClient=%v",
				update.SignedCommitment.Commitment.BlockNumber,
				validatorSetID,
				initialValidatorSetID,
			)
		}
		err = relay.ethereumWriter.initialize(ctx)
		if err != nil {
			return fmt.Errorf("initialize EthereumWriter: %w", err)
		}
		err = relay.ethereumWriter.submit(ctx, update)
		if err != nil {
			return fmt.Errorf("fail to submit beefy update: %w", err)
		}
		updatedBeefyBlock, _, _ := relay.getCurrentState(ctx)
		if updatedBeefyBlock != uint64(update.SignedCommitment.Commitment.BlockNumber) {
			return fmt.Errorf("fail to sync beefy update")
		}
		log.WithFields(log.Fields{
			"initialValidatorSetID": initialValidatorSetID,
			"initialBeefyBlock":     initialBeefyBlock,
			"blockNumber":           blockNumber,
			"updatedBeefyBlock":     updatedBeefyBlock,
		}).Info("Sync beefy update success")
	}
	return nil
}
