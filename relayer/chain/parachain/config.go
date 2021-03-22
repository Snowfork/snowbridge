package parachain

type Config struct {
	Parachain ParachainConfig `mapstructure:"parachain"`
	Ethereum  EthereumConfig  `mapstructure:"ethereum"`
}

type ParachainConfig struct {
	Endpoint   string `mapstructure:"endpoint"`
	PrivateKey string `mapstructure:"private-key"`
}

type EthereumConfig struct {
	Endpoint        string    `mapstructure:"endpoint"`
	PrivateKey      string    `mapstructure:"private-key"`
	Contracts       Contracts `mapstructure:"contracts"`
	BeefyBlockDelay uint64    `mapstructure:"beefy-block-delay"`
}

type Contracts struct {
	RelayBridgeLightClient string `mapstructure:"relay-bridge-light-client"`
	ValidatorRegistry      string `mapstructure:"validator-registry"`
}
