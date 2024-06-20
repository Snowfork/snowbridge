package config

import "errors"

type PolkadotConfig struct {
	Endpoint string `mapstructure:"endpoint"`
}

type ParachainConfig struct {
	Endpoint             string `mapstructure:"endpoint"`
	MaxWatchedExtrinsics int64  `mapstructure:"maxWatchedExtrinsics"`
}

type EthereumConfig struct {
	Endpoint  string `mapstructure:"endpoint"`
	GasFeeCap uint64 `mapstructure:"gas-fee-cap"`
	GasTipCap uint64 `mapstructure:"gas-tip-cap"`
	GasLimit  uint64 `mapstructure:"gas-limit"`
}

func (p ParachainConfig) Validate() error {
	if p.Endpoint == "" {
		return errors.New("[endpoint] is not set")
	}
	if p.MaxWatchedExtrinsics == 0 {
		return errors.New("[maxWatchedExtrinsics] is not set")
	}
	return nil
}

func (e EthereumConfig) Validate() error {
	if e.Endpoint == "" {
		return errors.New("[endpoint] config is not set")
	}
	return nil
}

func (p PolkadotConfig) Validate() error {
	if p.Endpoint == "" {
		return errors.New("[endpoint] config is not set")
	}
	return nil
}
