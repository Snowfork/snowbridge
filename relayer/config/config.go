package config

import "errors"

type PolkadotConfig struct {
	Endpoint string `mapstructure:"endpoint"`
}

type ParachainConfig struct {
	Endpoint string `mapstructure:"endpoint"`
}

type EthereumConfig struct {
	Endpoint  string `mapstructure:"endpoint"`
	GasFeeCap uint64 `mapstructure:"gas-fee-cap"`
	GasTipCap uint64 `mapstructure:"gas-tip-cap"`
	GasLimit  uint64 `mapstructure:"gas-limit"`
}

type OFACConfig struct {
	Enabled bool   `mapstructure:"enabled"`
	ApiKey  string `mapstructure:"apiKey"`
}

func (p ParachainConfig) Validate() error {
	if p.Endpoint == "" {
		return errors.New("[endpoint] is not set")
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

func (o OFACConfig) Validate() error {
	if o.Enabled && o.ApiKey == "" {
		return errors.New("OFAC is enabled but no [apiKey] set")
	}
	return nil
}
