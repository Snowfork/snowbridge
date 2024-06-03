package config

import (
	"errors"
	"fmt"
	"github.com/snowfork/snowbridge/relayer/config"
)

type Config struct {
	Source SourceConfig `mapstructure:"source"`
	Sink   SinkConfig   `mapstructure:"sink"`
}

type SpecSettings struct {
	SyncCommitteeSize            uint64 `mapstructure:"syncCommitteeSize"`
	SlotsInEpoch                 uint64 `mapstructure:"slotsInEpoch"`
	EpochsPerSyncCommitteePeriod uint64 `mapstructure:"epochsPerSyncCommitteePeriod"`
	DenebForkEpoch               uint64 `mapstructure:"denebForkedEpoch"`
}

type SourceConfig struct {
	Beacon BeaconConfig `mapstructure:"beacon"`
}

type DataStore struct {
	Location   string `mapstructure:"location"`
	MaxEntries uint64 `mapstructure:"maxEntries"`
}

type BeaconConfig struct {
	Endpoint      string       `mapstructure:"endpoint"`
	StateEndpoint string       `mapstructure:"stateEndpoint"`
	Spec          SpecSettings `mapstructure:"spec"`
	DataStore     DataStore    `mapstructure:"datastore"`
}

type SinkConfig struct {
	Parachain config.ParachainConfig `mapstructure:"parachain"`
}

func (c Config) Validate() error {
	err := c.Source.Beacon.Validate()
	if err != nil {
		return fmt.Errorf("beacon config validation: %w", err)
	}
	err = c.Sink.Parachain.Validate()
	if err != nil {
		return fmt.Errorf("parachain config validation: %w", err)
	}
	return nil
}

func (b BeaconConfig) Validate() error {
	// spec settings
	if b.Spec.EpochsPerSyncCommitteePeriod == 0 {
		return errors.New("setting EpochsPerSyncCommitteePeriod is 0")
	}
	if b.Spec.SlotsInEpoch == 0 {
		return errors.New("setting SlotsInEpoch is 0")
	}
	if b.Spec.SyncCommitteeSize == 0 {
		return errors.New("setting SyncCommitteeSize is 0")
	}
	// data store
	if b.DataStore.Location == "" {
		return errors.New("datastore Location is empty")
	}
	if b.DataStore.MaxEntries == 0 {
		return errors.New("datastore MaxEntries is 0")
	}
	// api endpoints
	if b.Endpoint == "" {
		return errors.New("beacon Endpoint is empty")
	}
	if b.StateEndpoint == "" {
		return errors.New("beacon StateEndpoint is empty")
	}
	return nil
}
