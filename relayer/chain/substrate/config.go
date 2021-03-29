package substrate

type Config struct {
	Parachain  ParachainConfig  `mapstructure:"parachain"`
	Relaychain RelaychainConfig `mapstructure:"relaychain"`
}

type ParachainConfig struct {
	Endpoint   string `mapstructure:"endpoint"`
	PrivateKey string `mapstructure:"private-key"`
}

type RelaychainConfig struct {
	Endpoint   string `mapstructure:"endpoint"`
	PrivateKey string `mapstructure:"private-key"`
}
