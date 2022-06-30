package syncer

import (
	"fmt"
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestComputeEpochAtSlot(t *testing.T) {
	values := []struct {
		name     string
		slot     uint64
		expected uint64
	}{
		{
			name:     "valid",
			slot:     3433200,
			expected: 107287,
		},
		{
			name:     "valid",
			slot:     400,
			expected: 12,
		},
		{
			name:     "0",
			slot:     0,
			expected: 0,
		},
	}

	for _, tt := range values {
		total := computeEpochAtSlot(tt.slot)
		if total != tt.expected {
			t.Errorf("ComputeEpochAtSlot of slot (%d) was incorrect, got: %d, want: %d.", tt.slot, total, tt.expected)
		}
	}
}

func TestComputeEpochForNextPeriod(t *testing.T) {
	values := []struct {
		name     string
		epoch    uint64
		expected uint64
	}{
		{
			name:     "first epoch",
			epoch:    0,
			expected: 256,
		},
		{
			name:     "another epoch",
			epoch:    30,
			expected: 256,
		},
		{
			name:     "another epoch",
			epoch:    513,
			expected: 768,
		},
	}

	for _, tt := range values {
		total := computeEpochForNextPeriod(tt.epoch)
		if total != tt.expected {
			t.Errorf("TestComputeEpochForNextPeriod of epoch (%d) was incorrect, got: %d, want: %d.", tt.epoch, total, tt.expected)
		}
	}
}

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
		name string
		hex  string
	}{
		{
			name: "committee bits hex",
			hex:  "0xedfdbdffbffbffffffffdffffffff7ff7feffff7fffffffffbff7dfafdefffffdffbffaffffffeffffffeefbf6dffffffffffffffffffeffdfff7ffffff7fdff",
		},
		{
			name: "aggregation bits",
			hex:  "0x0000000000000000000000000000000104",
		},
	}

	for _, tt := range values {
		result, err := hexStringToByteArray(tt.hex)
		fmt.Println(result)
		assert.NoError(t, err)
		assert.NotEmpty(t, result)
	}
}
