package beefy

import (
	"fmt"

	"github.com/snowfork/snowbridge/relayer/config"
)

type Config struct {
	Source       SourceConfig       `mapstructure:"source"`
	Sink         SinkConfig         `mapstructure:"sink"`
	OnDemandSync OnDemandSyncConfig `mapstructure:"on-demand-sync"`
}

type SourceConfig struct {
	Polkadot  config.PolkadotConfig  `mapstructure:"polkadot"`
	BridgeHub config.ParachainConfig `mapstructure:"bridge-hub"`
}

type SinkConfig struct {
	Ethereum              config.EthereumConfig `mapstructure:"ethereum"`
	DescendantsUntilFinal uint64                `mapstructure:"descendants-until-final"`
	Contracts             ContractsConfig       `mapstructure:"contracts"`
}

type ContractsConfig struct {
	BeefyClient string `mapstructure:"BeefyClient"`
	Gateway     string `mapstructure:"Gateway"`
}

type OnDemandSyncConfig struct {
	AssetHubChannelID string `mapstructure:"asset-hub-channel-id"`
	MaxTokens         uint64 `mapstructure:"max-tokens"`
	RefillAmount      uint64 `mapstructure:"refill-amount"`
	RefillPeriod      uint64 `mapstructure:"refill-period"`
}

func (c Config) Validate() error {
	err := c.Source.Polkadot.Validate()
	if err != nil {
		return fmt.Errorf("source polkadot config: %w", err)
	}
	err = c.Sink.Ethereum.Validate()
	if err != nil {
		return fmt.Errorf("sink ethereum config: %w", err)
	}
	if c.Sink.DescendantsUntilFinal == 0 {
		return fmt.Errorf("sink ethereum setting [descendants-until-final] is not set")
	}
	if c.Sink.Contracts.BeefyClient == "" {
		return fmt.Errorf("sink contracts setting [BeefyClient] is not set")
	}
	if c.Sink.Contracts.Gateway == "" {
		return fmt.Errorf("sink contracts setting [Gateway] is not set")
	}
	return nil
}
