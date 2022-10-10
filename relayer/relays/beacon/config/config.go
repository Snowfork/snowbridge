package config

import (
	"strings"

	"github.com/ethereum/go-ethereum/common"
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
	Beacon                BeaconConfig          `mapstructure:"beacon"`
	Ethereum              config.EthereumConfig `mapstructure:"ethereum"`
	Contracts             ContractsConfig       `mapstructure:"contracts"`
	BasicChannelAddresses []string              `mapstructure:"basicChannelAddresses"`
}

func (c *SourceConfig) GetBasicChannelAddresses() ([]common.Address, error) {
	var addresses []common.Address

	for _, address := range c.BasicChannelAddresses {
		trimmedAddress := strings.TrimPrefix(address, "0x")

		addresses = append(addresses, common.HexToAddress(trimmedAddress))
	}

	return addresses, nil
}

type ContractsConfig struct {
	BasicOutboundChannel        string `mapstructure:"BasicOutboundChannel"`
	IncentivizedOutboundChannel string `mapstructure:"IncentivizedOutboundChannel"`
}

type BeaconConfig struct {
	Endpoint                string `mapstructure:"endpoint"`
	FinalizedUpdateEndpoint string `mapstructure:"finalizedUpdateEndpoint"`
	Spec                    Spec   `mapstructure:"spec"`
	ActiveSpec              string `mapstructure:"activeSpec"`
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
