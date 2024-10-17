package parachain

import (
	"errors"
	"fmt"

	"github.com/snowfork/snowbridge/relayer/config"
)

type Config struct {
	Source   SourceConfig      `mapstructure:"source"`
	Sink     SinkConfig        `mapstructure:"sink"`
	Schedule ScheduleConfig    `mapstructure:"schedule"`
	OFAC     config.OFACConfig `mapstructure:"ofac"`
}

type SourceConfig struct {
	Polkadot  config.PolkadotConfig  `mapstructure:"polkadot"`
	Parachain config.ParachainConfig `mapstructure:"parachain"`
	Ethereum  config.EthereumConfig  `mapstructure:"ethereum"`
	Contracts SourceContractsConfig  `mapstructure:"contracts"`
	ChannelID ChannelID              `mapstructure:"channel-id"`
}

type SourceContractsConfig struct {
	BeefyClient string `mapstructure:"BeefyClient"`
	Gateway     string `mapstructure:"Gateway"`
}

type SinkConfig struct {
	Ethereum  config.EthereumConfig `mapstructure:"ethereum"`
	Contracts SinkContractsConfig   `mapstructure:"contracts"`
}

type SinkContractsConfig struct {
	Gateway string `mapstructure:"Gateway"`
}

type ScheduleConfig struct {
	// ID of current relayer, starting from 0
	ID uint64 `mapstructure:"id"`
	// Number of total count of all relayers
	TotalRelayerCount uint64 `mapstructure:"totalRelayerCount"`
	// Sleep interval(in seconds) to check if message(nonce) has already been relayed
	SleepInterval uint64 `mapstructure:"sleepInterval"`
}

func (r ScheduleConfig) Validate() error {
	if r.TotalRelayerCount < 1 {
		return errors.New("Number of relayer is not set")
	}
	if r.ID >= r.TotalRelayerCount {
		return errors.New("ID of the Number of relayer is not set")
	}
	return nil
}

type ChannelID [32]byte

func (c Config) Validate() error {
	// Source
	err := c.Source.Polkadot.Validate()
	if err != nil {
		return fmt.Errorf("source polkadot config: %w", err)
	}
	err = c.Source.Parachain.Validate()
	if err != nil {
		return fmt.Errorf("source parachain config: %w", err)
	}
	err = c.Source.Ethereum.Validate()
	if err != nil {
		return fmt.Errorf("source ethereum config: %w", err)
	}
	if c.Source.Contracts.BeefyClient == "" {
		return fmt.Errorf("source contracts setting [BeefyClient] is not set")
	}
	if c.Source.Contracts.Gateway == "" {
		return fmt.Errorf("source contracts setting [Gateway] is not set")
	}
	if c.Source.ChannelID == [32]byte{} {
		return fmt.Errorf("source setting [channel-id] is not set")
	}

	// Sink
	err = c.Sink.Ethereum.Validate()
	if err != nil {
		return fmt.Errorf("sink ethereum config: %w", err)
	}
	if c.Sink.Contracts.Gateway == "" {
		return fmt.Errorf("sink contracts setting [Gateway] is not set")
	}

	// Relay
	err = c.Schedule.Validate()
	if err != nil {
		return fmt.Errorf("relay config: %w", err)
	}
	err = c.OFAC.Validate()
	if err != nil {
		return fmt.Errorf("ofac config: %w", err)
	}

	return nil
}
