package config

import (
	"errors"
	"fmt"
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
	Fulu    uint64 `mapstructure:"fulu"`
}

type SourceConfig struct {
	Beacon BeaconConfig `mapstructure:"beacon"`
}

type DataStore struct {
	Location   string `mapstructure:"location"`
	MaxEntries uint64 `mapstructure:"maxEntries"`
}

type BeaconConfig struct {
	Endpoint             string       `mapstructure:"endpoint"`
	StateServiceEndpoint string       `mapstructure:"stateServiceEndpoint"`
	Spec                 SpecSettings `mapstructure:"spec"`
	DataStore            DataStore    `mapstructure:"datastore"`
}

type SinkConfig struct {
	Parachain          ParachainConfig `mapstructure:"parachain"`
	UpdateSlotInterval uint64          `mapstructure:"updateSlotInterval"`
}

type ParachainConfig struct {
	Endpoint             string `mapstructure:"endpoint"`
	MaxWatchedExtrinsics int64  `mapstructure:"maxWatchedExtrinsics"`
	// The max number of header in the FinalizedBeaconStateBuffer on-chain.
	// https://github.com/paritytech/polkadot-sdk/blob/master/bridges/snowbridge/pallets/ethereum-client/src/types.rs#L23
	HeaderRedundancy uint64 `mapstructure:"headerRedundancy"`
	HeartbeatSecs    uint64 `mapstructure:"heartbeat-secs"`
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
	if err := b.validateCommon(); err != nil {
		return err
	}
	// state service is required for beacon relay
	if b.StateServiceEndpoint == "" {
		return errors.New("source beacon setting [stateServiceEndpoint] is not set")
	}
	return nil
}

// ValidateForStateService validates the beacon config for use by the beacon state service.
// Unlike Validate(), this requires DataStore settings instead of StateServiceEndpoint
// (since the beacon state service IS the state service).
func (b BeaconConfig) ValidateForStateService() error {
	if err := b.validateCommon(); err != nil {
		return err
	}
	// data store is required for beacon state service
	if b.DataStore.Location == "" {
		return errors.New("source beacon datastore [location] is not set")
	}
	if b.DataStore.MaxEntries == 0 {
		return errors.New("source beacon datastore [maxEntries] is not set")
	}
	return nil
}

func (b BeaconConfig) validateCommon() error {
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
	// api endpoint
	if b.Endpoint == "" {
		return errors.New("source beacon setting [endpoint] is not set")
	}
	return nil
}

func (p ParachainConfig) Validate() error {
	if p.Endpoint == "" {
		return errors.New("[endpoint] is not set")
	}
	if p.MaxWatchedExtrinsics == 0 {
		return errors.New("[maxWatchedExtrinsics] is not set")
	}
	if p.HeaderRedundancy == 0 {
		return errors.New("[headerRedundancy] is not set")
	}
	return nil
}
