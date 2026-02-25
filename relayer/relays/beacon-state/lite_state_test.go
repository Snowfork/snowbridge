package beaconstate

import (
	"bytes"
	"encoding/hex"
	"os"
	"testing"

	"github.com/snowfork/snowbridge/relayer/relays/beacon/state"
)

// TestLiteStateMatchesFullState verifies that the lite unmarshaler produces
// the same Merkle tree root as the full unmarshaler.
func TestLiteStateMatchesFullState(t *testing.T) {
	// Load test beacon state
	data, err := os.ReadFile("testdata/beacon_state_sepolia.ssz")
	if err != nil {
		t.Skipf("Skipping test: could not read test data: %v", err)
	}

	t.Logf("Loaded beacon state: %d bytes", len(data))

	// Determine if this is Electra or Deneb based on size
	isElectra := len(data) >= minStateSizeElectra
	t.Logf("State appears to be Electra: %v (size %d, min Electra: %d, min Deneb: %d)",
		isElectra, len(data), minStateSizeElectra, minStateSizeDeneb)

	// Unmarshal with full unmarshaler
	var fullState state.BeaconState
	if isElectra {
		fullState = &state.BeaconStateElectra{}
	} else {
		fullState = &state.BeaconStateDenebMainnet{}
	}
	err = fullState.UnmarshalSSZ(data)
	if err != nil {
		t.Fatalf("Failed to unmarshal full state: %v", err)
	}
	t.Logf("Full state slot: %d", fullState.GetSlot())

	// Unmarshal with lite unmarshaler
	var liteState *LiteBeaconState
	if isElectra {
		liteState, err = UnmarshalSSZLiteElectra(data)
	} else {
		liteState, err = UnmarshalSSZLiteDeneb(data)
	}
	if err != nil {
		t.Fatalf("Failed to unmarshal lite state: %v", err)
	}
	t.Logf("Lite state slot: %d", liteState.GetSlot())

	// Compare slots
	if fullState.GetSlot() != liteState.GetSlot() {
		t.Errorf("Slot mismatch: full=%d, lite=%d", fullState.GetSlot(), liteState.GetSlot())
	}

	// Compare block roots
	fullBlockRoots := fullState.GetBlockRoots()
	liteBlockRoots := liteState.GetBlockRoots()
	if len(fullBlockRoots) != len(liteBlockRoots) {
		t.Errorf("BlockRoots length mismatch: full=%d, lite=%d", len(fullBlockRoots), len(liteBlockRoots))
	} else {
		for i := 0; i < len(fullBlockRoots); i++ {
			if !bytes.Equal(fullBlockRoots[i], liteBlockRoots[i]) {
				t.Errorf("BlockRoots[%d] mismatch", i)
				break
			}
		}
	}

	// Compare finalized checkpoint
	fullCheckpoint := fullState.GetFinalizedCheckpoint()
	liteCheckpoint := liteState.GetFinalizedCheckpoint()
	if fullCheckpoint.Epoch != liteCheckpoint.Epoch {
		t.Errorf("FinalizedCheckpoint.Epoch mismatch: full=%d, lite=%d",
			fullCheckpoint.Epoch, liteCheckpoint.Epoch)
	}
	if !bytes.Equal(fullCheckpoint.Root, liteCheckpoint.Root) {
		t.Errorf("FinalizedCheckpoint.Root mismatch")
	}

	// Get trees from both
	fullTree, err := fullState.GetTree()
	if err != nil {
		t.Fatalf("Failed to get full state tree: %v", err)
	}

	liteTree, err := liteState.GetTree()
	if err != nil {
		t.Fatalf("Failed to get lite state tree: %v", err)
	}

	// Compare tree roots
	fullRoot := fullTree.Hash()
	liteRoot := liteTree.Hash()

	t.Logf("Full state tree root: 0x%s", hex.EncodeToString(fullRoot))
	t.Logf("Lite state tree root: 0x%s", hex.EncodeToString(liteRoot))

	if !bytes.Equal(fullRoot, liteRoot) {
		t.Errorf("Tree root mismatch!\n  Full: 0x%s\n  Lite: 0x%s",
			hex.EncodeToString(fullRoot), hex.EncodeToString(liteRoot))

		// Debug: compare individual field hashes
		debugFieldHashes(t, fullState, liteState, data)
	}
}

