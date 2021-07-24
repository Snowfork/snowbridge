package core

import (
	"github.com/sirupsen/logrus"

	"github.com/snowfork/polkadot-ethereum/relayer/workers"
	"github.com/snowfork/polkadot-ethereum/relayer/workers/beefy"
	"github.com/snowfork/polkadot-ethereum/relayer/workers/ethereum"
	"github.com/snowfork/polkadot-ethereum/relayer/workers/parachain"
)

type Relay struct{}

func (re *Relay) Run() error {
	config, err := LoadConfig()
	if err != nil {
		return err
	}

	var pool workers.WorkerPool

	if config.Workers.Ethereum.Enabled {
		ethereumFactory := func() (workers.Worker, *workers.WorkerConfig, error) {
			return ethereum.NewWorker(
				config.Global.DataDir,
				&config.Eth,
				&config.Parachain,
				logrus.WithField("worker", ethereum.Name),
			), &config.Workers.Ethereum, nil
		}
		pool = append(pool, ethereumFactory)
	}

	if config.Workers.Beefy.Enabled {
		beefyFactory := func() (workers.Worker, *workers.WorkerConfig, error) {
			beefyRelayer, err := beefy.NewWorker(
				&config.Relaychain,
				&config.Eth,
				logrus.WithField("worker", beefy.Name),
			)
			if err != nil {
				return nil, nil, err
			}
			return beefyRelayer, &config.Workers.Beefy, nil
		}
		pool = append(pool, beefyFactory)
	}

	if config.Workers.Parachain.Enabled {
		parachainFactory := func() (workers.Worker, *workers.WorkerConfig, error) {
			parachain, err := parachain.NewWorker(
				&config.Parachain,
				&config.Relaychain,
				&config.Eth,
				logrus.WithField("worker", parachain.Name),
			)
			if err != nil {
				return nil, nil, err
			}
			return parachain, &config.Workers.Parachain, nil
		}
		pool = append(pool, parachainFactory)
	}

	return pool.Run()
}
