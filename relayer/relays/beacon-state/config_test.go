package beaconstate

import (
	"testing"

	beaconconf "github.com/snowfork/snowbridge/relayer/relays/beacon/config"
)

func TestHTTPConfig_Validate(t *testing.T) {
	tests := []struct {
		name    string
		config  HTTPConfig
		wantErr bool
	}{
		{
			name: "valid config",
			config: HTTPConfig{
				Port:         8080,
				ReadTimeout:  "30s",
				WriteTimeout: "30s",
			},
			wantErr: false,
		},
		{
			name: "missing port",
			config: HTTPConfig{
				Port:         0,
				ReadTimeout:  "30s",
				WriteTimeout: "30s",
			},
			wantErr: true,
		},
		{
			name: "missing read timeout",
			config: HTTPConfig{
				Port:         8080,
				ReadTimeout:  "",
				WriteTimeout: "30s",
			},
			wantErr: true,
		},
		{
			name: "missing write timeout",
			config: HTTPConfig{
				Port:         8080,
				ReadTimeout:  "30s",
				WriteTimeout: "",
			},
			wantErr: true,
		},
		{
			name: "invalid read timeout format",
			config: HTTPConfig{
				Port:         8080,
				ReadTimeout:  "invalid",
				WriteTimeout: "30s",
			},
			wantErr: true,
		},
		{
			name: "invalid write timeout format",
			config: HTTPConfig{
				Port:         8080,
				ReadTimeout:  "30s",
				WriteTimeout: "invalid",
			},
			wantErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := tt.config.Validate()
			if (err != nil) != tt.wantErr {
				t.Errorf("Validate() error = %v, wantErr %v", err, tt.wantErr)
			}
		})
	}
}

func TestCacheConfig_Validate(t *testing.T) {
	tests := []struct {
		name    string
		config  CacheConfig
		wantErr bool
	}{
		{
			name: "valid config",
			config: CacheConfig{
				MaxProofs:       100,
				ProofTTLSeconds: 3600,
			},
			wantErr: false,
		},
		{
			name: "missing max proofs",
			config: CacheConfig{
				MaxProofs:       0,
				ProofTTLSeconds: 3600,
			},
			wantErr: true,
		},
		{
			name: "missing proof TTL",
			config: CacheConfig{
				MaxProofs:       100,
				ProofTTLSeconds: 0,
			},
			wantErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := tt.config.Validate()
			if (err != nil) != tt.wantErr {
				t.Errorf("Validate() error = %v, wantErr %v", err, tt.wantErr)
			}
		})
	}
}

func TestPersistConfig_Validate(t *testing.T) {
	tests := []struct {
		name    string
		config  PersistConfig
		wantErr bool
	}{
		{
			name: "disabled config is always valid",
			config: PersistConfig{
				Enabled:           false,
				SaveIntervalHours: 0,
				MaxEntries:        0,
			},
			wantErr: false,
		},
		{
			name: "valid enabled config",
			config: PersistConfig{
				Enabled:           true,
				SaveIntervalHours: 24,
				MaxEntries:        10,
			},
			wantErr: false,
		},
		{
			name: "enabled but missing save interval",
			config: PersistConfig{
				Enabled:           true,
				SaveIntervalHours: 0,
				MaxEntries:        10,
			},
			wantErr: true,
		},
		{
			name: "enabled but missing max entries",
			config: PersistConfig{
				Enabled:           true,
				SaveIntervalHours: 24,
				MaxEntries:        0,
			},
			wantErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := tt.config.Validate()
			if (err != nil) != tt.wantErr {
				t.Errorf("Validate() error = %v, wantErr %v", err, tt.wantErr)
			}
		})
	}
}

func TestWatchConfig_Validate(t *testing.T) {
	tests := []struct {
		name    string
		config  WatchConfig
		wantErr bool
	}{
		{
			name: "disabled config is always valid",
			config: WatchConfig{
				Enabled:             false,
				PollIntervalSeconds: 0,
			},
			wantErr: false,
		},
		{
			name: "valid enabled config",
			config: WatchConfig{
				Enabled:             true,
				PollIntervalSeconds: 12,
			},
			wantErr: false,
		},
		{
			name: "enabled but missing poll interval",
			config: WatchConfig{
				Enabled:             true,
				PollIntervalSeconds: 0,
			},
			wantErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := tt.config.Validate()
			if (err != nil) != tt.wantErr {
				t.Errorf("Validate() error = %v, wantErr %v", err, tt.wantErr)
			}
		})
	}
}

func TestConfig_Validate(t *testing.T) {
	validBeaconConfig := beaconconf.BeaconConfig{
		Endpoint: "http://localhost:5052",
		Spec: beaconconf.SpecSettings{
			SyncCommitteeSize:            512,
			SlotsInEpoch:                 32,
			EpochsPerSyncCommitteePeriod: 256,
			ForkVersions: beaconconf.ForkVersions{
				Deneb:   0,
				Electra: 1000000,
				Fulu:    1000000,
			},
		},
		DataStore: beaconconf.DataStore{
			Location:   "/tmp/test",
			MaxEntries: 10,
		},
	}

	tests := []struct {
		name    string
		config  Config
		wantErr bool
	}{
		{
			name: "valid config",
			config: Config{
				Beacon: validBeaconConfig,
				HTTP: HTTPConfig{
					Port:         8080,
					ReadTimeout:  "30s",
					WriteTimeout: "30s",
				},
				Cache: CacheConfig{
					MaxProofs:       100,
					ProofTTLSeconds: 3600,
				},
				Persist: PersistConfig{
					Enabled: false,
				},
				Watch: WatchConfig{
					Enabled: false,
				},
			},
			wantErr: false,
		},
		{
			name: "invalid http config",
			config: Config{
				Beacon: validBeaconConfig,
				HTTP: HTTPConfig{
					Port: 0, // Invalid
				},
				Cache: CacheConfig{
					MaxProofs:       100,
					ProofTTLSeconds: 3600,
				},
			},
			wantErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := tt.config.Validate()
			if (err != nil) != tt.wantErr {
				t.Errorf("Validate() error = %v, wantErr %v", err, tt.wantErr)
			}
		})
	}
}
