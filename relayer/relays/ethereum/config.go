package ethereum

type Config struct {
	DataDir   string          `mapstructure:"data-dir"`
	Parachain ParachainConfig `mapstructure:"parachain"`
	Ethereum  EthereumConfig  `mapstructure:"ethereum"`
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
	BasicOutboundChannel        string `mapstructure:"BasicOutboundChannel"`
	IncentivizedOutboundChannel string `mapstructure:"IncentivizedOutboundChannel"`
}
