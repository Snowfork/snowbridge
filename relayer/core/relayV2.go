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

	ethrelayer := ethrelayer.NewWorker(
		&config.Eth,
		&config.Parachain,
		logrus.WithField("worker", ethrelayer.Name),
	)

	relayWorkers := make([]workers.Worker, 0)
	// TODO: add all workers here
	relayWorkers = append(relayWorkers, ethrelayer)

	pool := workers.NewWorkerPool(relayWorkers)

	return pool.Run()
}
