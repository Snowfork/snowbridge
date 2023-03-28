package execution

import (
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/config"
)

type Config struct {
	Source SourceConfig `mapstructure:"source"`
	Sink   SinkConfig   `mapstructure:"sink"`
}

type SourceConfig struct {
	Ethereum  config.EthereumConfig `mapstructure:"ethereum"`
	Contracts ContractsConfig       `mapstructure:"contracts"`
	LaneID    MultiLocation         `mapstructure:"lane-id"`
}

type Address []byte

type ContractsConfig struct {
	OutboundChannel string `mapstructure:"OutboundChannel"`
}

type SinkConfig struct {
	Parachain config.ParachainConfig `mapstructure:"parachain"`
}

type MultiLocation types.MultiLocationV3

func (m *MultiLocation) UnmarshalJSON(b []byte) error {
	return types.DecodeFromHexString(string(b), m)
}
