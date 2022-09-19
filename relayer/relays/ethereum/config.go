package ethereum

import (
	"github.com/snowfork/snowbridge/relayer/config"
)

type Config struct {
	Source SourceConfig `mapstructure:"source"`
	Sink   SinkConfig   `mapstructure:"sink"`
}

type SourceConfig struct {
	Ethereum              config.EthereumConfig `mapstructure:"ethereum"`
	DataDir               string                `mapstructure:"data-dir"`
	DescendantsUntilFinal uint64                `mapstructure:"descendants-until-final"`
	Contracts             ContractsConfig       `mapstructure:"contracts"`
}

type ContractsConfig struct {
	BasicOutboundChannel        string `mapstructure:"BasicOutboundChannel"`
	IncentivizedOutboundChannel string `mapstructure:"IncentivizedOutboundChannel"`
}

type SinkConfig struct {
	Parachain config.ParachainConfig `mapstructure:"parachain"`
}
