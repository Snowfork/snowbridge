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

type SourceConfig struct {
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

type SinkConfig struct {
	Parachain config.ParachainConfig `mapstructure:"parachain"`
}
