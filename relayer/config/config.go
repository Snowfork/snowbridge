package config

import "errors"

type PolkadotConfig struct {
	Endpoint string `mapstructure:"endpoint"`
}

type ParachainConfig struct {
	Endpoint             string `mapstructure:"endpoint"`
	MaxWatchedExtrinsics int64  `mapstructure:"maxWatchedExtrinsics"`
	MaxBatchCallSize     int64  `mapstructure:"maxBatchCallSize"`
	UpdateSlotInterval   uint64 `mapstructure:"updateSlotInterval"`
}

type EthereumConfig struct {
	Endpoint  string `mapstructure:"endpoint"`
	GasFeeCap uint64 `mapstructure:"gas-fee-cap"`
	GasTipCap uint64 `mapstructure:"gas-tip-cap"`
	GasLimit  uint64 `mapstructure:"gas-limit"`
}

func (p ParachainConfig) Validate() error {
	if p.Endpoint == "" {
		return errors.New("parachain [endpoint] config is not set")
	}
	if p.UpdateSlotInterval == 0 {
		return errors.New("parachain [updateSlotInterval] config is not set")
	}
	if p.MaxWatchedExtrinsics == 0 {
		return errors.New("parachain config [maxWatchedExtrinsics] is not set")
	}
	if p.MaxBatchCallSize == 0 {
		return errors.New("parachain config [maxBatchCallSize] is not set")
	}
	return nil
}

func (e EthereumConfig) Validate() error {
	if e.Endpoint == "" {
		return errors.New("ethereum [endpoint] config is not set")
	}
	return nil
}

func (p PolkadotConfig) Validate() error {
	if p.Endpoint == "" {
		return errors.New("polkadot [endpoint] config is not set")
	}
	return nil
}
