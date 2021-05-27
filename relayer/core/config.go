package core

import (
	"fmt"
	"os"
	"strings"

	"github.com/snowfork/polkadot-ethereum/relayer/chain/ethereum"
	"github.com/snowfork/polkadot-ethereum/relayer/chain/parachain"
	"github.com/snowfork/polkadot-ethereum/relayer/chain/relaychain"
	"github.com/snowfork/polkadot-ethereum/relayer/workers"
	"github.com/snowfork/polkadot-ethereum/relayer/workers/beefyrelayer/store"
	"github.com/spf13/viper"
)

type WorkerConfig struct {
	ParachainCommitmentRelayer workers.WorkerConfig `mapstructure:"parachaincommitmentrelayer"`
	BeefyRelayer               workers.WorkerConfig `mapstructure:"beefyrelayer"`
	EthRelayer                 workers.WorkerConfig `mapstructure:"ethrelayer"`
}

type Config struct {
	Eth                  ethereum.Config   `mapstructure:"ethereum"`
	Parachain            parachain.Config  `mapstructure:"parachain"`
	Relaychain           relaychain.Config `mapstructure:"relaychain"`
	BeefyRelayerDatabase store.Config      `mapstructure:"database"`
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
	value, ok = os.LookupEnv("BEEFY_RELAYER_ETHEREUM_KEY")
	if !ok {
		return nil, fmt.Errorf("environment variable not set: BEEFY_RELAYER_ETHEREUM_KEY")
	}
	config.Eth.BeefyPrivateKey = strings.TrimPrefix(value, "0x")

	value, ok = os.LookupEnv("PARACHAIN_COMMITMENT_RELAYER_ETHEREUM_KEY")
	if !ok {
		return nil, fmt.Errorf("environment variable not set: PARACHAIN_COMMITMENT_RELAYER_ETHEREUM_KEY")
	}
	config.Eth.ParachainCommitmentsPrivateKey = strings.TrimPrefix(value, "0x")

	// Parachain configuration
	value, ok = os.LookupEnv("ARTEMIS_PARACHAIN_KEY")
	if !ok {
		return nil, fmt.Errorf("environment variable not set: ARTEMIS_PARACHAIN_KEY")
	}
	config.Parachain.PrivateKey = value

	// Relaychain configuration
	value, ok = os.LookupEnv("ARTEMIS_RELAYCHAIN_KEY")
	if !ok {
		return nil, fmt.Errorf("environment variable not set: ARTEMIS_RELAYCHAIN_KEY")
	}
	config.Relaychain.PrivateKey = value

	return &config, nil
}
