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
	BeefyActivationBlock uint64 `mapstructure:"beefy-activation-block"`
	Account              string `mapstructure:"account"`
}

func (c *SourceConfig) getAccount() (*[32]byte, error) {
	accountID := strings.TrimPrefix(c.Account, "0x")
	decodedAccount, err := hex.DecodeString(accountID)
	if err != nil {
		return nil, fmt.Errorf("decode account id: %w", err)
	} else if len(decodedAccount) != 32 {
		// The conversion below will panic if decodedAccount has
		// fewer than 32 bytes.
		// We expect exactly 32 bytes.
		return nil, fmt.Errorf("account id was not 32 bytes long: %v", decodedAccount)
	}
	return (*[32]byte)(decodedAccount), nil
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
