package syncer

import (
	"fmt"
	"testing"
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
		total := ComputeEpochAtSlot(tt.slot)
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
		total := ComputeEpochForNextPeriod(tt.epoch)
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
		result := HexToBinaryString(tt.hex)
		if result != tt.expected {
			t.Errorf("HexToBinaryString was incorrect, got: %s, want: %s", result, tt.expected)
		}
	}
}

func TestProofs(t *testing.T) {
	syncer := New("http://localhost:9596")

	mew, err := syncer.GetFinalizedCheckpointProofs("0xe1c879117085b9dfe94243b22fa1944e2ddedc22cbb7b166affb8c2576b8fc30")
	if err != nil {
		t.Errorf("unable to get proofs")
	}

	fmt.Printf("%v", mew)
}
