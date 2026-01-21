package beaconstate

import (
	"errors"
	"fmt"
	"time"

	beaconconf "github.com/snowfork/snowbridge/relayer/relays/beacon/config"
)

type Config struct {
	Beacon BeaconConfig `mapstructure:"beacon"`
	HTTP   HTTPConfig   `mapstructure:"http"`
	Cache  CacheConfig  `mapstructure:"cache"`
}

type BeaconConfig struct {
	Endpoint      string                   `mapstructure:"endpoint"`
	StateEndpoint string                   `mapstructure:"stateEndpoint"`
	Spec          beaconconf.SpecSettings  `mapstructure:"spec"`
	DataStore     beaconconf.DataStore     `mapstructure:"datastore"`
}

type HTTPConfig struct {
	Port         int    `mapstructure:"port"`
	ReadTimeout  string `mapstructure:"readTimeout"`
	WriteTimeout string `mapstructure:"writeTimeout"`
}

type CacheConfig struct {
	MaxStates       int `mapstructure:"maxStates"`
	MaxProofs       int `mapstructure:"maxProofs"`
	StateTTLSeconds int `mapstructure:"stateTTLSeconds"`
	ProofTTLSeconds int `mapstructure:"proofTTLSeconds"`
}

func (c Config) Validate() error {
	err := c.Beacon.Validate()
	if err != nil {
		return fmt.Errorf("beacon config: %w", err)
	}
	err = c.HTTP.Validate()
	if err != nil {
		return fmt.Errorf("http config: %w", err)
	}
	err = c.Cache.Validate()
	if err != nil {
		return fmt.Errorf("cache config: %w", err)
	}
	return nil
}

func (b BeaconConfig) Validate() error {
	if b.Endpoint == "" {
		return errors.New("[endpoint] is not set")
	}
	if b.StateEndpoint == "" {
		return errors.New("[stateEndpoint] is not set")
	}
	if b.Spec.EpochsPerSyncCommitteePeriod == 0 {
		return errors.New("spec [epochsPerSyncCommitteePeriod] is not set")
	}
	if b.Spec.SlotsInEpoch == 0 {
		return errors.New("spec [slotsInEpoch] is not set")
	}
	if b.Spec.SyncCommitteeSize == 0 {
		return errors.New("spec [syncCommitteeSize] is not set")
	}
	if b.DataStore.Location == "" {
		return errors.New("datastore [location] is not set")
	}
	if b.DataStore.MaxEntries == 0 {
		return errors.New("datastore [maxEntries] is not set")
	}
	return nil
}

func (h HTTPConfig) Validate() error {
	if h.Port == 0 {
		return errors.New("[port] is not set")
	}
	if h.ReadTimeout == "" {
		return errors.New("[readTimeout] is not set")
	}
	if h.WriteTimeout == "" {
		return errors.New("[writeTimeout] is not set")
	}
	_, err := time.ParseDuration(h.ReadTimeout)
	if err != nil {
		return fmt.Errorf("invalid readTimeout: %w", err)
	}
	_, err = time.ParseDuration(h.WriteTimeout)
	if err != nil {
		return fmt.Errorf("invalid writeTimeout: %w", err)
	}
	return nil
}

func (c CacheConfig) Validate() error {
	if c.MaxStates == 0 {
		return errors.New("[maxStates] is not set")
	}
	if c.MaxProofs == 0 {
		return errors.New("[maxProofs] is not set")
	}
	if c.StateTTLSeconds == 0 {
		return errors.New("[stateTTLSeconds] is not set")
	}
	if c.ProofTTLSeconds == 0 {
		return errors.New("[proofTTLSeconds] is not set")
	}
	return nil
}
