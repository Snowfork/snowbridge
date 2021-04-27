// Copyright 2021 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package core

import (
	"github.com/sirupsen/logrus"

	"github.com/snowfork/polkadot-ethereum/relayer/workers"
	"github.com/snowfork/polkadot-ethereum/relayer/workers/ethrelayer"
)

type RelayV2 struct{}

func (re *RelayV2) Run() error {
	config, err := LoadConfig()
	if err != nil {
		return err
	}

	ethrelayerFactory := func() (workers.Worker, *workers.WorkerConfig, error) {
		return ethrelayer.NewWorker(
			&config.Eth,
			&config.Parachain,
			logrus.WithField("worker", ethrelayer.Name),
		), &config.Workers.EthRelayer, nil
	}

	// TODO: add all workers
	pool := workers.WorkerPool{
		ethrelayerFactory,
	}

	return pool.Run()
}
