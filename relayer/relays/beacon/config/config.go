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
	SyncCommitteeSize            uint64       `mapstructure:"syncCommitteeSize"`
	SlotsInEpoch                 uint64       `mapstructure:"slotsInEpoch"`
	EpochsPerSyncCommitteePeriod uint64       `mapstructure:"epochsPerSyncCommitteePeriod"`
	ForkVersions                 ForkVersions `mapstructure:"forkVersions"`
}

type ForkVersions struct {
	Deneb   uint64 `mapstructure:"deneb"`
	Electra uint64 `mapstructure:"electra"`
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
	Parachain          config.ParachainConfig `mapstructure:"parachain"`
	UpdateSlotInterval uint64                 `mapstructure:"updateSlotInterval"`
}

func (c Config) Validate() error {
	err := c.Source.Beacon.Validate()
	if err != nil {
		return fmt.Errorf("source beacon config: %w", err)
	}
	err = c.Sink.Parachain.Validate()
	if err != nil {
		return fmt.Errorf("sink parachain config: %w", err)
	}
	if c.Sink.UpdateSlotInterval == 0 {
		return errors.New("parachain [updateSlotInterval] config is not set")
	}
	return nil
}

func (b BeaconConfig) Validate() error {
	// spec settings
	if b.Spec.EpochsPerSyncCommitteePeriod == 0 {
		return errors.New("source beacon setting [epochsPerSyncCommitteePeriod] is not set")
	}
	if b.Spec.SlotsInEpoch == 0 {
		return errors.New("source beacon setting [slotsInEpoch] is not set")
	}
	if b.Spec.SyncCommitteeSize == 0 {
		return errors.New("source beacon setting [syncCommitteeSize] is not set")
	}
	// data store
	if b.DataStore.Location == "" {
		return errors.New("source beacon datastore [location] is not set")
	}
	if b.DataStore.MaxEntries == 0 {
		return errors.New("source beacon datastore [maxEntries] is not set")
	}
	// api endpoints
	if b.Endpoint == "" {
		return errors.New("source beacon setting [endpoint] is not set")
	}
	if b.StateEndpoint == "" {
		return errors.New("source beacon setting [stateEndpoint] is not set")
	}
	return nil
}
