package core

import (
	"fmt"
	"os"
	"strings"

	"github.com/snowfork/snowbridge/relayer/chain/ethereum"
	"github.com/snowfork/snowbridge/relayer/chain/parachain"
	"github.com/snowfork/snowbridge/relayer/chain/relaychain"
	worker "github.com/snowfork/snowbridge/relayer/relays"
	"github.com/spf13/viper"
)

type WorkerConfig struct {
	ParachainCommitmentRelayer worker.WorkerConfig `mapstructure:"parachaincommitmentrelayer"`
	BeefyRelayer               worker.WorkerConfig `mapstructure:"beefyrelayer"`
	EthRelayer                 worker.WorkerConfig `mapstructure:"ethrelayer"`
}

type GlobalConfig struct {
	DataDir string `mapstructure:"data-dir"`
}

type Config struct {
	Global               GlobalConfig      `mapstructure:"global"`
	Eth                  ethereum.Config   `mapstructure:"ethereum"`
	Parachain            parachain.Config  `mapstructure:"parachain"`
	Relaychain           relaychain.Config `mapstructure:"relaychain"`
	Workers              WorkerConfig      `mapstructure:"workers"`
}

func LoadConfig() (*Config, error) {
	var config Config
	err := viper.Unmarshal(&config)
	if err != nil {
		return nil, err
	}

	// Load secrets from environment variables
	var value string
	var ok bool

	// Ethereum configuration

	value, ok = os.LookupEnv("SNOWBRIDGE_MESSAGE_KEY")
	if !ok {
		return nil, fmt.Errorf("environment variable not set: SNOWBRIDGE_MESSAGE_KEY")
	}
	config.Eth.MessagePrivateKey = strings.TrimPrefix(value, "0x")

	// Parachain configuration
	value, ok = os.LookupEnv("SNOWBRIDGE_PARACHAIN_KEY")
	if !ok {
		return nil, fmt.Errorf("environment variable not set: SNOWBRIDGE_PARACHAIN_KEY")
	}
	config.Parachain.PrivateKey = value

	// Relaychain configuration
	value, ok = os.LookupEnv("SNOWBRIDGE_RELAYCHAIN_KEY")
	if !ok {
		return nil, fmt.Errorf("environment variable not set: SNOWBRIDGE_RELAYCHAIN_KEY")
	}
	config.Relaychain.PrivateKey = value

	return &config, nil
}
