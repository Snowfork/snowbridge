package beaconstate

import (
	"bytes"
	"encoding/binary"
	"encoding/hex"
	"os"
	"testing"

	ssz "github.com/ferranbt/fastssz"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/state"
)

// TestHashFunctionsMatchSSZ verifies that our hash functions produce the same
// results as fastssz's HashTreeRoot.
func TestHashFunctionsMatchSSZ(t *testing.T) {
	data, err := os.ReadFile("testdata/beacon_state_sepolia.ssz")
	if err != nil {
		t.Skipf("Skipping test: could not read test data: %v", err)
	}

	// Unmarshal full state
	fullState := &state.BeaconStateElectra{}
	err = fullState.UnmarshalSSZ(data)
	if err != nil {
		t.Fatalf("Failed to unmarshal full state: %v", err)
	}

	// Test StateRoots hash
	t.Run("StateRoots", func(t *testing.T) {
		// Get expected hash from full state using HashTreeRoot
		stateRootsContainer := &stateRootsVector{roots: fullState.StateRoots}
		expectedHash, err := stateRootsContainer.HashTreeRoot()
		if err != nil {
			t.Fatalf("Failed to get state roots hash: %v", err)
		}

		// Get actual hash from our function
		stateRootsData := data[offsetStateRoots:offsetStateRootsEnd]
		actualHash := hashFixedVector(stateRootsData, 32, 8192)

		t.Logf("Expected: 0x%s", hex.EncodeToString(expectedHash[:]))
		t.Logf("Actual:   0x%s", hex.EncodeToString(actualHash[:]))

		if !bytes.Equal(expectedHash[:], actualHash[:]) {
			t.Error("StateRoots hash mismatch")
		}
	})

	// Test Validators hash
	t.Run("Validators", func(t *testing.T) {
		// Get expected hash using full state
		validatorsContainer := &validatorsVector{validators: fullState.Validators}
		expectedHash, err := validatorsContainer.HashTreeRoot()
		if err != nil {
			t.Fatalf("Failed to get validators hash: %v", err)
		}

		// Get validators data from raw bytes
		o11 := binary.LittleEndian.Uint32(data[offsetValidatorsPtr:])
		o12 := binary.LittleEndian.Uint32(data[offsetBalancesPtr:])
		validatorsData := data[o11:o12]
		actualHash := hashValidators(validatorsData)

		t.Logf("Expected: 0x%s", hex.EncodeToString(expectedHash[:]))
		t.Logf("Actual:   0x%s", hex.EncodeToString(actualHash[:]))
		t.Logf("Num validators: %d", len(fullState.Validators))

		if !bytes.Equal(expectedHash[:], actualHash[:]) {
			t.Error("Validators hash mismatch")
		}
	})

	// Test Balances hash
	t.Run("Balances", func(t *testing.T) {
		balancesContainer := &balancesVector{balances: fullState.Balances}
		expectedHash, err := balancesContainer.HashTreeRoot()
		if err != nil {
			t.Fatalf("Failed to get balances hash: %v", err)
		}

		o12 := binary.LittleEndian.Uint32(data[offsetBalancesPtr:])
		o15 := binary.LittleEndian.Uint32(data[offsetPrevEpochPartPtr:])
		balancesData := data[o12:o15]
		actualHash := hashBalances(balancesData)

		t.Logf("Expected: 0x%s", hex.EncodeToString(expectedHash[:]))
		t.Logf("Actual:   0x%s", hex.EncodeToString(actualHash[:]))

		if !bytes.Equal(expectedHash[:], actualHash[:]) {
			t.Error("Balances hash mismatch")
		}
	})
}

// Helper types to compute hash tree roots for verification

type stateRootsVector struct {
	roots [][]byte
}

func (s *stateRootsVector) HashTreeRoot() ([32]byte, error) {
	hh := ssz.DefaultHasherPool.Get()
	if err := s.HashTreeRootWith(hh); err != nil {
		ssz.DefaultHasherPool.Put(hh)
		return [32]byte{}, err
	}
	root, err := hh.HashRoot()
	ssz.DefaultHasherPool.Put(hh)
	return root, err
}

func (s *stateRootsVector) HashTreeRootWith(hh ssz.HashWalker) error {
	subIndx := hh.Index()
	for _, root := range s.roots {
		hh.Append(root)
	}
	hh.Merkleize(subIndx)
	return nil
}

type validatorsVector struct {
	validators []*state.Validator
}

func (v *validatorsVector) HashTreeRoot() ([32]byte, error) {
	hh := ssz.DefaultHasherPool.Get()
	if err := v.HashTreeRootWith(hh); err != nil {
		ssz.DefaultHasherPool.Put(hh)
		return [32]byte{}, err
	}
	root, err := hh.HashRoot()
	ssz.DefaultHasherPool.Put(hh)
	return root, err
}

func (v *validatorsVector) HashTreeRootWith(hh ssz.HashWalker) error {
	subIndx := hh.Index()
	for _, val := range v.validators {
		if err := val.HashTreeRootWith(hh); err != nil {
			return err
		}
	}
	hh.MerkleizeWithMixin(subIndx, uint64(len(v.validators)), 1099511627776)
	return nil
}

type balancesVector struct {
	balances []uint64
}

func (b *balancesVector) HashTreeRoot() ([32]byte, error) {
	hh := ssz.DefaultHasherPool.Get()
	if err := b.HashTreeRootWith(hh); err != nil {
		ssz.DefaultHasherPool.Put(hh)
		return [32]byte{}, err
	}
	root, err := hh.HashRoot()
	ssz.DefaultHasherPool.Put(hh)
	return root, err
}

func (b *balancesVector) HashTreeRootWith(hh ssz.HashWalker) error {
	subIndx := hh.Index()
	for _, bal := range b.balances {
		hh.AppendUint64(bal)
	}
	hh.FillUpTo32()
	numItems := uint64(len(b.balances))
	hh.MerkleizeWithMixin(subIndx, numItems, ssz.CalculateLimit(1099511627776, numItems, 8))
	return nil
}
