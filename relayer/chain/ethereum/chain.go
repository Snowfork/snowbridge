// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package ethereum

import (
	"context"
	"fmt"

	"github.com/snowfork/polkadot-ethereum/relayer/chain"
	"golang.org/x/sync/errgroup"

	"github.com/sirupsen/logrus"
	"github.com/snowfork/polkadot-ethereum/relayer/crypto/secp256k1"
)

// Chain streams the Ethereum blockchain and routes tx data packets
type Chain struct {
	config   *Config
	listener *Listener
	writer   *Writer
	conn     *Connection
	log      *logrus.Entry
}

const Name = "Ethereum"

// NewChain initializes a new instance of EthChain
func NewChain(config *Config) (*Chain, error) {
	log := logrus.WithField("chain", Name)

	kp, err := secp256k1.NewKeypairFromString(config.PrivateKey)
	if err != nil {
		return nil, err
	}

	return &Chain{
		config:   config,
		listener: nil,
		writer:   nil,
		conn:     NewConnection(config.Endpoint, kp, log),
		log:      log,
	}, nil
}

func (ch *Chain) SetReceiver(subMessages <-chan chain.Message, _ <-chan chain.Header) error {
	bridgeContract, err := LoadBridgeContract(ch.config)
	if err != nil {
		return err
	}

	writer, err := NewWriter(ch.conn, subMessages, bridgeContract, ch.log)
	if err != nil {
		return err
	}
	ch.writer = writer

	return nil
}

func (ch *Chain) SetSender(ethMessages chan<- chain.Message, ethHeaders chan<- chain.Header) error {
	appContracts, err := LoadAppContracts(ch.config)
	if err != nil {
		return err
	}

	listener, err := NewListener(ch.conn, ethMessages, ethHeaders, appContracts, ch.log)
	if err != nil {
		return err
	}
	ch.listener = listener

	return nil
}

func (ch *Chain) Start(ctx context.Context, eg *errgroup.Group, subInit chan<- chain.Init, ethInit <-chan chain.Init) error {
	if ch.listener == nil && ch.writer == nil {
		return fmt.Errorf("Sender and/or receiver need to be set before starting chain")
	}

	err := ch.conn.Connect(ctx)
	if err != nil {
		return err
	}

	// If the Substrate chain needs init params from Ethereum,
	// retrieve them here and send to subInit before closing.
	close(subInit)

	eg.Go(func() error {
		ethInitHeaderID := (<-ethInit).(*HeaderID)
		ch.log.WithFields(logrus.Fields{
			"blockNumber": ethInitHeaderID.Number,
			"blockHash":   ethInitHeaderID.Hash.Hex(),
		}).Debug("Received init params for Ethereum from Substrate")

		if ch.listener != nil {
			err = ch.listener.Start(ctx, eg, uint64(ethInitHeaderID.Number), uint64(ch.config.DescendantsUntilFinal))
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
	})

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
