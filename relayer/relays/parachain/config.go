package parachain

import (
	"fmt"

	"github.com/snowfork/snowbridge/relayer/config"
)

type Config struct {
	Source        SourceConfig      `mapstructure:"source"`
	Sink          SinkConfig        `mapstructure:"sink"`
	RewardAddress string            `mapstructure:"reward-address"`
	OFAC          config.OFACConfig `mapstructure:"ofac"`
}

type SourceConfig struct {
	Polkadot  config.PolkadotConfig  `mapstructure:"polkadot"`
	Parachain config.ParachainConfig `mapstructure:"parachain"`
	Ethereum  config.EthereumConfig  `mapstructure:"ethereum"`
	Contracts SourceContractsConfig  `mapstructure:"contracts"`
}

type SourceContractsConfig struct {
	BeefyClient string `mapstructure:"BeefyClient"`
	Gateway     string `mapstructure:"Gateway"`
}

type SinkConfig struct {
	Ethereum  config.EthereumConfig `mapstructure:"ethereum"`
	Contracts SinkContractsConfig   `mapstructure:"contracts"`
	Fees      FeeConfig             `mapstructure:"fees"`
}

type SinkContractsConfig struct {
	Gateway string `mapstructure:"Gateway"`
}

type FeeConfig struct {
	// The gas cost of v2_submit excludes command execution, mainly covers the verification
	BaseDeliveryGas uint64 `mapstructure:"base-delivery-gas"`
	// The gas cost of unlock ERC20 token
	BaseUnlockGas uint64 `mapstructure:"base-unlock-gas"`
	// The gas cost of mint Polkadot native asset
	BaseMintGas         uint64 `mapstructure:"base-mint-gas"`
	FeeRatioNumerator   uint64 `mapstructure:"fee-ratio-numerator"`
	FeeRatioDenominator uint64 `mapstructure:"fee-ratio-denominator"`
}

func (f FeeConfig) Validate() error {
	if f.FeeRatioDenominator == 0 {
		return errors.New("fee-ratio-denominator must be non-zero")
	}
	if f.FeeRatioNumerator == 0 {
		return errors.New("fee-ratio-numerator must be non-zero")
	}
	return nil
}

func (c Config) Validate() error {
	// Source
	err := c.Source.Polkadot.Validate()
	if err != nil {
		return fmt.Errorf("source polkadot config: %w", err)
	}
	err = c.Source.Parachain.Validate()
	if err != nil {
		return fmt.Errorf("source parachain config: %w", err)
	}
	err = c.Source.Ethereum.Validate()
	if err != nil {
		return fmt.Errorf("source ethereum config: %w", err)
	}
	if c.Source.Contracts.BeefyClient == "" {
		return fmt.Errorf("source contracts setting [BeefyClient] is not set")
	}
	if c.Source.Contracts.Gateway == "" {
		return fmt.Errorf("source contracts setting [Gateway] is not set")
	}

	// Sink
	err = c.Sink.Ethereum.Validate()
	if err != nil {
		return fmt.Errorf("sink ethereum config: %w", err)
	}
	if c.Sink.Contracts.Gateway == "" {
		return fmt.Errorf("sink contracts setting [Gateway] is not set")
	}
	err = c.Sink.Fees.Validate()
	if err != nil {
		return fmt.Errorf("sink fees config: %w", err)
	}

	err = c.OFAC.Validate()
	if err != nil {
		return fmt.Errorf("ofac config: %w", err)
	}

	if c.RewardAddress == "" {
		return fmt.Errorf("reward address is not set")
	}

	return nil
}
