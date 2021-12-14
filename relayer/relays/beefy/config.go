package beefy

import (
	"github.com/snowfork/snowbridge/relayer/config"
)

type Config struct {
	Source SourceConfig `mapstructure:"source"`
	Sink   SinkConfig   `mapstructure:"sink"`
}

type SourceConfig struct {
	Polkadot config.PolkadotConfig `mapstructure:"polkadot"`
	// Block number when Beefy was activated
	BeefyActivationBlock uint64 `mapstructure:"beefy-activation-block"`
	// Number of blocks to skip between reading justifications
	BeefySkipPeriod uint64 `mapstructure:"beefy-skip-period"`
}

type SinkConfig struct {
	Ethereum              config.EthereumConfig `mapstructure:"ethereum"`
	DescendantsUntilFinal uint64                `mapstructure:"descendants-until-final"`
	Contracts             ContractsConfig       `mapstructure:"contracts"`
}

type ContractsConfig struct {
	BeefyLightClient string `mapstructure:"BeefyLightClient"`
}
