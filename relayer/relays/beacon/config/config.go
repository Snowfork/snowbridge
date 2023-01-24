package config

import (
	"github.com/snowfork/snowbridge/relayer/config"
)

type Config struct {
	Source SourceConfig `mapstructure:"source"`
	Sink   SinkConfig   `mapstructure:"sink"`
}

type SpecSettings struct {
	SlotsInEpoch                 uint64 `mapstructure:"slotsInEpoch"`
	EpochsPerSyncCommitteePeriod uint64 `mapstructure:"epochsPerSyncCommitteePeriod"`
}

type Spec struct {
	Minimal SpecSettings `mapstructure:"minimal"`
	Mainnet SpecSettings `mapstructure:"mainnet"`
}

type SourceConfig struct {
	Beacon BeaconConfig `mapstructure:"beacon"`
}

type BeaconConfig struct {
	Network    string `mapstructure:"network"`
	Endpoint   string `mapstructure:"endpoint"`
	Spec       Spec   `mapstructure:"spec"`
	ActiveSpec string `mapstructure:"activeSpec"`
}

type SinkConfig struct {
	Parachain config.ParachainConfig `mapstructure:"parachain"`
}

func (c Config) GetSpecSettings() SpecSettings {
	if c.Source.Beacon.ActiveSpec == "minimal" {
		return c.Source.Beacon.Spec.Minimal
	}

	return c.Source.Beacon.Spec.Mainnet
}

type Network string

const (
	Mainnet Network = "mainnet"
	Goerli  Network = "goerli"
	Local   Network = "local"
)

func (c Config) GetNetwork() Network {
	switch c.Source.Beacon.Network {
	case string(Mainnet):
		return Mainnet
	case string(Goerli):
		return Goerli
	case string(Local):
		return Local
	default:
		return Mainnet
	}
}

func (n Network) IsGoerli() bool {
	return n == Goerli
}

func (n Network) IsMainnet() bool {
	return n == Mainnet
}

func (n Network) IsLocal() bool {
	return n == Local
}
