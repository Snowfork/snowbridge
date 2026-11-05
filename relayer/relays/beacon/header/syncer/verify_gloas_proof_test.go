package syncer

import (
	"testing"

	"github.com/pk910/dynamic-ssz/treeproof"
)

func verifyGloasExecProof(t *testing.T, leaf [32]byte, branch [][32]byte, gindex uint64, root [32]byte) bool {
	t.Helper()
	hashes := make([][]byte, len(branch))
	for i := range branch {
		h := branch[i]
		hashes[i] = h[:]
	}
	ok, err := treeproof.VerifyProof(root[:], &treeproof.Proof{
		Index:  int(gindex),
		Leaf:   leaf[:],
		Hashes: hashes,
	})
	if err != nil {
		t.Fatalf("verify: %v", err)
	}
	return ok
}
