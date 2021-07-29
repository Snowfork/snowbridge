package core

import (
	"github.com/sirupsen/logrus"

	worker "github.com/snowfork/snowbridge/relayer/relays"
	parachainRelay "github.com/snowfork/snowbridge/relayer/relays/parachain"
)

type Relay struct{}

func (re *Relay) Run() error {
	config, err := LoadConfig()
	if err != nil {
		return err
	}

	var pool worker.WorkerPool

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
