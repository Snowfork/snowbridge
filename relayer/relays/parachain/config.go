package parachain

import (
	"github.com/snowfork/snowbridge/relayer/config"
)

type Config struct {
	Source SourceConfig `mapstructure:"source"`
	Sink   SinkConfig   `mapstructure:"sink"`
}

// TODO: check whether LaneID should be uint32 (as in the parachain) or big.Int (uint256, as in the Gateway contract)
// TODO: rename LaneID to ChannelID
type SourceConfig struct {
	Polkadot  config.PolkadotConfig  `mapstructure:"polkadot"`
	Parachain config.ParachainConfig `mapstructure:"parachain"`
	Ethereum  config.EthereumConfig  `mapstructure:"ethereum"`
	Contracts SourceContractsConfig  `mapstructure:"contracts"`
	LaneID    uint32                 `mapstructure:"lane-id"`
}

type SourceContractsConfig struct {
	BeefyClient string `mapstructure:"BeefyClient"`
	Gateway     string `mapstructure:"Gateway"`
}

type SinkConfig struct {
	Ethereum  config.EthereumConfig `mapstructure:"ethereum"`
	Contracts SinkContractsConfig   `mapstructure:"contracts"`
}

type SinkContractsConfig struct {
	Gateway string `mapstructure:"Gateway"`
}
