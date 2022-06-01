package beacon

import "github.com/snowfork/snowbridge/relayer/config"

type Config struct {
	Source SourceConfig `mapstructure:"source"`
	Sink   SinkConfig   `mapstructure:"sink"`
}

type SourceConfig struct {
	Beacon BeaconConfig `mapstructure:"beacon"`
}

type BeaconConfig struct {
	Endpoint                string `mapstructure:"endpoint"`
	FinalizedUpdateEndpoint string `mapstructure:"finalizedUpdateEndpoint"`
}

type SinkConfig struct {
	Parachain config.ParachainConfig `mapstructure:"parachain"`
}
