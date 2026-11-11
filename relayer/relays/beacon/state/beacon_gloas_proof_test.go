package state

import (
	"testing"

	"github.com/pk910/dynamic-ssz/treeproof"
)

// The four generalized indices the on-chain light client hardcodes for Gloas.
const (
	gindexFinalizedRoot        = 735
	gindexCurrentSyncCommittee = 2945
	gindexNextSyncCommittee    = 2946
	gindexBlockRoots           = 352
)

func TestGloasProofBranchesMatchPalletIndices(t *testing.T) {
	raw, wantRoot := fetchGloasState(t)

	var s BeaconStateGloas
	if err := s.UnmarshalSSZ(raw); err != nil {
		t.Fatalf("decode: %v", err)
	}
	root, err := s.HashTreeRoot()
	if err != nil {
		t.Fatalf("hash tree root: %v", err)
	}
	if "0x"+hexOf(root) != wantRoot {
		t.Fatalf("HashTreeRoot disagrees with node\n got  0x%s\n want %s", hexOf(root), wantRoot)
	}

	tree, err := s.GloasProofTree()
	if err != nil {
		t.Fatalf("proof tree: %v", err)
	}

	for _, tc := range []struct {
		name   string
		gindex int
	}{
		{"current_sync_committee", gindexCurrentSyncCommittee},
		{"next_sync_committee", gindexNextSyncCommittee},
		{"block_roots", gindexBlockRoots},
		{"finalized_root", gindexFinalizedRoot},
	} {
		t.Run(tc.name, func(t *testing.T) {
			proof, err := tree.Prove(tc.gindex)
			if err != nil {
				t.Fatalf("prove: %v", err)
			}
			ok, err := treeproof.VerifyProof(root[:], proof)
			if err != nil {
				t.Fatalf("verify: %v", err)
			}
			if !ok {
				t.Fatalf("branch at gindex %d does not verify (depth %d)", tc.gindex, len(proof.Hashes))
			}
			t.Logf("gindex %d verified, depth %d", tc.gindex, len(proof.Hashes))
		})
	}
}

func hexOf(b [32]byte) string {
	const hexdigits = "0123456789abcdef"
	out := make([]byte, 64)
	for i, v := range b {
		out[i*2] = hexdigits[v>>4]
		out[i*2+1] = hexdigits[v&0xf]
	}
	return string(out)
}
