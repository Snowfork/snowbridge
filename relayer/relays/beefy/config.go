package beefy

import (
	"github.com/snowfork/snowbridge/relayer/config"
)

type Config struct {
	Source SourceConfig `mapstructure:"source"`
	Sink   SinkConfig   `mapstructure:"sink"`
}

type SourceConfig struct {
	Polkadot            config.PolkadotConfig `mapstructure:"polkadot"`
	SyncBlockNumberJump uint64                `mapstructure:"sync-block-number-jump"`
}

type SinkConfig struct {
	Ethereum              config.EthereumConfig `mapstructure:"ethereum"`
	StartBlock            uint64                `mapstructure:"start-block"`
	DescendantsUntilFinal uint64                `mapstructure:"descendants-until-final"`
	Contracts             ContractsConfig       `mapstructure:"contracts"`
}

type ContractsConfig struct {
	BeefyLightClient string `mapstructure:"BeefyLightClient"`
}
