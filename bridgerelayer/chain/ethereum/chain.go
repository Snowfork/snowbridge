// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package ethereum

import (
	"context"

	"github.com/snowfork/polkadot-ethereum/bridgerelayer/chain"
	"golang.org/x/sync/errgroup"

	"github.com/sirupsen/logrus"
	"github.com/snowfork/polkadot-ethereum/bridgerelayer/crypto/secp256k1"
)

// Chain streams the Ethereum blockchain and routes tx data packets
type Chain struct {
	config   *Config
	listener *Listener
	writer   *Writer
	conn     *Connection
}

const Name = "Ethereum"

// NewChain initializes a new instance of EthChain
func NewChain(config *Config, ethMessages chan chain.Message, subMessages chan chain.Message) (*Chain, error) {
	log := logrus.WithField("chain", Name)

	bridgeContract, err := LoadBridgeContract(config)
	if err != nil {
		return nil, err
	}

	appContracts, err := LoadAppContracts(config)
	if err != nil {
		return nil, err
	}

	kp, err := secp256k1.NewKeypairFromString(config.PrivateKey)
	if err != nil {
		return nil, err
	}

	conn := NewConnection(config.Endpoint, kp, log)

	listener, err := NewListener(conn, ethMessages, appContracts, log)
	if err != nil {
		return nil, err
	}

	writer, err := NewWriter(conn, subMessages, bridgeContract, log)
	if err != nil {
		return nil, err
	}

	return &Chain{
		config:   config,
		listener: listener,
		writer:   writer,
		conn:     conn,
	}, nil
}

func (ch *Chain) Start(ctx context.Context, eg *errgroup.Group) error {
	err := ch.conn.Connect(ctx)
	if err != nil {
		return err
	}

	err = ch.listener.Start(ctx, eg)
	if err != nil {
		return err
	}

	err = ch.writer.Start(ctx, eg)
	if err != nil {
		return err
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
