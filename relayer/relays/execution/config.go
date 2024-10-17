package execution

import (
	"errors"
	"fmt"

	"github.com/snowfork/snowbridge/relayer/config"
	beaconconf "github.com/snowfork/snowbridge/relayer/relays/beacon/config"
)

type Config struct {
	Source              SourceConfig      `mapstructure:"source"`
	Sink                SinkConfig        `mapstructure:"sink"`
	InstantVerification bool              `mapstructure:"instantVerification"`
	Schedule            ScheduleConfig    `mapstructure:"schedule"`
	OFAC                config.OFACConfig `mapstructure:"ofac"`
}

type ScheduleConfig struct {
	// ID of current relayer, starting from 0
	ID uint64 `mapstructure:"id"`
	// Number of total count of all relayers
	TotalRelayerCount uint64 `mapstructure:"totalRelayerCount"`
	// Sleep interval(in seconds) to check if message(nonce) has already been relayed
	SleepInterval uint64 `mapstructure:"sleepInterval"`
}

func (r ScheduleConfig) Validate() error {
	if r.TotalRelayerCount < 1 {
		return errors.New("Number of relayer is not set")
	}
	if r.ID >= r.TotalRelayerCount {
		return errors.New("ID of the Number of relayer is not set")
	}
	return nil
}

type SourceConfig struct {
	Ethereum  config.EthereumConfig   `mapstructure:"ethereum"`
	Contracts ContractsConfig         `mapstructure:"contracts"`
	ChannelID ChannelID               `mapstructure:"channel-id"`
	Beacon    beaconconf.BeaconConfig `mapstructure:"beacon"`
}

type ContractsConfig struct {
	Gateway string `mapstructure:"Gateway"`
}

type SinkConfig struct {
	Parachain  beaconconf.ParachainConfig `mapstructure:"parachain"`
	SS58Prefix uint8                      `mapstructure:"ss58Prefix"`
}

type ChannelID [32]byte

func (c Config) Validate() error {
	err := c.Source.Beacon.Validate()
	if err != nil {
		return fmt.Errorf("beacon config validation: %w", err)
	}
	err = c.Sink.Parachain.Validate()
	if err != nil {
		return fmt.Errorf("parachain config validation: %w", err)
	}
	if c.Source.ChannelID == [32]byte{} {
		return fmt.Errorf("source setting [channel-id] is not set")
	}
	if c.Source.Contracts.Gateway == "" {
		return fmt.Errorf("source setting [gateway] is not set")
	}
	err = c.Schedule.Validate()
	if err != nil {
		return fmt.Errorf("schedule config: %w", err)
	}
	err = c.OFAC.Validate()
	if err != nil {
		return fmt.Errorf("ofac config: %w", err)
	}
	return nil
}
