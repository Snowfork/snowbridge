package reward

import (
	"fmt"

	"github.com/snowfork/snowbridge/relayer/config"
	beaconconf "github.com/snowfork/snowbridge/relayer/relays/beacon/config"
)

type Config struct {
	Source        SourceConfig `mapstructure:"source"`
	Sink          SinkConfig   `mapstructure:"sink"`
	RewardAddress string       `mapstructure:"reward-address"`
}

type SourceConfig struct {
	Ethereum  config.EthereumConfig   `mapstructure:"ethereum"`
	Contracts ContractsConfig         `mapstructure:"contracts"`
	Beacon    beaconconf.BeaconConfig `mapstructure:"beacon"`
}

type ContractsConfig struct {
	Gateway string `mapstructure:"Gateway"`
}

type SinkConfig struct {
	Parachain  beaconconf.ParachainConfig `mapstructure:"parachain"`
	SS58Prefix uint8                      `mapstructure:"ss58Prefix"`
}

func (c Config) Validate() error {
	err := c.Source.Beacon.Validate()
	if err != nil {
		return fmt.Errorf("beacon config validation: %w", err)
	}
	err = c.Sink.Parachain.Validate()
	if err != nil {
		return fmt.Errorf("parachain config validation: %w", err)
	}
	if c.Source.Contracts.Gateway == "" {
		return fmt.Errorf("source setting [gateway] is not set")
	}
	return nil
}
