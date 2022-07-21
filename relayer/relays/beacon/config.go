package beacon

import "github.com/snowfork/snowbridge/relayer/config"

type Config struct {
	Source SourceConfig `mapstructure:"source"`
	Sink   SinkConfig   `mapstructure:"sink"`
}

type SourceConfig struct {
	Beacon    BeaconConfig          `mapstructure:"beacon"`
	Ethereum  config.EthereumConfig `mapstructure:"ethereum"`
	Contracts ContractsConfig       `mapstructure:"contracts"`
}

type ContractsConfig struct {
	BasicOutboundChannel        string `mapstructure:"BasicOutboundChannel"`
	IncentivizedOutboundChannel string `mapstructure:"IncentivizedOutboundChannel"`
}

type BeaconConfig struct {
	Endpoint                string `mapstructure:"endpoint"`
	FinalizedUpdateEndpoint string `mapstructure:"finalizedUpdateEndpoint"`
}

type SinkConfig struct {
	Parachain config.ParachainConfig `mapstructure:"parachain"`
}
