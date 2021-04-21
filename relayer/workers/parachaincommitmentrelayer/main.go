package parachaincommitmentrelayer

import (
	"context"
	"fmt"

	"golang.org/x/sync/errgroup"

	"github.com/sirupsen/logrus"
	"github.com/snowfork/polkadot-ethereum/relayer/chain"
	"github.com/snowfork/polkadot-ethereum/relayer/chain/ethereum"
	"github.com/snowfork/polkadot-ethereum/relayer/chain/parachain"
	"github.com/snowfork/polkadot-ethereum/relayer/chain/relaychain"
	"github.com/snowfork/polkadot-ethereum/relayer/contracts/inbound"
	"github.com/snowfork/polkadot-ethereum/relayer/crypto/secp256k1"
	"github.com/snowfork/polkadot-ethereum/relayer/substrate"
)

type Worker struct {
	parachainConfig             *parachain.Config
	relaychainConfig            *relaychain.Config
	ethereumConfig              *ethereum.Config
	parachainConn               *parachain.Connection
	relaychainConn              *relaychain.Connection
	parachainCommitmentListener *ParachainCommitmentListener
	ethereumConn                *ethereum.Connection
	ethereumChannelWriter       *EthereumChannelWriter
	log                         *logrus.Entry
}

const Name = "parachain-commitment-relayer"

func NewWorker(parachainConfig *parachain.Config,
	relaychainConfig *relaychain.Config, ethereumConfig *ethereum.Config) (*Worker, error) {
	log := logrus.WithField("parachain-commitment-relayer", Name)

	fmt.Println("Creating parachain-commitment-relayer")

	ethereumKp, err := secp256k1.NewKeypairFromString(ethereumConfig.PrivateKey)
	if err != nil {
		return nil, err
	}

	parachainConn := parachain.NewConnection(parachainConfig.Endpoint, nil, log)
	relaychainConn := relaychain.NewConnection(relaychainConfig.Endpoint, log)
	ethereumConn := ethereum.NewConnection(ethereumConfig.Endpoint, ethereumKp, log)

	// channel for messages from substrate
	var subMessages = make(chan []chain.Message, 1)

	parachainCommitmentListener := NewParachainCommitmentListener(
		parachainConfig,
		parachainConn,
		relaychainConn,
		subMessages,
		log,
	)

	contracts := make(map[substrate.ChannelID]*inbound.Contract)

	ethereumChannelWriter, err := NewEthereumChannelWriter(ethereumConfig, ethereumConn,
		subMessages, contracts, log)
	if err != nil {
		return nil, err
	}

	return &Worker{
		parachainConfig:             parachainConfig,
		relaychainConfig:            relaychainConfig,
		ethereumConfig:              ethereumConfig,
		parachainConn:               parachainConn,
		relaychainConn:              relaychainConn,
		parachainCommitmentListener: parachainCommitmentListener,
		ethereumConn:                ethereumConn,
		ethereumChannelWriter:       ethereumChannelWriter,
		log:                         log,
	}, nil
}

func (worker *Worker) Start(ctx context.Context, eg *errgroup.Group) error {
	fmt.Println("Starting parachain-commitment-relayer")

	if worker.parachainCommitmentListener == nil || worker.ethereumChannelWriter == nil {
		return fmt.Errorf("Sender and/or receiver need to be set before starting chain")
	}

	err := worker.parachainConn.Connect(ctx)
	if err != nil {
		return err
	}

	err = worker.ethereumConn.Connect(ctx)
	if err != nil {
		return err
	}

	if worker.parachainCommitmentListener != nil {
		err = worker.parachainCommitmentListener.Start(ctx, eg)
		if err != nil {
			return err
		}
	}

	eg.Go(func() error {
		if worker.ethereumChannelWriter != nil {
			err = worker.ethereumChannelWriter.Start(ctx, eg)
			if err != nil {
				return err
			}
		}

		return nil
	})

	return nil
}

func (worker *Worker) Stop() {
	if worker.parachainConn != nil {
		worker.parachainConn.Close()
	}
	if worker.relaychainConn != nil {
		worker.relaychainConn.Close()
	}
	if worker.ethereumConn != nil {
		worker.ethereumConn.Close()
	}
}

func (ch *Worker) Name() string {
	return Name
}
