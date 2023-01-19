package parachain

import (
	"encoding/hex"
	"fmt"
	"strings"

	"github.com/snowfork/snowbridge/relayer/config"
)

type Config struct {
	Source SourceConfig `mapstructure:"source"`
	Sink   SinkConfig   `mapstructure:"sink"`
}

type SourceConfig struct {
	Polkadot  config.PolkadotConfig  `mapstructure:"polkadot"`
	Parachain config.ParachainConfig `mapstructure:"parachain"`
	Ethereum  config.EthereumConfig  `mapstructure:"ethereum"`
	Contracts SourceContractsConfig  `mapstructure:"contracts"`
	// Block number when Beefy was activated
	BeefyActivationBlock uint64   `mapstructure:"beefy-activation-block"`
	BasicChannelAccounts []string `mapstructure:"basicChannelAccounts"`
}

func (c *SourceConfig) getAccounts() ([][32]byte, error) {
	var accounts [][32]byte

	for _, account := range c.BasicChannelAccounts {
		trimmedAccount := strings.TrimPrefix(account, "0x")
		accountBytes, err := hex.DecodeString(trimmedAccount)

		if err != nil {
			return nil, fmt.Errorf("decode account id: %w", err)
		} else if len(accountBytes) != 32 {
			// The conversion below will panic if decodedAccount has
			// fewer than 32 bytes: we expect exactly 32 bytes.
			return nil, fmt.Errorf("account id was not 32 bytes long: %v", accountBytes)
		}

		decodedAccount := *(*[32]byte)(accountBytes)
		accounts = append(accounts, decodedAccount)
	}

	return accounts, nil
}

type SourceContractsConfig struct {
	BeefyClient         string `mapstructure:"BeefyClient"`
	BasicInboundChannel string `mapstructure:"BasicInboundChannel"`
}

type SinkConfig struct {
	Ethereum  config.EthereumConfig `mapstructure:"ethereum"`
	Contracts SinkContractsConfig   `mapstructure:"contracts"`
}

type SinkContractsConfig struct {
	BasicInboundChannel string `mapstructure:"BasicInboundChannel"`
}
