package ethereum

import "github.com/ethereum/go-ethereum/common"

type Config struct {
	Endpoint              string         `mapstructure:"endpoint"`
	PrivateKey            string         `mapstructure:"private-key"`
	DescendantsUntilFinal byte           `mapstructure:"descendants-until-final"`
	Channels              ChannelsConfig `mapstructure:"channels"`
}

type ChannelsConfig struct {
	Basic        ChannelConfig `mapstructure:"basic"`
	Incentivized ChannelConfig `mapstructure:"incentivized"`
}

type ChannelConfig struct {
	Inbound             string   `mapstructure:"inbound"`
	Outbound            string   `mapstructure:"outbound"`
	AccountWhitelist    []string `mapstructure:"account_whitelist"`
	AccountWhitelistMap map[common.Address]bool
}
