package ethereum

type Config struct {
	Endpoint   string                 `mapstructure:"endpoint"`
	PrivateKey string                 `mapstructure:"private-key"`
	Apps       map[string]Application `mapstructure:"apps"`
}

type Application struct {
	Address string `mapstructure:"address"`
	AbiPath string `mapstructure:"abi"`
}
