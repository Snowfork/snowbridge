package syncer

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestHexToBinaryString(t *testing.T) {
	values := []struct {
		name     string
		hex      string
		expected string
	}{
		{
			name:     "committee bits",
			hex:      "0xfbfefffffffdffffffffffeffffffffffffffffffffffbfffffffffffffffffffffffffffffffffffdfffffffeffffffffffffffffffffffffffffffffffffff",
			expected: "11111011111111101111111111111111111111111111110111111111111111111111111111111111111111111110111111111111111111111111111111111111111111111111111111111111111111111111111111111111111110111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111011111111111111111111111111111111011111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111",
		},
		{
			name:     "other",
			hex:      "fbfef",
			expected: "11111011111111101111",
		},
		{
			name:     "other",
			hex:      "fbfefffdef",
			expected: "1111101111111110111111111111110111101111",
		},
	}

	for _, tt := range values {
		result := hexToBinaryString(tt.hex)
		if result != tt.expected {
			t.Errorf("HexToBinaryString was incorrect, got: %s, want: %s", result, tt.expected)
		}
	}
}

func TestHexToBytes(t *testing.T) {
	values := []struct {
		name     string
		hex      string
		expected []byte
	}{
		{
			name:     "committee bits hex",
			hex:      "0xedfdbdffbffbffffffffdffffffff7ff7feffff7fffffffffbff7dfafdefffffdffbffaffffffeffffffeefbf6dffffffffffffffffffeffdfff7ffffff7fdff",
			expected: []byte{0xed, 0xfd, 0xbd, 0xff, 0xbf, 0xfb, 0xff, 0xff, 0xff, 0xff, 0xdf, 0xff, 0xff, 0xff, 0xf7, 0xff, 0x7f, 0xef, 0xff, 0xf7, 0xff, 0xff, 0xff, 0xff, 0xfb, 0xff, 0x7d, 0xfa, 0xfd, 0xef, 0xff, 0xff, 0xdf, 0xfb, 0xff, 0xaf, 0xff, 0xff, 0xfe, 0xff, 0xff, 0xff, 0xee, 0xfb, 0xf6, 0xdf, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe, 0xff, 0xdf, 0xff, 0x7f, 0xff, 0xff, 0xf7, 0xfd, 0xff},
		},
		{
			name:     "aggregation bits",
			hex:      "0x0000000000000000000000000000000104",
			expected: []byte{0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1, 0x4},
		},
	}

	for _, tt := range values {
		result, err := hexStringToByteArray(tt.hex)
		assert.NoError(t, err)
		assert.NotEmpty(t, result)
		assert.Equal(t, tt.expected, result)
	}
}

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

	syncer := Syncer{}
	syncer.SlotsInEpoch = 32

	for _, tt := range values {
		result := syncer.IsStartOfEpoch(tt.slot)
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

	syncer := Syncer{}
	syncer.SlotsInEpoch = 8
	syncer.EpochsPerSyncCommitteePeriod = 8

	for _, tt := range values {
		result := syncer.CalculateNextCheckpointSlot(tt.slot)
		assert.Equal(t, tt.expected, result, "expected %t but found %t for slot %d", tt.expected, result, tt.slot)
	}
}
