package substrate

import "github.com/snowfork/go-substrate-rpc-client/v2/types"

type Config struct {
	Endpoint            string   `mapstructure:"endpoint"`
	PrivateKey          string   `mapstructure:"private-key"`
	AccountWhitelist    []string `mapstructure:"account_whitelist"`
	AccountWhitelistMap map[types.AccountID]bool
}
