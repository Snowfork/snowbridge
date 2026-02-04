package beefy

import (
	"fmt"

	"github.com/snowfork/snowbridge/relayer/config"
)

type Config struct {
	Source       SourceConfig       `mapstructure:"source"`
	Sink         SinkConfig         `mapstructure:"sink"`
	OnDemandSync OnDemandSyncConfig `mapstructure:"on-demand-sync"`
}

type SourceConfig struct {
	Polkadot  config.PolkadotConfig  `mapstructure:"polkadot"`
	BridgeHub config.ParachainConfig `mapstructure:"bridge-hub"`
}

type SinkConfig struct {
	Ethereum  config.EthereumConfig `mapstructure:"ethereum"`
	Contracts ContractsConfig       `mapstructure:"contracts"`
}

type ContractsConfig struct {
	BeefyClient string `mapstructure:"BeefyClient"`
	Gateway     string `mapstructure:"Gateway"`
}

type OnDemandSyncConfig struct {
	// ID of the AssetHub channel
	AssetHubChannelID string `mapstructure:"asset-hub-channel-id"`
	// Maximum number of tasks that can run concurrently
	MaxTasks uint64 `mapstructure:"max-tasks"`
	// Time Period (in seconds) during which a previous task can be merged
	MergePeriod uint64 `mapstructure:"merge-period"`
	// Time period (in seconds) after which merging is not allowed
	ExpiredPeriod uint64 `mapstructure:"expired-period"`
}

func (c Config) Validate() error {
	err := c.Source.Polkadot.Validate()
	if err != nil {
		return fmt.Errorf("source polkadot config: %w", err)
	}
	err = c.Sink.Ethereum.Validate()
	if err != nil {
		return fmt.Errorf("sink ethereum config: %w", err)
	}
	if c.Sink.Contracts.BeefyClient == "" {
		return fmt.Errorf("sink contracts setting [BeefyClient] is not set")
	}
	if c.Sink.Contracts.Gateway == "" {
		return fmt.Errorf("sink contracts setting [Gateway] is not set")
	}
	if c.OnDemandSync.AssetHubChannelID == "" {
		return fmt.Errorf("`on-demand-sync.asset-hub-channel-id` not set")
	}
	if c.OnDemandSync.MaxTasks == 0 || c.OnDemandSync.MaxTasks > 8 {
		return fmt.Errorf("`on-demand-sync.max-tasks` should be configured non zero and no more than 8")
	}
	if c.OnDemandSync.MergePeriod == 0 || c.OnDemandSync.MergePeriod > 1800 {
		return fmt.Errorf("`on-demand-sync.merge-period` should be configured non zero and no more than 1800 seconds")
	}
	if c.OnDemandSync.ExpiredPeriod == 0 || c.OnDemandSync.ExpiredPeriod > 14400 || c.OnDemandSync.ExpiredPeriod < c.OnDemandSync.MergePeriod {
		return fmt.Errorf("`on-demand-sync.expired-period` should be configured non zero and no more than 14400 seconds, and more than MergePeriod")
	}
	return nil
}
