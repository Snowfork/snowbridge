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
	"github.com/snowfork/polkadot-ethereum/relayer/crypto/secp256k1"
	"github.com/snowfork/polkadot-ethereum/relayer/crypto/sr25519"
	"github.com/snowfork/polkadot-ethereum/relayer/relaychain"
)

type Chain struct {
	config   *Config
	listener *Listener
	writer   *Writer
	conn     *Connection
	econn    *ethereum.Connection
	log      *logrus.Entry
}

const Name = "Relaychain"

func NewChain(config *Config) (*Chain, error) {
	log := logrus.WithField("chain", Name)

	// Generate keypair from secret
	kpPara, err := sr25519.NewKeypairFromSeed(config.Relaychain.PrivateKey, "")
	if err != nil {
		return nil, err
	}

	kpEth, err := secp256k1.NewKeypairFromString(config.Ethereum.PrivateKey)
	if err != nil {
		return nil, err
	}

	return &Chain{
		config:   config,
		conn:     NewConnection(config.Relaychain.Endpoint, kpPara.AsKeyringPair(), log),
		econn:    ethereum.NewConnection(config.Ethereum.Endpoint, kpEth, log),
		listener: nil,
		writer:   nil,
		log:      log,
	}, nil
}

func (ch *Chain) SetReceiver(messages <-chan []chain.Message, ethHeaders <-chan chain.Header,
	beefy chan relaychain.BeefyCommitmentInfo) error {

	writer, err := NewWriter(ch.config, ch.conn, ch.econn, messages, beefy, ch.log)
	if err != nil {
		return err
	}
	ch.writer = writer
	return nil
}

func (ch *Chain) SetSender(subMessages chan<- []chain.Message, _ chan<- chain.Header,
	beefy chan relaychain.BeefyCommitmentInfo) error {
	listener := NewListener(
		ch.config,
		ch.conn,
		ch.econn,
		subMessages,
		beefy,
		ch.log,
	)
	ch.listener = listener
	return nil
}

func (ch *Chain) Start(ctx context.Context, eg *errgroup.Group, ethInit chan<- chain.Init, _ <-chan chain.Init) error {
	if ch.listener == nil && ch.writer == nil {
		return fmt.Errorf("Sender and/or receiver need to be set before starting chain")
	}

	err := ch.conn.Connect(ctx)
	if err != nil {
		return err
	}

	err = ch.econn.Connect(ctx)
	if err != nil {
		return err
	}

	// The Ethereum chain needs init params from Parachain
	// to complete startup.
	ethInitHeaderID, err := ch.queryEthereumInitParams()
	if err != nil {
		return err
	}

	ch.log.WithFields(logrus.Fields{
		"blockNumber": ethInitHeaderID.Number,
		"blockHash":   ethInitHeaderID.Hash.Hex(),
	}).Info("Retrieved init params for Ethereum from Relaychain")
	ethInit <- ethInitHeaderID
	close(ethInit)

	if ch.listener != nil {
		err = ch.listener.Start(ctx, eg, uint64(ethInitHeaderID.Number), uint64(ch.config.Ethereum.BeefyBlockDelay))
		if err != nil {
			return err
		}
	}

	if ch.writer != nil {
		err = ch.writer.Start(ctx, eg)
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

	finalizedHash, err := ch.conn.api.RPC.Chain.GetFinalizedHead()
	if err != nil {
		return nil, err
	}

	finalizedBlock, err := ch.conn.api.RPC.Chain.GetBlock(finalizedHash)
	if err != nil {
		return nil, err
	}

	finalizedHashEncoded := types.NewH256(types.MustHexDecodeString(finalizedHash.Hex()))

	headerID := ethereum.HeaderID{
		Number: types.NewU64(uint64(finalizedBlock.Block.Header.Number)),
		Hash:   finalizedHashEncoded,
	}

	fmt.Printf("\nRelaychain header ID: %v\n", headerID)
	return &headerID, nil
}

func (ch *Chain) QueryCurrentEpoch() error {
	ch.log.Info("Creating storage key...")

	storageKey, err := types.CreateStorageKey(&ch.conn.metadata, "Babe", "Epoch", nil, nil)
	if err != nil {
		return err
	}

	ch.log.Info("Attempting to query current epoch...")

	var epochData interface{}
	_, err = ch.conn.api.RPC.State.GetStorageLatest(storageKey, &epochData)
	if err != nil {
		return err
	}

	ch.log.Info("Retrieved current epoch data:", epochData)

	return nil
}
