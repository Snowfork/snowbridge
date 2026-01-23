package beaconstate

import (
	"errors"
	"fmt"
	"time"

	beaconconf "github.com/snowfork/snowbridge/relayer/relays/beacon/config"
)

type Config struct {
	Beacon  beaconconf.BeaconConfig `mapstructure:"beacon"`
	HTTP    HTTPConfig              `mapstructure:"http"`
	Cache   CacheConfig             `mapstructure:"cache"`
	Persist PersistConfig           `mapstructure:"persist"`
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

type PersistConfig struct {
	// Enabled controls whether periodic state saving is enabled
	Enabled bool `mapstructure:"enabled"`
	// SaveIntervalHours is how often to save states to disk (in hours)
	SaveIntervalHours int `mapstructure:"saveIntervalHours"`
	// MaxEntries is the maximum number of beacon state entries to keep in the persistent store
	// Older entries are pruned after each save
	MaxEntries uint64 `mapstructure:"maxEntries"`
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
	err = c.Persist.Validate()
	if err != nil {
		return fmt.Errorf("persist config: %w", err)
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

func (p PersistConfig) Validate() error {
	if !p.Enabled {
		return nil
	}
	if p.SaveIntervalHours == 0 {
		return errors.New("[persist.saveIntervalHours] is not set")
	}
	if p.MaxEntries == 0 {
		return errors.New("[persist.maxEntries] is not set")
	}
	return nil
}
