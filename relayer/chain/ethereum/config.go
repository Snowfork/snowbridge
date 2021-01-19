package ethereum

type Config struct {
	Endpoint              string                  `mapstructure:"endpoint"`
	PrivateKey            string                  `mapstructure:"private-key"`
	DescendantsUntilFinal byte                    `mapstructure:"descendants-until-final"`
	Bridge                ContractInfo            `mapstructure:"bridge"`
	Apps                  map[string]ContractInfo `mapstructure:"apps"`
}

type ContractInfo struct {
	Address string `mapstructure:"address"`
	AbiPath string `mapstructure:"abi"`
}
