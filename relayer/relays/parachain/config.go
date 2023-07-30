package parachain

import (
	"github.com/snowfork/snowbridge/relayer/config"
)

type Config struct {
	Source SourceConfig `mapstructure:"source"`
	Sink   SinkConfig   `mapstructure:"sink"`
}

// TODO: check whether ChannelID should be uint32 (as in the parachain) or big.Int (uint256, as in the Gateway contract)
type SourceConfig struct {
	Polkadot  config.PolkadotConfig  `mapstructure:"polkadot"`
	Parachain config.ParachainConfig `mapstructure:"parachain"`
	Ethereum  config.EthereumConfig  `mapstructure:"ethereum"`
	Contracts SourceContractsConfig  `mapstructure:"contracts"`
	ChannelID uint32                 `mapstructure:"channel-id"`
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
