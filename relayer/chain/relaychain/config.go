package relaychain

type Config struct {
	Relaychain RelaychainConfig `mapstructure:"relaychain"`
	Ethereum   EthereumConfig   `mapstructure:"ethereum"`
}

type RelaychainConfig struct {
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
