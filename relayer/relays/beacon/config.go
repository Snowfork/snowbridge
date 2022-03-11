package beacon

type Config struct {
	Source SourceConfig `mapstructure:"source"`
}

type SourceConfig struct {
	Beacon BeaconConfig `mapstructure:"beacon"`
}

type BeaconConfig struct {
	Endpoint string `mapstructure:"endpoint"`
}
