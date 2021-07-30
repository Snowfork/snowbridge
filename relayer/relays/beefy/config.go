package beefy

type Config struct {
	Polkadot PolkadotConfig `mapstructure:"polkadot"`
	Ethereum EthereumConfig `mapstructure:"ethereum"`
}

type PolkadotConfig struct {
	Endpoint string `mapstructure:"endpoint"`
}

type EthereumConfig struct {
	Endpoint              string          `mapstructure:"endpoint"`
	StartBlock            uint64          `mapstructure:"start-block"`
	DescendantsUntilFinal uint64          `mapstructure:"descendants-until-final"`
	Contracts             ContractsConfig `mapstructure:"contracts"`
}

type ContractsConfig struct {
	BeefyLightClient string `mapstructure:"BeefyLightClient"`
}
