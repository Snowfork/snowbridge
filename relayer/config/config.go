package config

type PolkadotConfig struct {
	Endpoint           string `mapstructure:"endpoint"`
	BeefyStartingBlock uint64 `mapstructure:"beefy-starting-block"`
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
