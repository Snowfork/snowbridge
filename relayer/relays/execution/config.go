package execution

import (
	"fmt"
	"github.com/snowfork/snowbridge/relayer/config"
	beaconconf "github.com/snowfork/snowbridge/relayer/relays/beacon/config"
)

type Config struct {
	Source              SourceConfig `mapstructure:"source"`
	Sink                SinkConfig   `mapstructure:"sink"`
	InstantVerification bool         `mapstructure:"instantVerification"`
}

type SourceConfig struct {
	Ethereum  config.EthereumConfig   `mapstructure:"ethereum"`
	Contracts ContractsConfig         `mapstructure:"contracts"`
	ChannelID ChannelID               `mapstructure:"channel-id"`
	Beacon    beaconconf.BeaconConfig `mapstructure:"beacon"`
}

type ContractsConfig struct {
	Gateway string `mapstructure:"Gateway"`
}

type SinkConfig struct {
	Parachain config.ParachainConfig `mapstructure:"parachain"`
}

type ChannelID [32]byte

func (c Config) Validate() error {
	err := c.Source.Beacon.Validate()
	if err != nil {
		return fmt.Errorf("beacon config validation: %w", err)
	}
	err = c.Sink.Parachain.Validate()
	if err != nil {
		return fmt.Errorf("parachain config validation: %w", err)
	}
	if c.Source.ChannelID == [32]byte{} {
		return fmt.Errorf("channel ID is empty")
	}
	if c.Source.Contracts.Gateway == "" {
		return fmt.Errorf("gateway contract is empty")
	}
	return nil
}
