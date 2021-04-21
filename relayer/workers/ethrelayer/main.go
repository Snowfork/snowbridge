package ethrelayer

import (
	"context"
	"fmt"

	"golang.org/x/sync/errgroup"

	"github.com/sirupsen/logrus"
	"github.com/snowfork/go-substrate-rpc-client/v2/types"
	"github.com/snowfork/polkadot-ethereum/relayer/chain"
	"github.com/snowfork/polkadot-ethereum/relayer/chain/ethereum"
	"github.com/snowfork/polkadot-ethereum/relayer/chain/parachain"
	"github.com/snowfork/polkadot-ethereum/relayer/crypto/sr25519"
)

type Worker struct {
	parachainConfig  *parachain.Config
	ethereumConfig   *ethereum.Config
	parachainConn    *parachain.Connection
	ethereumConn     *ethereum.Connection
	ethereumListener *EthereumListener
	parachainWriter  *ParachainWriter
	log              *logrus.Entry
}

const Name = "eth-relayer"

func NewWorker(ethereumConfig *ethereum.Config, parachainConfig *parachain.Config) (*Worker, error) {
	log := logrus.WithField("worker", Name)

	log.Info("Creating worker")

	ethereumConn := ethereum.NewConnection(ethereumConfig.Endpoint, nil, log)

	// Generate keypair from secret
	parachainKeypair, err := sr25519.NewKeypairFromSeed(parachainConfig.PrivateKey, "")
	if err != nil {
		return nil, err
	}
	parachainConn := parachain.NewConnection(parachainConfig.Endpoint, parachainKeypair.AsKeyringPair(), log)

	// channel for messages from ethereum
	ethMessages := make(chan []chain.Message, 1)
	// channel for headers from ethereum (it's a blocking channel so that we
	// can guarantee that a header is forwarded before we send dependent messages)
	ethHeaders := make(chan chain.Header)

	ethereumListener, err := NewEthereumListener(ethereumConfig, ethereumConn, ethMessages, ethHeaders, log)
	if err != nil {
		return nil, err
	}

	parachainWriter, err := NewParachainWriter(parachainConn, ethMessages, ethHeaders, log)
	if err != nil {
		return nil, err
	}

	return &Worker{
		ethereumConfig:   ethereumConfig,
		ethereumConn:     ethereumConn,
		parachainConfig:  parachainConfig,
		parachainConn:    parachainConn,
		ethereumListener: ethereumListener,
		parachainWriter:  parachainWriter,
		log:              log,
	}, nil

}

func (worker *Worker) Start(ctx context.Context, eg *errgroup.Group) error {
	worker.log.Info("Starting worker")

	if worker.ethereumListener == nil || worker.parachainWriter == nil {
		return fmt.Errorf("Sender and/or receiver need to be set before starting chain")
	}

	err := worker.ethereumConn.Connect(ctx)
	if err != nil {
		return err
	}

	// Short-lived channels that communicate initialization parameters
	// between the two chains. The chains close them after startup.
	ethInit := make(chan chain.Init)

	eg.Go(func() error {
		ethInitHeaderID := (<-ethInit).(*ethereum.HeaderID)
		worker.log.WithFields(logrus.Fields{
			"blockNumber": ethInitHeaderID.Number,
			"blockHash":   ethInitHeaderID.Hash.Hex(),
		}).Debug("Received init params for Ethereum from Substrate")

		if worker.ethereumListener != nil {
			err = worker.ethereumListener.Start(ctx, eg, uint64(ethInitHeaderID.Number),
				uint64(worker.ethereumConfig.DescendantsUntilFinal))
			if err != nil {
				return err
			}
		}

		return nil
	})

	err = worker.parachainConn.Connect(ctx)
	if err != nil {
		return err
	}

	// The Ethereum chain needs init params from Substrate
	// to complete startup.
	ethInitHeaderID, err := worker.queryEthereumInitParams()
	if err != nil {
		return err
	}
	worker.log.WithFields(logrus.Fields{
		"blockNumber": ethInitHeaderID.Number,
		"blockHash":   ethInitHeaderID.Hash.Hex(),
	}).Info("Retrieved init params for Ethereum from Substrate")
	ethInit <- ethInitHeaderID
	close(ethInit)

	if worker.parachainWriter != nil {
		err = worker.parachainWriter.Start(ctx, eg)
		if err != nil {
			return err
		}
	}

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

func (worker *Worker) queryEthereumInitParams() (*ethereum.HeaderID, error) {
	storageKey, err := types.CreateStorageKey(worker.parachainConn.GetMetadata(), "VerifierLightclient", "FinalizedBlock", nil, nil)
	if err != nil {
		return nil, err
	}

	var headerID ethereum.HeaderID
	_, err = worker.parachainConn.GetAPI().RPC.State.GetStorageLatest(storageKey, &headerID)
	if err != nil {
		return nil, err
	}

	nextHeaderID := ethereum.HeaderID{Number: types.NewU64(uint64(headerID.Number) + 1)}
	return &nextHeaderID, nil
}
