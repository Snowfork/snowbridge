// Copyright 2021 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package ethereum

import (
	"context"

	"golang.org/x/sync/errgroup"

	"github.com/snowfork/go-substrate-rpc-client/v3/types"
	"github.com/snowfork/snowbridge/relayer/chain/ethereum"
	"github.com/snowfork/snowbridge/relayer/chain/parachain"
	"github.com/snowfork/snowbridge/relayer/crypto/sr25519"

	log "github.com/sirupsen/logrus"
)

type Config struct {
	DataDir  string         `mapstructure:"data-dir"`
	Polkadot PolkadotConfig `mapstructure:"polkadot"`
	Ethereum EthereumConfig `mapstructure:"ethereum"`
}

type PolkadotConfig struct {
	Endpoint string `mapstructure:"endpoint"`
}

type EthereumConfig struct {
	Endpoint              string         `mapstructure:"endpoint"`
	DescendantsUntilFinal uint64         `mapstructure:"descendants-until-final"`
	Channels              ChannelsConfig `mapstructure:"channels"`
}

type ChannelsConfig struct {
	Basic        ChannelConfig `mapstructure:"basic"`
	Incentivized ChannelConfig `mapstructure:"incentivized"`
}

type ChannelConfig struct {
	Inbound  string `mapstructure:"inbound"`
	Outbound string `mapstructure:"outbound"`
}

type Relay struct {
	config     *Config
	keypair    *sr25519.Keypair
	ethconfig  *ethereum.Config
	ethconn    *ethereum.Connection
	paraconfig *parachain.Config
	paraconn   *parachain.Connection
}

func NewRelay(
	config *Config,
	keypair *sr25519.Keypair,
) *Relay {
	return &Relay{
		config:  config,
		keypair: keypair,
	}
}

func (r *Relay) Start(ctx context.Context, eg *errgroup.Group) error {
	err := r.connect(ctx)
	if err != nil {
		return err
	}

	// Clean up after ourselves
	eg.Go(func() error {
		<-ctx.Done()
		r.disconnect()
		return nil
	})

	// channel for payloads from ethereum
	payloads := make(chan ParachainPayload, 1)

	listener := NewEthereumListener(
		r.config,
		r.ethconn,
		payloads,
	)
	writer := NewParachainWriter(
		r.paraconn,
		payloads,
	)

	finalizedBlockNumber, err := r.queryFinalizedBlockNumber()
	if err != nil {
		return err
	}
	log.WithField("blockNumber", finalizedBlockNumber).Debug("Retrieved finalized block number from parachain")

	err = listener.Start(ctx, eg, finalizedBlockNumber+1, uint64(r.ethconfig.DescendantsUntilFinal))
	if err != nil {
		return err
	}

	err = writer.Start(ctx, eg)
	if err != nil {
		return err
	}

	return nil
}

func (r *Relay) queryFinalizedBlockNumber() (uint64, error) {
	storageKey, err := types.CreateStorageKey(r.paraconn.Metadata(), "EthereumLightClient", "FinalizedBlock", nil, nil)
	if err != nil {
		return 0, err
	}

	var finalizedHeader ethereum.HeaderID
	_, err = r.paraconn.API().RPC.State.GetStorageLatest(storageKey, &finalizedHeader)
	if err != nil {
		return 0, err
	}

	return uint64(finalizedHeader.Number), nil
}

func (r *Relay) connect(ctx context.Context) error {
	kpForPara, err := sr25519.NewKeypairFromSeed(r.paraconfig.PrivateKey, 42)
	if err != nil {
		return err
	}

	r.ethconn = ethereum.NewConnection(r.ethconfig.Endpoint, nil)
	r.paraconn = parachain.NewConnection(r.paraconfig.Endpoint, kpForPara.AsKeyringPair())

	err = r.ethconn.Connect(ctx)
	if err != nil {
		return err
	}

	err = r.paraconn.Connect(ctx)
	if err != nil {
		return err
	}

	return nil
}

func (r *Relay) disconnect() {
	if r.ethconn != nil {
		r.ethconn.Close()
		r.ethconn = nil
	}

	if r.paraconn != nil {
		r.paraconn.Close()
		r.paraconn = nil
	}
}
