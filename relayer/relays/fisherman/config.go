package fisherman

import (
	"fmt"

	"github.com/snowfork/snowbridge/relayer/config"
)

type Config struct {
	Source SourceConfig `mapstructure:"source"`
	Sink   SinkConfig   `mapstructure:"sink"`
}

type SourceConfig struct {
	Polkadot  config.PolkadotConfig `mapstructure:"polkadot"`
	Ethereum  config.EthereumConfig `mapstructure:"ethereum"`
	Contracts SourceContractsConfig `mapstructure:"contracts"`
}

type SourceContractsConfig struct {
	BeefyClient string `mapstructure:"BeefyClient"`
}

type SinkConfig struct {
	Polkadot config.PolkadotConfig `mapstructure:"polkadot"`
}

func (c Config) Validate() error {
	// Source
	err := c.Source.Polkadot.Validate()
	if err != nil {
		return fmt.Errorf("source polkadot config: %w", err)
	}
	err = c.Source.Ethereum.Validate()
	if err != nil {
		return fmt.Errorf("source ethereum config: %w", err)
	}
	if c.Source.Contracts.BeefyClient == "" {
		return fmt.Errorf("source contracts setting [BeefyClient] is not set")
	}

	// Sink
	err = c.Sink.Polkadot.Validate()
	if err != nil {
		return fmt.Errorf("sink polkadot config: %w", err)
	}

	return nil
}
