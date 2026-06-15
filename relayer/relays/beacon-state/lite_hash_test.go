package beaconstate

import (
	"bytes"
	"encoding/binary"
	"testing"
)

func TestMerkleize(t *testing.T) {
	// Test empty leaves
	emptyResult := merkleize(nil)
	if emptyResult != [32]byte{} {
		t.Error("merkleize of empty leaves should return zero hash")
	}

	// Test single leaf
	leaf := [32]byte{1, 2, 3, 4, 5}
	singleResult := merkleize([][32]byte{leaf})
	if singleResult != leaf {
		t.Error("merkleize of single leaf should return the leaf")
	}

	// Test two leaves
	leaf1 := [32]byte{1}
	leaf2 := [32]byte{2}
	twoResult := merkleize([][32]byte{leaf1, leaf2})
	expected := hashTwo(leaf1, leaf2)
	if twoResult != expected {
		t.Errorf("merkleize of two leaves = %x, want %x", twoResult, expected)
	}
}

func TestMixInLength(t *testing.T) {
	root := [32]byte{1, 2, 3, 4}
	length := uint64(42)
	result := mixInLength(root, length)

	// The result should be hash(root || length_as_le_bytes)
	var lengthLeaf [32]byte
	binary.LittleEndian.PutUint64(lengthLeaf[:8], length)
	expected := hashTwo(root, lengthLeaf)

	if result != expected {
		t.Errorf("mixInLength = %x, want %x", result, expected)
	}
}

func TestHashBalances(t *testing.T) {
	// Test empty balances - should produce a valid hash (not zero)
	emptyResult := hashBalances(nil)
	// Empty list has a valid SSZ hash (zero hash at depth mixed with length 0)
	if emptyResult == [32]byte{} {
		t.Error("hashBalances of empty should return non-zero SSZ hash")
	}

	// Test single balance
	balance := uint64(32000000000) // 32 ETH in gwei
	data := make([]byte, 8)
	binary.LittleEndian.PutUint64(data, balance)
	result := hashBalances(data)

	// Should produce a different hash than empty
	if result == emptyResult {
		t.Error("hashBalances with data should differ from empty hash")
	}
}

func TestHashParticipation(t *testing.T) {
	// Test empty participation - should produce a valid hash (not zero)
	emptyResult := hashParticipation(nil)
	// Empty list has a valid SSZ hash (zero hash at depth mixed with length 0)
	if emptyResult == [32]byte{} {
		t.Error("hashParticipation of empty should return non-zero SSZ hash")
	}

	// Test participation with data
	data := make([]byte, 100)
	for i := range data {
		data[i] = byte(i % 256)
	}
	result := hashParticipation(data)
	// Should produce a different hash than empty
	if result == emptyResult {
		t.Error("hashParticipation with data should differ from empty hash")
	}
}

func TestNextPowerOfTwo(t *testing.T) {
	tests := []struct {
		input    int
		expected int
	}{
		{0, 1},
		{1, 1},
		{2, 2},
		{3, 4},
		{4, 4},
		{5, 8},
		{7, 8},
		{8, 8},
		{9, 16},
		{28, 32},
		{37, 64},
	}

	for _, tt := range tests {
		result := nextPowerOfTwo(tt.input)
		if result != tt.expected {
			t.Errorf("nextPowerOfTwo(%d) = %d, want %d", tt.input, result, tt.expected)
		}
	}
}

func TestUint64ToLeaf(t *testing.T) {
	val := uint64(0x0102030405060708)
	leaf := uint64ToLeaf(val)

	// Check little-endian encoding in first 8 bytes
	got := binary.LittleEndian.Uint64(leaf[:8])
	if got != val {
		t.Errorf("uint64ToLeaf first 8 bytes = %x, want %x", got, val)
	}

	// Check rest is zeros
	for i := 8; i < 32; i++ {
		if leaf[i] != 0 {
			t.Errorf("uint64ToLeaf byte %d = %d, want 0", i, leaf[i])
		}
	}
}

func TestHashValidator(t *testing.T) {
	// Create a mock 121-byte validator
	validatorData := make([]byte, 121)
	// pubkey (48 bytes)
	for i := 0; i < 48; i++ {
		validatorData[i] = byte(i)
	}
	// withdrawal_credentials (32 bytes)
	for i := 48; i < 80; i++ {
		validatorData[i] = byte(i)
	}
	// effective_balance (8 bytes)
	binary.LittleEndian.PutUint64(validatorData[80:88], 32000000000)
	// slashed (1 byte)
	validatorData[88] = 0
	// epochs (4 x 8 bytes)
	binary.LittleEndian.PutUint64(validatorData[89:97], 100)
	binary.LittleEndian.PutUint64(validatorData[97:105], 200)
	binary.LittleEndian.PutUint64(validatorData[105:113], 18446744073709551615) // FAR_FUTURE_EPOCH
	binary.LittleEndian.PutUint64(validatorData[113:121], 18446744073709551615)

	result := hashValidator(validatorData)

	// Should produce a non-zero hash
	if result == [32]byte{} {
		t.Error("hashValidator should not return zero hash")
	}

	// Same input should produce same output
	result2 := hashValidator(validatorData)
	if !bytes.Equal(result[:], result2[:]) {
		t.Error("hashValidator should be deterministic")
	}
}
