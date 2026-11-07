package parachainV3

import (
	"fmt"
	"strings"

	"github.com/snowfork/snowbridge/relayer/config"
)

type Config struct {
	Source        SourceConfig      `mapstructure:"source"`
	Sink          SinkConfig        `mapstructure:"sink"`
	RewardAddress string            `mapstructure:"reward-address"`
	OFAC         config.OFACConfig `mapstructure:"ofac"`
	IgnoreNonces []uint64          `mapstructure:"ignoreNonces"`
	// DeadNoncesPath is the file path for a persistent set of nonces to skip at
	// scan time. Operators can edit this file to permanently skip un-relayable
	// nonces across relayer restarts. When empty, the store is in-memory only.
	DeadNoncesPath string `mapstructure:"deadNoncesPath"`
	// ExtraPrivateSubmitEndpoints are submit-only builder RPCs (Titan, Quasar) that receive the
	// same signed fast-path tx after Flashbots. Empty entries (unset envsubst) are ignored.
	// These endpoints need not support eth_chainId / heartbeats.
	// Hosts labeled titan or quasar are sent with eth_sendBundle; any other host gets
	// eth_sendRawTransaction.
	ExtraPrivateSubmitEndpoints []string `mapstructure:"extra-private-submit-endpoints"`
}

// PrivateSubmitEndpoints returns non-empty extra private builder submit URLs.
func (c *Config) PrivateSubmitEndpoints() []string {
	if c == nil {
		return nil
	}
	out := make([]string, 0, len(c.ExtraPrivateSubmitEndpoints))
	for _, e := range c.ExtraPrivateSubmitEndpoints {
		if e = strings.TrimSpace(e); e != "" {
			out = append(out, e)
		}
	}
	return out
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
}

type SinkContractsConfig struct {
	Gateway    string `mapstructure:"Gateway"`
	Multicall3 string `mapstructure:"Multicall3"`
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

	err = c.OFAC.Validate()
	if err != nil {
		return fmt.Errorf("ofac config: %w", err)
	}

	if c.RewardAddress == "" {
		return fmt.Errorf("reward address is not set")
	}

	return nil
}

// ValidateForInstantRelay checks fields required when running the parachain relayer with instant verification
// (batched Fiat-Shamir + v2_submit via Multicall3).
func (c Config) ValidateForInstantRelay() error {
	if c.Sink.Contracts.Multicall3 == "" {
		return fmt.Errorf("sink contracts setting [Multicall3] is not set (required for instant verification)")
	}
	return nil
}
