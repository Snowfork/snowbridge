package protocol

import (
	"errors"
	"testing"

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