// debugFieldHashes helps identify which field is causing the mismatch
func debugFieldHashes(t *testing.T, fullState state.BeaconState, liteState *LiteBeaconState, data []byte) {
	t.Log("Debugging field hashes...")

	// Get full tree and examine each leaf
	fullTree, _ := fullState.GetTree()

	// The tree has leaves at indices 0-27 (or 0-36 for Electra)
	// We can prove each leaf and compare

	// Compare specific field trees that we extract
	fullHeader, _ := fullState.GetLatestBlockHeader().GetTree()
	liteHeader, _ := liteState.GetLatestBlockHeader().GetTree()
	fullHeaderHash := fullHeader.Hash()
	liteHeaderHash := liteHeader.Hash()
	if !bytes.Equal(fullHeaderHash, liteHeaderHash) {
		t.Logf("  Field 4 LatestBlockHeader MISMATCH")
	} else {
		t.Logf("  Field 4 LatestBlockHeader OK: 0x%s", hex.EncodeToString(fullHeaderHash))
	}

	// Compare block roots
	fullBlockRootsContainer := &state.BlockRootsContainerMainnet{}
	fullBlockRootsContainer.SetBlockRoots(fullState.GetBlockRoots())
	fullBlockRootsTree, _ := fullBlockRootsContainer.GetTree()
	liteBlockRootsContainer := &state.BlockRootsContainerMainnet{}
	liteBlockRootsContainer.SetBlockRoots(liteState.GetBlockRoots())
	liteBlockRootsTree, _ := liteBlockRootsContainer.GetTree()
	fullBlockRootsHash := fullBlockRootsTree.Hash()
	liteBlockRootsHash := liteBlockRootsTree.Hash()
	if !bytes.Equal(fullBlockRootsHash, liteBlockRootsHash) {
		t.Logf("  Field 5 BlockRoots MISMATCH: full=0x%s lite=0x%s",
			hex.EncodeToString(fullBlockRootsHash), hex.EncodeToString(liteBlockRootsHash))
	} else {
		t.Logf("  Field 5 BlockRoots OK: 0x%s", hex.EncodeToString(fullBlockRootsHash))
	}

	// Compare FinalizedCheckpoint
	fullFinal, _ := fullState.GetFinalizedCheckpoint().GetTree()
	liteFinal, _ := liteState.GetFinalizedCheckpoint().GetTree()
	fullFinalHash := fullFinal.Hash()
	liteFinalHash := liteFinal.Hash()
	if !bytes.Equal(fullFinalHash, liteFinalHash) {
		t.Logf("  Field 20 FinalizedCheckpoint MISMATCH")
	} else {
		t.Logf("  Field 20 FinalizedCheckpoint OK: 0x%s", hex.EncodeToString(fullFinalHash))
	}

	// Compare sync committees
	fullCurrSync, _ := fullState.GetCurrentSyncCommittee().GetTree()
	liteCurrSync, _ := liteState.GetCurrentSyncCommittee().GetTree()
	fullCurrSyncHash := fullCurrSync.Hash()
	liteCurrSyncHash := liteCurrSync.Hash()
	if !bytes.Equal(fullCurrSyncHash, liteCurrSyncHash) {
		t.Logf("  Field 22 CurrentSyncCommittee MISMATCH")
	} else {
		t.Logf("  Field 22 CurrentSyncCommittee OK: 0x%s", hex.EncodeToString(fullCurrSyncHash))
	}

	fullNextSync, _ := fullState.GetNextSyncCommittee().GetTree()
	liteNextSync, _ := liteState.GetNextSyncCommittee().GetTree()
	fullNextSyncHash := fullNextSync.Hash()
	liteNextSyncHash := liteNextSync.Hash()
	if !bytes.Equal(fullNextSyncHash, liteNextSyncHash) {
		t.Logf("  Field 23 NextSyncCommittee MISMATCH")
	} else {
		t.Logf("  Field 23 NextSyncCommittee OK: 0x%s", hex.EncodeToString(fullNextSyncHash))
	}

	// Now let's check individual leaf values from the full tree
	// by extracting proofs and comparing
	t.Log("Checking tree structure...")

	// Use proof to get leaf at index 5 (BlockRoots) from full tree
	// GeneralizedIndex for field 5 in a 32-leaf container is 32 + 5 = 37
	for fieldIdx := 0; fieldIdx < 28; fieldIdx++ {
		generalizedIndex := 32 + fieldIdx
		proof, err := fullTree.Prove(generalizedIndex)
		if err != nil {
			t.Logf("  Field %d: could not get proof: %v", fieldIdx, err)
			continue
		}
		leafHash := proof.Leaf
		t.Logf("  Field %d leaf hash: 0x%s", fieldIdx, hex.EncodeToString(leafHash))
	}
}

