package substrate

type Config struct {
	Endpoint       string `mapstructure:"endpoint"`
	PrivateKey     string `mapstructure:"private-key"`
	CommitInterval uint32 `mapstructure:"commit-interval"`
	Targets        map[string][20]byte
}
