package parachain

type Config struct {
	Endpoint   string `mapstructure:"endpoint"`
	PrivateKey string `mapstructure:"private-key"`
}
