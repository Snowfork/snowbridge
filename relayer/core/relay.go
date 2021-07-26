package core

import (
	"github.com/sirupsen/logrus"

	worker "github.com/snowfork/snowbridge/relayer/relays"
	beefyRelay "github.com/snowfork/snowbridge/relayer/relays/beefy"
	ethereumRelay "github.com/snowfork/snowbridge/relayer/relays/ethereum"
	parachainRelay "github.com/snowfork/snowbridge/relayer/relays/parachain"
)

type Relay struct{}

func (re *Relay) Run() error {
	config, err := LoadConfig()
	if err != nil {
		return err
	}

	var pool worker.WorkerPool

	if config.Workers.EthRelayer.Enabled {
		ethereumRelayFactory := func() (worker.Worker, *worker.WorkerConfig, error) {
			return ethereumRelay.NewWorker(
				config.Global.DataDir,
				&config.Eth,
				&config.Parachain,
				logrus.WithField("worker", ethereumRelay.Name),
			), &config.Workers.EthRelayer, nil
		}
		pool = append(pool, ethereumRelayFactory)
	}

	if config.Workers.BeefyRelayer.Enabled {
		beefyRelayFactory := func() (worker.Worker, *worker.WorkerConfig, error) {
			beefyRelayer, err := beefyRelay.NewWorker(
				&config.Relaychain,
				&config.Eth,
				logrus.WithField("worker", beefyRelay.Name),
			)
			if err != nil {
				return nil, nil, err
			}
			return beefyRelayer, &config.Workers.BeefyRelayer, nil
		}
		pool = append(pool, beefyRelayFactory)
	}

	if config.Workers.ParachainCommitmentRelayer.Enabled {
		parachainRelayFactory := func() (worker.Worker, *worker.WorkerConfig, error) {
			relay, err := parachainRelay.NewWorker(
				&config.Parachain,
				&config.Relaychain,
				&config.Eth,
				logrus.WithField("worker", parachainRelay.Name),
			)
			if err != nil {
				return nil, nil, err
			}
			return relay, &config.Workers.ParachainCommitmentRelayer, nil
		}
		pool = append(pool, parachainRelayFactory)
	}

	return pool.Run()
}
