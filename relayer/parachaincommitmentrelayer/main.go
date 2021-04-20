package parachaincommitmentrelayer

import (
	"context"
	"fmt"

	"golang.org/x/sync/errgroup"

	"github.com/sirupsen/logrus"
	"github.com/snowfork/polkadot-ethereum/relayer/chain"
	"github.com/snowfork/polkadot-ethereum/relayer/chain/ethereum"
	"github.com/snowfork/polkadot-ethereum/relayer/chain/parachain"
	"github.com/snowfork/polkadot-ethereum/relayer/contracts/inbound"
	"github.com/snowfork/polkadot-ethereum/relayer/crypto/secp256k1"
	"github.com/snowfork/polkadot-ethereum/relayer/substrate"
)

type Worker struct {
	parachainConfig   *parachain.Config
	ethereumConfig    *ethereum.Config
	parachainConn     *parachain.Connection
	parachainListener *parachain.Listener
	ethereumConn      *ethereum.Connection
	ethereumWriter    *EthereumWriter
	log               *logrus.Entry
}

const Name = "parachain-commitment-relayer"

func NewWorker(parachainConfig *parachain.Config, ethereumConfig *ethereum.Config) (*Worker, error) {
	log := logrus.WithField("parachain-commitment-relayer", Name)

	fmt.Println("Creating parachain-commitment-relayer")

	ethereumKp, err := secp256k1.NewKeypairFromString(ethereumConfig.PrivateKey)
	if err != nil {
		return nil, err
	}

	parachainConn := parachain.NewConnection(parachainConfig.Endpoint, nil, log)
	ethereumConn := ethereum.NewConnection(ethereumConfig.Endpoint, ethereumKp, log)
	// channel for messages from substrate
	var subMessages = make(chan []chain.Message, 1)

	parachainListener := parachain.NewListener(
		parachainConfig,
		parachainConn,
		subMessages,
		log,
	)

	contracts := make(map[substrate.ChannelID]*inbound.Contract)

	ethereumWriter, err := NewEthereumWriter(ethereumConfig, ethereumConn,
		subMessages, contracts, log)
	if err != nil {
		return nil, err
	}

	return &Worker{
		parachainConfig:   parachainConfig,
		ethereumConfig:    ethereumConfig,
		parachainConn:     parachainConn,
		parachainListener: parachainListener,
		ethereumConn:      ethereumConn,
		ethereumWriter:    ethereumWriter,
		log:               log,
	}, nil
}

func (worker *Worker) Start(ctx context.Context, eg *errgroup.Group) error {
	fmt.Println("Starting parachain-commitment-relayer")

	if worker.parachainListener == nil || worker.ethereumWriter == nil {
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

	if worker.parachainListener != nil {
		err = worker.parachainListener.Start(ctx, eg)
		if err != nil {
			return err
		}
	}

	eg.Go(func() error {
		if worker.ethereumWriter != nil {
			err = worker.ethereumWriter.Start(ctx, eg)
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
	if worker.ethereumConn != nil {
		worker.ethereumConn.Close()
	}
}

func (ch *Worker) Name() string {
	return Name
}
