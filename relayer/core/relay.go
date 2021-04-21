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

	"github.com/snowfork/polkadot-ethereum/relayer/chain"
	"github.com/snowfork/polkadot-ethereum/relayer/chain/ethereum"
	"github.com/snowfork/polkadot-ethereum/relayer/chain/parachain"
	"github.com/snowfork/polkadot-ethereum/relayer/chain/relaychain"
	"github.com/snowfork/polkadot-ethereum/relayer/store"
	"github.com/snowfork/polkadot-ethereum/relayer/workers/beefyrelayer"
	"github.com/snowfork/polkadot-ethereum/relayer/workers/parachaincommitmentrelayer"
)

type Relay struct {
	ethChain                   chain.Chain
	paraChain                  chain.Chain
	database                   *store.Database
	direction                  Direction
	parachainCommitmentRelayer *parachaincommitmentrelayer.Worker
	beefyRelayer               *beefyrelayer.Worker
}

type Direction int

const (
	Bidirectional Direction = iota
	EthToSub
	SubToEth
)

type RelayConfig struct {
	Direction   Direction `mapstructure:"direction"`
	HeadersOnly bool      `mapstructure:"headers-only"`
}

type WorkerConfig struct {
	ParachainCommitmentRelayer bool `mapstructure:"parachaincommitmentrrelayer"`
	BeefyRelayer               bool `mapstructure:"beefyrelayer"`
}

type Config struct {
	Relay      RelayConfig       `mapstructure:"relay"`
	Eth        ethereum.Config   `mapstructure:"ethereum"`
	Parachain  parachain.Config  `mapstructure:"parachain"`
	Relaychain relaychain.Config `mapstructure:"relaychain"`
	Database   store.Config      `mapstructure:"database"`
	Workers    WorkerConfig      `mapstructure:"workers"`
}

func NewRelay() (*Relay, error) {
	config, err := LoadConfig()
	if err != nil {
		return nil, err
	}

	ethChain, err := ethereum.NewChain(&config.Eth)
	if err != nil {
		return nil, err
	}

	paraChain, err := parachain.NewChain(&config.Parachain)
	if err != nil {
		return nil, err
	}

	direction := config.Relay.Direction
	headersOnly := config.Relay.HeadersOnly
	if direction == Bidirectional || direction == EthToSub {
		// channel for messages from ethereum
		var ethMessages chan []chain.Message
		if !headersOnly {
			ethMessages = make(chan []chain.Message, 1)
		}
		// channel for headers from ethereum (it's a blocking channel so that we
		// can guarantee that a header is forwarded before we send dependent messages)
		ethHeaders := make(chan chain.Header)

		err = ethChain.SetSender(ethMessages, ethHeaders)
		if err != nil {
			return nil, err
		}

		err = paraChain.SetReceiver(ethMessages, ethHeaders)
		if err != nil {
			return nil, err
		}
	}

	if direction == Bidirectional || direction == SubToEth {
		// channel for messages from substrate
		var subMessages chan []chain.Message
		if !headersOnly {
			subMessages = make(chan []chain.Message, 1)
		}

		err = ethChain.SetReceiver(subMessages, nil)
		if err != nil {
			return nil, err
		}

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
		beefyRelayer, err = beefyrelayer.NewWorker(&config.Relaychain, &config.Eth, &config.Database)
		if err != nil {
			return nil, err
		}
	}

	return &Relay{
		ethChain:                   ethChain,
		paraChain:                  paraChain,
		direction:                  direction,
		parachainCommitmentRelayer: parachainCommitmentRelayer,
		beefyRelayer:               beefyRelayer,
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

	// Short-lived channels that communicate initialization parameters
	// between the two chains. The chains close them after startup.
	subInit := make(chan chain.Init)
	ethSubInit := make(chan chain.Init)

	err := re.ethChain.Start(ctx, eg, subInit, ethSubInit)
	if err != nil {
		log.WithFields(log.Fields{
			"chain": re.ethChain.Name(),
			"error": err,
		}).Error("Failed to start chain")
		return
	}
	log.WithField("name", re.ethChain.Name()).Info("Started chain")
	defer re.ethChain.Stop()

	err = re.paraChain.Start(ctx, eg, ethSubInit, subInit)
	if err != nil {
		log.WithFields(log.Fields{
			"chain": re.paraChain.Name(),
			"error": err,
		}).Error("Failed to start chain")
		return
	}
	log.WithField("name", re.paraChain.Name()).Info("Started chain")
	defer re.paraChain.Stop()

	if re.beefyRelayer != nil {
		err = re.beefyRelayer.Start(ctx, eg)
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
		err = re.parachainCommitmentRelayer.Start(ctx, eg)
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
		re.ethChain.Stop()
		re.paraChain.Stop()
		re.parachainCommitmentRelayer.Stop()
		re.beefyRelayer.Stop()
		re.database.Stop()

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

	var direction = config.Relay.Direction
	if direction != Bidirectional &&
		direction != EthToSub &&
		direction != SubToEth {
		return nil, fmt.Errorf("'direction' has invalid value %d", direction)
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
