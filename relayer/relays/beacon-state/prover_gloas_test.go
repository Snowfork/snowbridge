package beaconstate

import (
	"os"
	"path/filepath"
	"testing"

	"github.com/pk910/dynamic-ssz/treeproof"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/state"
)

// The prover the service will actually use must produce branches that verify at the
// generalized indices the on-chain light client hardcodes. Uses the state cached by the
// state package's oracle test; skips when it is absent.
func TestGloasProverProducesVerifiableBranches(t *testing.T) {
	path := filepath.Join("..", "beacon", "state", "testdata", "gloas_state.ssz")
	raw, err := os.ReadFile(path)
	if err != nil {
		t.Skipf("no cached Gloas state (run the state package tests first): %v", err)
	}

	gloas := &state.BeaconStateGloas{}
	if err := gloas.UnmarshalSSZ(raw); err != nil {
		t.Fatalf("decode: %v", err)
	}
	p, err := newProver(gloas)
	if err != nil {
		t.Fatalf("newProver: %v", err)
	}
	if _, ok := p.(dynsszProver); !ok {
		t.Fatalf("expected a dynssz prover for a Gloas state, got %T", p)
	}
	root, err := gloas.HashTreeRoot()
	if err != nil {
		t.Fatalf("state root: %v", err)
	}

	for _, tc := range []struct {
		name   string
		gindex int
		depth  int
	}{
		{"current_sync_committee", 2945, 11},
		{"next_sync_committee", 2946, 11},
		{"block_roots", 352, 8},
		{"finalized_root", 735, 9},
	} {
		t.Run(tc.name, func(t *testing.T) {
			leaf, hashes, err := p.prove(tc.gindex)
			if err != nil {
				t.Fatalf("prove: %v", err)
			}
			if len(hashes) != tc.depth {
				t.Fatalf("depth %d, expected %d", len(hashes), tc.depth)
			}
			ok, err := treeproof.VerifyProof(root[:], &treeproof.Proof{
				Index:  tc.gindex,
				Leaf:   leaf,
				Hashes: hashes,
			})
			if err != nil {
				t.Fatalf("verify: %v", err)
			}
			if !ok {
				t.Fatalf("branch at gindex %d does not verify against the state root", tc.gindex)
			}
		})
	}
}

// A pre-Gloas state must still go down the fastssz path.
func TestNonGloasStateStillUsesFastssz(t *testing.T) {
	p, err := newProver(&state.BeaconStateFulu{})
	if err == nil {
		if _, ok := p.(fastsszProver); !ok {
			t.Fatalf("expected a fastssz prover for a Fulu state, got %T", p)
		}
		return
	}
	if _, ok := p.(dynsszProver); ok {
		t.Fatal("a Fulu state must never get the dynssz prover")
	}
}
