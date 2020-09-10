// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package ethereum

import (
	"context"
	"fmt"

	"github.com/snowfork/polkadot-ethereum/bridgerelayer/chain"
	"golang.org/x/sync/errgroup"

	"github.com/sirupsen/logrus"
	"github.com/snowfork/polkadot-ethereum/bridgerelayer/crypto/secp256k1"
	"github.com/spf13/viper"
)

// Chain streams the Ethereum blockchain and routes tx data packets
type Chain struct {
	listener *Listener
	writer   *Writer
	conn     *Connection
}

const Name = "Ethereum"

// NewChain initializes a new instance of EthChain
func NewChain(ethMessages chan chain.Message, subMessages chan chain.Message) (*Chain, error) {
	log := logrus.WithField("chain", Name)

	// Validate and load configuration
	keys := []string{
		"ethereum.endpoint",
		"ethereum.private-key",
	}
	for _, key := range keys {
		if !viper.IsSet(key) {
			return nil, fmt.Errorf("config key %q not set", key)
		}
	}
	endpoint := viper.GetString("ethereum.endpoint")
	privateKey := viper.GetString("ethereum.private-key")

	kp, err := secp256k1.NewKeypairFromString(privateKey)
	if err != nil {
		return nil, err
	}

	conn := NewConnection(endpoint, kp, log)

	listener, err := NewListener(conn, ethMessages, log)
	if err != nil {
		return nil, err
	}

	writer, err := NewWriter(conn, subMessages, log)
	if err != nil {
		return nil, err
	}

	return &Chain{
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
