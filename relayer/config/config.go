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
		return errors.New("parachain endpoint is not set")
	}
	if p.UpdateSlotInterval == 0 {
		return errors.New("parachain config UpdateSlotInterval is 0")
	}
	if p.MaxWatchedExtrinsics == 0 {
		return errors.New("parachain config MaxWatchedExtrinsics is 0")
	}
	if p.MaxBatchCallSize == 0 {
		return errors.New("parachain config MaxBatchCallSize is 0")
	}
	return nil
}
