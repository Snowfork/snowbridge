// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package substrate

import (
	"context"

	"golang.org/x/sync/errgroup"

	"github.com/sirupsen/logrus"
	"github.com/snowfork/polkadot-ethereum/relayer/chain"
	"github.com/snowfork/polkadot-ethereum/relayer/crypto/sr25519"
)

type Chain struct {
	config   *Config
	listener *Listener
	writer   *Writer
	conn     *Connection
}

const Name = "Substrate"

func NewChain(config *Config, ethMessages chan chain.Message, subMessages chan chain.Message) (*Chain, error) {
	log := logrus.WithField("chain", Name)

	// Generate keypair from secret
	kp, err := sr25519.NewKeypairFromSeed(config.PrivateKey, "")
	if err != nil {
		return nil, err
	}

	conn := NewConnection(config.Endpoint, kp.AsKeyringPair(), log)

	listener := NewListener(
		config,
		conn,
		subMessages,
		log,
	)

	writer, err := NewWriter(conn, ethMessages, log)
	if err != nil {
		return nil, err
	}

	return &Chain{
		config:   config,
		conn:     conn,
		listener: listener,
		writer:   writer,
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
