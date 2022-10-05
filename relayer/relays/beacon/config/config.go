package config

import (
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
	Beacon    BeaconConfig          `mapstructure:"beacon"`
	Ethereum  config.EthereumConfig `mapstructure:"ethereum"`
	Contracts ContractsConfig       `mapstructure:"contracts"`
	Addresses []string              `mapstructure:"addresses"`
}

func (c *SourceConfig) GetAddresses() ([]common.Address, error) {
	var addresses []common.Address

	for _, address := range c.Addresses {
		// trimmedAddress := strings.TrimPrefix(address, "0x")
		// addressBytes, err := hex.DecodeString(trimmedAddress)

		// if err != nil {
		// 	return nil, fmt.Errorf("decode address: %w", err)
		// } else if len(addressBytes) != 20 {
		// 	// The conversion below will panic if decodedAddress has
		// 	// fewer than 20 bytes: we expect exactly 20 bytes.
		// 	return nil, fmt.Errorf("address was not 20 bytes long: %v", addressBytes)
		// }

		// decodedAddress := *(*[20]byte)(addressBytes)
		// addresses[common.HexToAddress(address)] = true
		addresses = append(addresses, common.HexToAddress(address))
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
