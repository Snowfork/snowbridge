package beacon

import "github.com/snowfork/snowbridge/relayer/config"

type Config struct {
	Source SourceConfig `mapstructure:"source"`
	Sink   SinkConfig   `mapstructure:"sink"`
}

type SourceConfig struct {
	Beacon                BeaconConfig          `mapstructure:"beacon"`
	Ethereum              config.EthereumConfig `mapstructure:"ethereum"`
	DataDir               string                `mapstructure:"data-dir"`
	DescendantsUntilFinal uint64                `mapstructure:"descendants-until-final"`
	Contracts             ContractsConfig       `mapstructure:"contracts"`
}

type ContractsConfig struct {
	BasicOutboundChannel        string `mapstructure:"BasicOutboundChannel"`
}

type BeaconConfig struct {
	Endpoint                string `mapstructure:"endpoint"`
	FinalizedUpdateEndpoint string `mapstructure:"finalizedUpdateEndpoint"`
}

type SinkConfig struct {
	Parachain config.ParachainConfig `mapstructure:"parachain"`
}
