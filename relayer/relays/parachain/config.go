package parachain

import (
	"github.com/snowfork/snowbridge/relayer/config"
)

type Config struct {
	Source SourceConfig `mapstructure:"source"`
	Sink   SinkConfig   `mapstructure:"sink"`
}

type SourceConfig struct {
	Polkadot  config.PolkadotConfig  `mapstructure:"polkadot"`
	Parachain config.ParachainConfig `mapstructure:"parachain"`
	Ethereum  config.EthereumConfig  `mapstructure:"ethereum"`
	Contracts SourceContractsConfig  `mapstructure:"contracts"`
	// Block number when Beefy was activated
	BeefyActivationBlock uint64 `mapstructure:"beefy-activation-block"`
}

type SourceContractsConfig struct {
	BeefyClient    string `mapstructure:"BeefyClient"`
	InboundChannel string `mapstructure:"InboundChannel"`
}

type SinkConfig struct {
	Ethereum  config.EthereumConfig `mapstructure:"ethereum"`
	Contracts SinkContractsConfig   `mapstructure:"contracts"`
}

type SinkContractsConfig struct {
	InboundChannel string `mapstructure:"InboundChannel"`
}
