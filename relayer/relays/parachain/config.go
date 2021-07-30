package parachain

type Config struct {
	Polkadot  PolkadotConfig  `mapstructure:"polkadot"`
	Parachain ParachainConfig `mapstructure:"parachain"`
	Ethereum  EthereumConfig  `mapstructure:"ethereum"`
}

type PolkadotConfig struct {
	Endpoint string `mapstructure:"endpoint"`
}

type ParachainConfig struct {
	Endpoint string `mapstructure:"endpoint"`
}

type EthereumConfig struct {
	Endpoint              string          `mapstructure:"endpoint"`
	DescendantsUntilFinal uint64          `mapstructure:"descendants-until-final"`
	Contracts             ContractsConfig `mapstructure:"contracts"`
}

type ContractsConfig struct {
	BeefyLightClient           string `mapstructure:"BeefyLightClient"`
	BasicInboundChannel        string `mapstructure:"BasicInboundChannel"`
	IncentivizedInboundChannel string `mapstructure:"IncentivizedInboundChannel"`
}
