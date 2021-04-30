package ethereum

type Config struct {
	Endpoint              string         `mapstructure:"endpoint"`
	PrivateKey            string         `mapstructure:"private-key"`
	DescendantsUntilFinal byte           `mapstructure:"descendants-until-final"`
	Channels              ChannelsConfig `mapstructure:"channels"`
	LightClientBridge     string         `mapstructure:"lightclientbridge"`
	StartBlock            uint64         `mapstructure:"startblock"`
}

type ChannelsConfig struct {
	Basic        ChannelConfig `mapstructure:"basic"`
	Incentivized ChannelConfig `mapstructure:"incentivized"`
}

type ChannelConfig struct {
	Inbound  string `mapstructure:"inbound"`
	Outbound string `mapstructure:"outbound"`
}
