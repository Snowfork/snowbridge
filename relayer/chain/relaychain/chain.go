// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package relaychain

import (
	"context"
	"fmt"

	"golang.org/x/sync/errgroup"

	"github.com/sirupsen/logrus"
	"github.com/snowfork/go-substrate-rpc-client/v2/types"
	"github.com/snowfork/polkadot-ethereum/relayer/chain"
	"github.com/snowfork/polkadot-ethereum/relayer/chain/ethereum"
	"github.com/snowfork/polkadot-ethereum/relayer/crypto/sr25519"
	"github.com/snowfork/polkadot-ethereum/relayer/store"
)

type Chain struct {
	config   *Config
	listener *Listener
	conn     *Connection
	log      *logrus.Entry
}

const Name = "Relaychain"

func NewChain(config *Config) (*Chain, error) {
	log := logrus.WithField("chain", Name)

	kp, err := sr25519.NewKeypairFromSeed(config.PrivateKey, "")
	if err != nil {
		return nil, err
	}

	return &Chain{
		config:   config,
		conn:     NewConnection(config.Endpoint, kp.AsKeyringPair(), log),
		listener: nil,
		log:      log,
	}, nil
}

func (ch *Chain) SetReceiver(_ <-chan []chain.Message, ethHeaders <-chan chain.Header, _ chan<- store.DatabaseCmd) error {
	return nil
}

func (ch *Chain) SetSender(subMessages chan<- []chain.Message, _ chan<- chain.Header, beefyMessages chan<- store.DatabaseCmd) error {
	listener := NewListener(
		ch.config,
		ch.conn,
		beefyMessages,
		ch.log,
	)
	ch.listener = listener
	return nil
}

func (ch *Chain) Start(ctx context.Context, eg *errgroup.Group, ethInit chan<- chain.Init, _ <-chan chain.Init) error {
	if ch.listener == nil {
		return fmt.Errorf("Sender needs to be set before starting chain")
	}

	err := ch.conn.Connect(ctx)
	if err != nil {
		return err
	}

	// The Ethereum chain needs init params from Substrate
	// to complete startup.
	ethInitHeaderID, err := ch.queryEthereumInitParams()
	if err != nil {
		return err
	}
	ch.log.WithFields(logrus.Fields{
		"blockNumber": ethInitHeaderID.Number,
		"blockHash":   ethInitHeaderID.Hash.Hex(),
	}).Info("Retrieved init params for Ethereum from Substrate")
	ethInit <- ethInitHeaderID
	close(ethInit)

	if ch.listener != nil {
		err = ch.listener.Start(ctx, eg)
		if err != nil {
			return err
		}
	}

	return nil
}

func (ch *Chain) Stop() {
	if ch.conn != nil {
		ch.conn.Close()
	}
}

func (ch *Chain) Name() string {
	return Name
}

func (ch *Chain) queryEthereumInitParams() (*ethereum.HeaderID, error) {
	storageKey, err := types.CreateStorageKey(&ch.conn.metadata, "VerifierLightclient", "FinalizedBlock", nil, nil)
	if err != nil {
		return nil, err
	}

	var headerID ethereum.HeaderID
	_, err = ch.conn.api.RPC.State.GetStorageLatest(storageKey, &headerID)
	if err != nil {
		return nil, err
	}

	nextHeaderID := ethereum.HeaderID{Number: types.NewU64(uint64(headerID.Number) + 1)}
	return &nextHeaderID, nil
}

func (ch *Chain) QueryCurrentEpoch() error {
	ch.log.Info("Creating storage key...")

	storageKey, err := types.CreateStorageKey(&ch.conn.metadata, "Babe", "Epoch", nil, nil)
	if err != nil {
		return err
	}

	ch.log.Info("Attempting to query current epoch...")

	// var headerID ethereum.HeaderID
	var epochData interface{}
	_, err = ch.conn.api.RPC.State.GetStorageLatest(storageKey, &epochData)
	if err != nil {
		return err
	}

	ch.log.Info("Retrieved current epoch data:", epochData)

	// nextHeaderID := ethereum.HeaderID{Number: types.NewU64(uint64(headerID.Number) + 1)}
	// return &nextHeaderID, nil

	return nil
}
