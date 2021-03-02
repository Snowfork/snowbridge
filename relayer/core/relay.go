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

	"github.com/ethereum/go-ethereum/common"
	"github.com/snowfork/polkadot-ethereum/relayer/chain"
	"github.com/snowfork/polkadot-ethereum/relayer/chain/ethereum"
	"github.com/snowfork/polkadot-ethereum/relayer/chain/substrate"
	"github.com/spf13/viper"
	"golang.org/x/sync/errgroup"

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
	config, err := LoadConfig()
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
	defer re.ethChain.Stop()

	err = re.subChain.Start(ctx, eg, ethInit, subInit)
	if err != nil {
		log.WithFields(log.Fields{
			"chain": re.subChain.Name(),
			"error": err,
		}).Error("Failed to start chain")
		return
	}
	log.WithField("name", re.subChain.Name()).Info("Started chain")
	defer re.subChain.Stop()

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
		re.subChain.Stop()
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

	accountWhitelist := &config.Eth.Channels.Basic.AccountWhitelist
	accountWhitelistMap := make(map[common.Address]bool)

	for i := 0; i < len(*accountWhitelist); i++ {
		account := common.HexToAddress((*accountWhitelist)[i])
		accountWhitelistMap[account] = true
	}

	config.Eth.Channels.Basic.AccountWhitelistMap = accountWhitelistMap

	return &config, nil
}
