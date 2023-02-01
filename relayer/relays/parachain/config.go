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
	BeefyActivationBlock  uint64   `mapstructure:"beefy-activation-block"`
	BasicChannelSourceIDs []string `mapstructure:"basicChannelSourceIDs"`
}

func (c *SourceConfig) getSourceIDs() ([][32]byte, error) {
	var sourceIDs [][32]byte

	for _, sourceID := range c.BasicChannelSourceIDs {
		trimmedSourceID := strings.TrimPrefix(sourceID, "0x")
		sourceIDBytes, err := hex.DecodeString(trimmedSourceID)

		if err != nil {
			return nil, fmt.Errorf("decode source id: %w", err)
		} else if len(sourceIDBytes) != 32 {
			// The conversion below will panic if sourceIDBytes has
			// fewer than 32 bytes: we expect exactly 32 bytes.
			return nil, fmt.Errorf("source id was not 32 bytes long: %v", sourceIDBytes)
		}

		decodedSourceID := *(*[32]byte)(sourceIDBytes)
		sourceIDs = append(sourceIDs, decodedSourceID)
	}

	return sourceIDs, nil
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
