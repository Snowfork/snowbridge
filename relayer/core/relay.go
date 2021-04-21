// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package core

import (
	"context"
	"errors"
	"fmt"
	"os"
	"os/signal"
	"strings"
	"syscall"
	"time"

	log "github.com/sirupsen/logrus"
	"github.com/spf13/viper"
	"golang.org/x/sync/errgroup"

	"github.com/snowfork/polkadot-ethereum/relayer/chain/ethereum"
	"github.com/snowfork/polkadot-ethereum/relayer/chain/parachain"
	"github.com/snowfork/polkadot-ethereum/relayer/chain/relaychain"
	"github.com/snowfork/polkadot-ethereum/relayer/workers/beefyrelayer"
	"github.com/snowfork/polkadot-ethereum/relayer/workers/beefyrelayer/store"
	"github.com/snowfork/polkadot-ethereum/relayer/workers/ethrelayer"
	"github.com/snowfork/polkadot-ethereum/relayer/workers/parachaincommitmentrelayer"
)

type Relay struct {
	parachainCommitmentRelayer *parachaincommitmentrelayer.Worker
	beefyRelayer               *beefyrelayer.Worker
	ethRelayer                 *ethrelayer.Worker
}

type WorkerConfig struct {
	ParachainCommitmentRelayer bool `mapstructure:"parachaincommitmentrrelayer"`
	BeefyRelayer               bool `mapstructure:"beefyrelayer"`
	EthRelayer                 bool `mapstructure:"ethrelayer"`
}

type Config struct {
	Eth                  ethereum.Config   `mapstructure:"ethereum"`
	Parachain            parachain.Config  `mapstructure:"parachain"`
	Relaychain           relaychain.Config `mapstructure:"relaychain"`
	BeefyRelayerDatabase store.Config      `mapstructure:"database"`
	Workers              WorkerConfig      `mapstructure:"workers"`
}

func NewRelay() (*Relay, error) {
	config, err := LoadConfig()
	if err != nil {
		return nil, err
	}

	parachainCommitmentRelayer := &parachaincommitmentrelayer.Worker{}

	if config.Workers.ParachainCommitmentRelayer == true {
		parachainCommitmentRelayer, err = parachaincommitmentrelayer.NewWorker(&config.Parachain, &config.Relaychain, &config.Eth)
		if err != nil {
			return nil, err
		}
	}

	beefyRelayer := &beefyrelayer.Worker{}

	if config.Workers.BeefyRelayer == true {
		beefyRelayer, err = beefyrelayer.NewWorker(&config.Relaychain, &config.Eth, &config.BeefyRelayerDatabase)
		if err != nil {
			return nil, err
		}
	}

	ethRelayer := &ethrelayer.Worker{}

	if config.Workers.EthRelayer == true {
		ethRelayer, err = ethrelayer.NewWorker(&config.Eth, &config.Parachain)
		if err != nil {
			return nil, err
		}
	}

	return &Relay{
		parachainCommitmentRelayer: parachainCommitmentRelayer,
		beefyRelayer:               beefyRelayer,
		ethRelayer:                 ethRelayer,
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

	if re.ethRelayer != nil {
		err := re.ethRelayer.Start(ctx, eg)
		if err != nil {
			log.WithFields(log.Fields{
				"worker": re.ethRelayer.Name(),
				"error":  err,
			}).Error("Failed to start worker")
			return
		}
		log.WithField("name", re.ethRelayer.Name()).Info("Started worker")
		defer re.ethRelayer.Stop()
	}

	if re.beefyRelayer != nil {
		err := re.beefyRelayer.Start(ctx, eg)
		if err != nil {
			log.WithFields(log.Fields{
				"worker": re.beefyRelayer.Name(),
				"error":  err,
			}).Error("Failed to start worker")
			return
		}
		log.WithField("name", re.beefyRelayer.Name()).Info("Started worker")
		defer re.beefyRelayer.Stop()
	}

	if re.parachainCommitmentRelayer != nil {
		err := re.parachainCommitmentRelayer.Start(ctx, eg)
		if err != nil {
			log.WithFields(log.Fields{
				"worker": re.parachainCommitmentRelayer.Name(),
				"error":  err,
			}).Error("Failed to start worker")
			return
		}
		log.WithField("name", re.parachainCommitmentRelayer.Name()).Info("Started worker")
		defer re.parachainCommitmentRelayer.Stop()
	}

	notifyWaitDone := make(chan struct{})

	go func() {
		err := eg.Wait()
		if err != nil && !errors.Is(err, context.Canceled) {
			log.WithField("error", err).Error("Encountered an unrecoverable failure")
		}
		close(notifyWaitDone)
	}()

	// Wait until a fatal error or signal is raised
	select {
	case <-notifyWaitDone:
		break
	case <-ctx.Done():
		// Goroutines are either shutting down or deadlocked.
		// Give them a few seconds...
		select {
		case <-time.After(3 * time.Second):
			break
		case _, stillWaiting := <-notifyWaitDone:
			if !stillWaiting {
				// All goroutines have ended
				return
			}
		}

		log.WithError(ctx.Err()).Error("Goroutines appear deadlocked. Killing process")
		re.ethRelayer.Stop()
		re.parachainCommitmentRelayer.Stop()
		re.beefyRelayer.Stop()

		relayProc, err := os.FindProcess(os.Getpid())
		if err != nil {
			log.WithError(err).Error("Failed to kill this process")
		}
		relayProc.Kill()
	}
}

func LoadConfig() (*Config, error) {
	var config Config
	err := viper.Unmarshal(&config)
	if err != nil {
		return nil, err
	}

	// Load secrets from environment variables
	var value string
	var ok bool

	// Ethereum configuration
	value, ok = os.LookupEnv("ARTEMIS_ETHEREUM_KEY")
	if !ok {
		return nil, fmt.Errorf("environment variable not set: ARTEMIS_ETHEREUM_KEY")
	}
	config.Eth.PrivateKey = strings.TrimPrefix(value, "0x")

	// Parachain configuration
	value, ok = os.LookupEnv("ARTEMIS_PARACHAIN_KEY")
	if !ok {
		return nil, fmt.Errorf("environment variable not set: ARTEMIS_PARACHAIN_KEY")
	}
	config.Parachain.PrivateKey = value

	// Relaychain configuration
	value, ok = os.LookupEnv("ARTEMIS_RELAYCHAIN_KEY")
	if !ok {
		return nil, fmt.Errorf("environment variable not set: ARTEMIS_RELAYCHAIN_KEY")
	}
	config.Relaychain.PrivateKey = value

	return &config, nil
}