// TestFuluLiteStateMatchesFullState verifies that the lite Fulu unmarshaler produces
// the same Merkle tree root as the full unmarshaler.
func TestFuluLiteStateMatchesFullState(t *testing.T) {
	// Load Fulu test beacon state
	data, err := os.ReadFile("testdata/beacon_state_fulu.ssz")
	if err != nil {
		t.Skipf("Skipping test: could not read Fulu test data: %v", err)
	}

	t.Logf("Loaded Fulu beacon state: %d bytes", len(data))

	// Verify it meets Fulu minimum size
	if len(data) < minStateSizeFulu {
		t.Fatalf("Data too small for Fulu state: %d < %d", len(data), minStateSizeFulu)
	}

	// Unmarshal with full Fulu unmarshaler
	fullState := &state.BeaconStateFulu{}
	err = fullState.UnmarshalSSZ(data)
	if err != nil {
		t.Fatalf("Failed to unmarshal full Fulu state: %v", err)
	}
	t.Logf("Full Fulu state slot: %d", fullState.GetSlot())

	// Unmarshal with lite Fulu unmarshaler
	liteState, err := UnmarshalSSZLiteFulu(data)
	if err != nil {
		t.Fatalf("Failed to unmarshal lite Fulu state: %v", err)
	}
	t.Logf("Lite Fulu state slot: %d", liteState.GetSlot())

	// Compare slots
	if fullState.GetSlot() != liteState.GetSlot() {
		t.Errorf("Slot mismatch: full=%d, lite=%d", fullState.GetSlot(), liteState.GetSlot())
	}

	// Compare block roots
	fullBlockRoots := fullState.GetBlockRoots()
	liteBlockRoots := liteState.GetBlockRoots()
	if len(fullBlockRoots) != len(liteBlockRoots) {
		t.Errorf("BlockRoots length mismatch: full=%d, lite=%d", len(fullBlockRoots), len(liteBlockRoots))
	} else {
		for i := 0; i < len(fullBlockRoots); i++ {
			if !bytes.Equal(fullBlockRoots[i], liteBlockRoots[i]) {
				t.Errorf("BlockRoots[%d] mismatch", i)
				break
			}
		}
	}

	// Compare finalized checkpoint
	fullCheckpoint := fullState.GetFinalizedCheckpoint()
	liteCheckpoint := liteState.GetFinalizedCheckpoint()
	if fullCheckpoint.Epoch != liteCheckpoint.Epoch {
		t.Errorf("FinalizedCheckpoint.Epoch mismatch: full=%d, lite=%d",
			fullCheckpoint.Epoch, liteCheckpoint.Epoch)
	}
	if !bytes.Equal(fullCheckpoint.Root, liteCheckpoint.Root) {
		t.Errorf("FinalizedCheckpoint.Root mismatch")
	}

	// Get trees from both
	fullTree, err := fullState.GetTree()
	if err != nil {
		t.Fatalf("Failed to get full Fulu state tree: %v", err)
	}

	liteTree, err := liteState.GetTree()
	if err != nil {
		t.Fatalf("Failed to get lite Fulu state tree: %v", err)
	}

	// Compare tree roots
	fullRoot := fullTree.Hash()
	liteRoot := liteTree.Hash()

	t.Logf("Full Fulu state tree root: 0x%s", hex.EncodeToString(fullRoot))
	t.Logf("Lite Fulu state tree root: 0x%s", hex.EncodeToString(liteRoot))

	if !bytes.Equal(fullRoot, liteRoot) {
		t.Errorf("Fulu tree root mismatch!\n  Full: 0x%s\n  Lite: 0x%s",
			hex.EncodeToString(fullRoot), hex.EncodeToString(liteRoot))

		// Debug: compare field-by-field
		t.Log("Comparing fields...")
		for fieldIdx := 0; fieldIdx < 40; fieldIdx++ {
			gidx := 64 + fieldIdx // Generalized index for fields in a 64-leaf tree
			fullProof, err1 := fullTree.Prove(gidx)
			liteProof, err2 := liteTree.Prove(gidx)
			if err1 != nil || err2 != nil {
				continue
			}
			if !bytes.Equal(fullProof.Leaf, liteProof.Leaf) {
				t.Logf("  Field %d MISMATCH: full=0x%s lite=0x%s",
					fieldIdx,
					hex.EncodeToString(fullProof.Leaf),
					hex.EncodeToString(liteProof.Leaf))
			}
		}
	}
}

