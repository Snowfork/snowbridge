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
	log      *logrus.Entry
}

const Name = "Substrate"

func NewChain(config *Config) (*Chain, error) {
	log := logrus.WithField("chain", Name)

	// Generate keypair from secret
	kp, err := sr25519.NewKeypairFromSeed(config.PrivateKey, "")
	if err != nil {
		return nil, err
	}

	return &Chain{
		config:   config,
		conn:     NewConnection(config.Endpoint, kp.AsKeyringPair(), log),
		listener: nil,
		writer:   nil,
		log:      log,
	}, nil
}

func (ch *Chain) SetReceiver(ethMessages chan chain.Message, ethHeaders chan chain.Header) error {
	writer, err := NewWriter(ch.conn, ethMessages, ethHeaders, ch.log)
	if err != nil {
		return err
	}
	ch.writer = writer
	return nil
}

func (ch *Chain) SetSender(subMessages chan chain.Message, _ chan chain.Header) error {
	listener := NewListener(
		ch.config,
		ch.conn,
		subMessages,
		ch.log,
	)
	ch.listener = listener
	return nil
}

func (ch *Chain) Start(ctx context.Context, eg *errgroup.Group) error {
	err := ch.conn.Connect(ctx)
	if err != nil {
		return err
	}

	if ch.listener != nil {
		err = ch.listener.Start(ctx, eg)
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
