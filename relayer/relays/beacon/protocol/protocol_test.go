package protocol

import (
	"errors"
	"testing"

	"github.com/snowfork/snowbridge/relayer/relays/beacon/config"
	"github.com/stretchr/testify/assert"
)

func TestIsStartOfEpoch(t *testing.T) {
	values := []struct {
		name     string
		slot     uint64
		expected bool
	}{
		{
			name:     "start of epoch",
			slot:     0,
			expected: true,
		},
		{
			name:     "middle of epoch",
			slot:     16,
			expected: false,
		},
		{
			name:     "end of epoch",
			slot:     31,
			expected: false,
		},
		{
			name:     "start of new of epoch",
			slot:     32,
			expected: true,
		},
	}

	p := Protocol{}
	p.Settings.SlotsInEpoch = 32

	for _, tt := range values {
		result := p.IsStartOfEpoch(tt.slot)
		assert.Equal(t, tt.expected, result, "expected %t but found %t for slot %d", tt.expected, result, tt.slot)
	}
}

func TestCalculateNextCheckpointSlot(t *testing.T) {
	values := []struct {
		name     string
		slot     uint64
		expected uint64
	}{
		{
			name:     "slot 41",
			slot:     41,
			expected: 64,
		},
		{
			name:     "slot 64",
			slot:     64,
			expected: 64,
		},
		{
			name:     "slot 78",
			slot:     78,
			expected: 128,
		},
	}

	p := Protocol{}
	p.Settings.SlotsInEpoch = 8
	p.Settings.EpochsPerSyncCommitteePeriod = 8

	for _, tt := range values {
		result := p.CalculateNextCheckpointSlot(tt.slot)
		assert.Equal(t, tt.expected, result, "expected %t but found %t for slot %d", tt.expected, result, tt.slot)
	}
}

func TestSyncCommitteeBits(t *testing.T) {
	values := []struct {
		name     string
		bits     string
		expected bool
		err      error
	}{
		{
			name:     "empty1",
			bits:     "0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
			expected: false,
			err:      nil,
		},
		{
			name:     "not supermajority",
			bits:     "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff0000000000000000000000000000000000000000000000000000000000000000",
			expected: false,
			err:      nil,
		},
		{
			name:     "supermajority",
			bits:     "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff000000000000000000000000000000000000000000",
			expected: true,
			err:      nil,
		},
		{
			name:     "invalid hex",
			bits:     "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff00000000000000000000000000000000000000000",
			expected: false,
			err:      errors.New("encoding/hex: odd length hex string"),
		},
	}

	p := Protocol{}
	p.Settings.SyncCommitteeSize = 512

	for _, tt := range values {
		result, err := p.SyncCommitteeSuperMajority(tt.bits)
		assert.Equal(t, tt.err, err, "expected %t but found %t", tt.err, err)
		assert.Equal(t, tt.expected, result, "expected %t but found %t", tt.expected, result)
	}
}

func TestForkVersion(t *testing.T) {
	values := []struct {
		name     string
		slot     uint64
		expected ForkVersion
	}{
		{
			name:     "deneb fork - slot before electra epoch",
			slot:     0,
			expected: Deneb,
		},
		{
			name:     "deneb fork - slot just before electra activation",
			slot:     25599999, // epoch 799999
			expected: Deneb,
		},
		{
			name:     "electra fork - first slot of electra epoch",
			slot:     25600000, // epoch 800000 (32 slots per epoch)
			expected: Electra,
		},
		{
			name:     "electra fork - middle of electra era",
			slot:     29000000, // epoch ~906250
			expected: Electra,
		},
		{
			name:     "electra fork - just before fulu activation",
			slot:     31999999, // epoch 999999
			expected: Electra,
		},
		{
			name:     "fulu fork - first slot of fulu epoch",
			slot:     32000000, // epoch 1000000 (32 slots per epoch)
			expected: Fulu,
		},
		{
			name:     "fulu fork - well into fulu era",
			slot:     50000000, // epoch 1562500
			expected: Fulu,
		},
	}

	p := Protocol{}
	p.Settings.SlotsInEpoch = 32
	p.Settings.ForkVersions = config.ForkVersions{
		Deneb:   0,
		Electra: 800000,
		Fulu:    1000000,
	}

	for _, tt := range values {
		t.Run(tt.name, func(t *testing.T) {
			result := p.ForkVersion(tt.slot)
			assert.Equal(t, tt.expected, result, "expected %s but found %s for slot %d", tt.expected, result, tt.slot)
		})
	}
}

func TestForkVersionEdgeCases(t *testing.T) {
	values := []struct {
		name     string
		slot     uint64
		expected ForkVersion
	}{
		{
			name:     "slot 0 is deneb",
			slot:     0,
			expected: Deneb,
		},
		{
			name:     "exact electra boundary",
			slot:     800000 * 32, // First slot of epoch 800000
			expected: Electra,
		},
		{
			name:     "exact fulu boundary",
			slot:     1000000 * 32, // First slot of epoch 1000000
			expected: Fulu,
		},
	}

	p := Protocol{}
	p.Settings.SlotsInEpoch = 32
	p.Settings.ForkVersions = config.ForkVersions{
		Deneb:   0,
		Electra: 800000,
		Fulu:    1000000,
	}

	for _, tt := range values {
		t.Run(tt.name, func(t *testing.T) {
			result := p.ForkVersion(tt.slot)
			assert.Equal(t, tt.expected, result, "expected %s but found %s for slot %d", tt.expected, result, tt.slot)
		})
	}
}

func TestForkVersionWithDifferentSlotsPerEpoch(t *testing.T) {
	values := []struct {
		name     string
		slot     uint64
		expected ForkVersion
	}{
		{
			name:     "deneb with 8 slots per epoch",
			slot:     0,
			expected: Deneb,
		},
		{
			name:     "electra boundary with 8 slots per epoch",
			slot:     8 * 800000, // epoch 800000 with 8 slots per epoch
			expected: Electra,
		},
		{
			name:     "fulu boundary with 8 slots per epoch",
			slot:     8 * 1000000, // epoch 1000000 with 8 slots per epoch
			expected: Fulu,
		},
	}

	p := Protocol{}
	p.Settings.SlotsInEpoch = 8 // Different from default 32
	p.Settings.ForkVersions = config.ForkVersions{
		Deneb:   0,
		Electra: 800000,
		Fulu:    1000000,
	}

	for _, tt := range values {
		t.Run(tt.name, func(t *testing.T) {
			result := p.ForkVersion(tt.slot)
			assert.Equal(t, tt.expected, result, "expected %s but found %s for slot %d", tt.expected, result, tt.slot)
		})
	}
}
