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
	PollSkipBlockCount  uint64                `mapstructure:"poll-skip-block-count"`
	PollIntervalSeconds uint64                `mapstructure:"poll-interval-seconds"`
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
