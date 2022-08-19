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
	AccountIDs           []string `mapstructure:"accountIDs"`
}

func (c *SourceConfig) getAccountIDs() ([][32]byte, error) {
	var accountIDs [][32]byte

	for _, accountID := range c.AccountIDs {
		trimmedAccountID := strings.TrimPrefix(accountID, "0x")
		accountIDBytes, err := hex.DecodeString(trimmedAccountID)

		if err != nil {
			return nil, fmt.Errorf("decode account id: %w", err)
		} else if len(accountIDBytes) != 32 {
			// The conversion below will panic if decodedAccount has
			// fewer than 32 bytes: we expect exactly 32 bytes.
			return nil, fmt.Errorf("account id was not 32 bytes long: %v", accountIDBytes)
		}

		decodedAccountID := *(*[32]byte)(accountIDBytes)
		accountIDs = append(accountIDs, decodedAccountID)
	}

	return accountIDs, nil
}

type SourceContractsConfig struct {
	BeefyClient                string `mapstructure:"BeefyClient"`
	BasicInboundChannel        string `mapstructure:"BasicInboundChannel"`
	IncentivizedInboundChannel string `mapstructure:"IncentivizedInboundChannel"`
}

type SinkConfig struct {
	Ethereum  config.EthereumConfig `mapstructure:"ethereum"`
	Contracts SinkContractsConfig   `mapstructure:"contracts"`
}

type SinkContractsConfig struct {
	BasicInboundChannel        string `mapstructure:"BasicInboundChannel"`
	IncentivizedInboundChannel string `mapstructure:"IncentivizedInboundChannel"`
}
