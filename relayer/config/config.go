package config

type PolkadotConfig struct {
	Endpoint string `mapstructure:"endpoint"`
}

type ParachainConfig struct {
	Endpoint string `mapstructure:"endpoint"`
}

type EthereumConfig struct {
	Endpoint         string  `mapstructure:"endpoint"`
	GasFeeCap        uint64  `mapstructure:"gas-fee-cap"`
	GasFeeMultiplier float64 `mapstructure:"gas-fee-multiplier"`
	GasTipCap        uint64  `mapstructure:"gas-tip-cap"`
	GasTipMultiplier float64 `mapstructure:"gas-tip-multiplier"`
	GasLimit         uint64  `mapstructure:"gas-limit"`
}
