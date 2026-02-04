package execution

import (
	"fmt"

	"github.com/snowfork/snowbridge/relayer/config"
	beaconconf "github.com/snowfork/snowbridge/relayer/relays/beacon/config"
)

type Config struct {
	Source              SourceConfig       `mapstructure:"source"`
	Sink                SinkConfig         `mapstructure:"sink"`
	InstantVerification bool               `mapstructure:"instantVerification"`
	OFAC                config.OFACConfig  `mapstructure:"ofac"`
	GasEstimation       GasEstimatorConfig `mapstructure:"gasEstimation"`
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
	if c.Source.Contracts.Gateway == "" {
		return fmt.Errorf("source setting [gateway] is not set")
	}
	err = c.OFAC.Validate()
	if err != nil {
		return fmt.Errorf("ofac config: %w", err)
	}
	err = c.GasEstimation.Validate()
	if err != nil {
		return fmt.Errorf("gas estimation config: %w", err)
	}
	return nil
}