// TestBlockRootsTreeHash specifically tests the block roots tree hash
func TestBlockRootsTreeHash(t *testing.T) {
	data, err := os.ReadFile("testdata/beacon_state_sepolia.ssz")
	if err != nil {
		t.Skipf("Skipping test: could not read test data: %v", err)
	}

	// Get full state
	isElectra := len(data) >= minStateSizeElectra
	var fullState state.BeaconState
	if isElectra {
		fullState = &state.BeaconStateElectra{}
	} else {
		fullState = &state.BeaconStateDenebMainnet{}
	}
	err = fullState.UnmarshalSSZ(data)
	if err != nil {
		t.Fatalf("Failed to unmarshal full state: %v", err)
	}

	// Get lite state
	var liteState *LiteBeaconState
	if isElectra {
		liteState, err = UnmarshalSSZLiteElectra(data)
	} else {
		liteState, err = UnmarshalSSZLiteDeneb(data)
	}
	if err != nil {
		t.Fatalf("Failed to unmarshal lite state: %v", err)
	}

	// Build block roots tree from full state
	fullBlockRoots := fullState.GetBlockRoots()
	fullBlockRootsContainer := &state.BlockRootsContainerMainnet{}
	fullBlockRootsContainer.SetBlockRoots(fullBlockRoots)
	fullBlockRootsTree, err := fullBlockRootsContainer.GetTree()
	if err != nil {
		t.Fatalf("Failed to get full block roots tree: %v", err)
	}
	fullBlockRootsHash := fullBlockRootsTree.Hash()

	// Build block roots tree from lite state using the same container
	liteBlockRootsContainer := &state.BlockRootsContainerMainnet{}
	liteBlockRootsContainer.SetBlockRoots(liteState.GetBlockRoots())
	liteBlockRootsTree, err := liteBlockRootsContainer.GetTree()
	if err != nil {
		t.Fatalf("Failed to build lite block roots tree: %v", err)
	}
	liteBlockRootsHash := liteBlockRootsTree.Hash()

	t.Logf("Full block roots hash: 0x%s", hex.EncodeToString(fullBlockRootsHash))
	t.Logf("Lite block roots hash: 0x%s", hex.EncodeToString(liteBlockRootsHash))

	if !bytes.Equal(fullBlockRootsHash, liteBlockRootsHash) {
		t.Errorf("Block roots tree hash mismatch!")
	}
}
