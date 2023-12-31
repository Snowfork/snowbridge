package execution

import (
	"github.com/snowfork/snowbridge/relayer/config"
	BeaconConfigs "github.com/snowfork/snowbridge/relayer/relays/beacon/config"
)

type Config struct {
	Source SourceConfig `mapstructure:"source"`
	Sink   SinkConfig   `mapstructure:"sink"`
}

type SourceConfig struct {
	Ethereum  config.EthereumConfig      `mapstructure:"ethereum"`
	Contracts ContractsConfig            `mapstructure:"contracts"`
	ChannelID ChannelID                  `mapstructure:"channel-id"`
	Beacon    BeaconConfigs.BeaconConfig `mapstructure:"beacon"`
}

type ContractsConfig struct {
	Gateway string `mapstructure:"Gateway"`
}

type SinkConfig struct {
	Parachain config.ParachainConfig `mapstructure:"parachain"`
}

type ChannelID [32]byte

func (c Config) GetSpecSettings() BeaconConfigs.SpecSettings {
	if c.Source.Beacon.ActiveSpec == "minimal" {
		return c.Source.Beacon.Spec.Minimal
	}

	return c.Source.Beacon.Spec.Mainnet
}

type ActiveSpec string

func (c Config) GetActiveSpec() BeaconConfigs.ActiveSpec {
	switch c.Source.Beacon.ActiveSpec {
	case "mainnet":
		return BeaconConfigs.Mainnet
	case "minimal":
		return BeaconConfigs.Minimal
	default:
		return BeaconConfigs.Mainnet
	}
}
