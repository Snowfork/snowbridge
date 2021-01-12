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

	"github.com/snowfork/polkadot-ethereum/relayer/chain"
	"github.com/snowfork/polkadot-ethereum/relayer/chain/ethereum"
	"github.com/snowfork/polkadot-ethereum/relayer/chain/substrate"
	"github.com/spf13/viper"
	"golang.org/x/sync/errgroup"

	"github.com/ethereum/go-ethereum/common"

	log "github.com/sirupsen/logrus"
)

type Relay struct {
	ethChain chain.Chain
	subChain chain.Chain
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

type Config struct {
	Relay RelayConfig      `mapstructure:"relay"`
	Eth   ethereum.Config  `mapstructure:"ethereum"`
	Sub   substrate.Config `mapstructure:"substrate"`
}

func NewRelay() (*Relay, error) {
	config, err := loadConfig()
	if err != nil {
		return nil, err
	}

	ethChain, err := ethereum.NewChain(&config.Eth)
	if err != nil {
		return nil, err
	}

	subChain, err := substrate.NewChain(&config.Sub)
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

		err := ethChain.SetSender(ethMessages, ethHeaders)
		if err != nil {
			return nil, err
		}
		err = subChain.SetReceiver(ethMessages, ethHeaders)
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

		err := subChain.SetSender(subMessages, nil)
		if err != nil {
			return nil, err
		}
		err = ethChain.SetReceiver(subMessages, nil)
		if err != nil {
			return nil, err
		}
	}

	return &Relay{
		ethChain: ethChain,
		subChain: subChain,
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
	ethInit := make(chan chain.Init, 1)
	subInit := make(chan chain.Init, 1)

	err := re.ethChain.Start(ctx, eg, subInit, ethInit)
	if err != nil {
		log.WithFields(log.Fields{
			"chain": re.ethChain.Name(),
			"error": err,
		}).Error("Failed to start chain")
		return
	}
	log.WithField("name", re.ethChain.Name()).Info("Started chain")

	err = re.subChain.Start(ctx, eg, ethInit, subInit)
	if err != nil {
		log.WithFields(log.Fields{
			"chain": re.subChain.Name(),
			"error": err,
		}).Error("Failed to start chain")
		return
	}
	log.WithField("name", re.subChain.Name()).Info("Started chain")

	// Wait until a fatal error or signal is raised
	if err := eg.Wait(); err != nil {
		if !errors.Is(err, context.Canceled) {
			log.WithField("error", err).Error("Encountered an unrecoverable failure")
		}
	}

	// Shutdown chains
	re.ethChain.Stop()
	re.subChain.Stop()
}

func loadConfig() (*Config, error) {
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

	if config.Relay.HeadersOnly {
		config.Eth.Apps = map[string]ethereum.ContractInfo{}
	}

	// Load secrets from environment variables
	var value string
	var ok bool

	value, ok = os.LookupEnv("ARTEMIS_ETHEREUM_KEY")
	if !ok {
		return nil, fmt.Errorf("environment variable not set: ARTEMIS_ETHEREUM_KEY")
	}
	config.Eth.PrivateKey = strings.TrimPrefix(value, "0x")

	value, ok = os.LookupEnv("ARTEMIS_SUBSTRATE_KEY")
	if !ok {
		return nil, fmt.Errorf("environment variable not set: ARTEMIS_SUBSTRATE_KEY")
	}
	config.Sub.PrivateKey = value

	// Copy over Ethereum application addresses to the Substrate config
	config.Sub.Targets = make(map[string][20]byte)
	for k, v := range config.Eth.Apps {
		config.Sub.Targets[k] = common.HexToAddress(v.Address)
	}

	return &config, nil
}
