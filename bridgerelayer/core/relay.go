// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package core

import (
	"context"
	"errors"
	"os"
	"os/signal"
	"syscall"

	"github.com/snowfork/polkadot-ethereum/bridgerelayer/chain"
	"github.com/snowfork/polkadot-ethereum/bridgerelayer/chain/ethereum"
	"github.com/snowfork/polkadot-ethereum/bridgerelayer/chain/substrate"
	"golang.org/x/sync/errgroup"

	log "github.com/sirupsen/logrus"
)

type Relay struct {
	chains []chain.Chain
}

func NewRelay() (*Relay, error) {

	// channel for messages from ethereum
	ethMessages := make(chan chain.Message, 1)

	// channel for messages from substrate
	subMessages := make(chan chain.Message, 1)

	ethChain, err := ethereum.NewChain(ethMessages, subMessages)
	if err != nil {
		return nil, err
	}

	subChain, err := substrate.NewChain(ethMessages, subMessages)
	if err != nil {
		return nil, err
	}

	return &Relay{
		chains: []chain.Chain{ethChain, subChain},
	}, nil
}

func (re *Relay) Start() {

	ctx, cancel := context.WithCancel(context.Background())
	eg, ctx := errgroup.WithContext(ctx)

	// Ensure clean termination upon SIGINT, SIGTERM
	eg.Go(func() error {
		notify := make(chan os.Signal, 1)
		signal.Notify(notify, syscall.SIGINT, syscall.SIGTERM)

		select {
		case <-ctx.Done():
			return ctx.Err()
		case sig := <-notify:
			log.WithField("signal", sig.String()).Info("Received signal")
			cancel()

		}

		return nil
	})

	for _, chain := range re.chains {
		err := chain.Start(ctx, eg)
		if err != nil {
			log.WithFields(log.Fields{
				"chain": chain.Name(),
				"error": err,
			}).Error("Failed to start chain")
			return
		}
		log.WithField("name", chain.Name()).Info("Started chain")
	}

	// Wait until a fatal error or signal is raised
	if err := eg.Wait(); err != nil {
		if !errors.Is(err, context.Canceled) {
			log.WithField("error", err).Error("Encountered an unrecoverable failure")
		}
	}

	// Shutdown chains
	for _, chain := range re.chains {
		chain.Stop()
	}
}
