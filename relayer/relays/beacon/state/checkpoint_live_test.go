package state

import (
	"encoding/hex"
	"encoding/json"
	"os"
	"testing"

)

// Verifies a checkpoint the relayer actually produced against the beacon state root it names,
// at the generalized indices the on-chain light client uses for Gloas. This is the same check
// the pallet performs on force_checkpoint, so it catches a disagreement between how the relay
// merkleizes a sync committee and how the runtime does, without needing a chain.
//
// Skips unless SNOWBRIDGE_CHECKPOINT points at a generated checkpoint.
func TestLiveGloasCheckpointBranchesVerify(t *testing.T) {
	path := os.Getenv("SNOWBRIDGE_CHECKPOINT")
	if path == "" {
		t.Skip("set SNOWBRIDGE_CHECKPOINT to a generated checkpoint json")
	}
	raw, err := os.ReadFile(path)
	if err != nil {
		t.Fatalf("read checkpoint: %v", err)
	}
	var cp struct {
		Header struct {
			Slot      uint64 `json:"slot"`
			StateRoot string `json:"state_root"`
		} `json:"header"`
		CurrentSyncCommittee struct {
			Pubkeys         []string `json:"pubkeys"`
			AggregatePubkey string   `json:"aggregate_pubkey"`
		} `json:"current_sync_committee"`
		CurrentSyncCommitteeBranch []string `json:"current_sync_committee_branch"`
		BlockRootsRoot             string   `json:"block_roots_root"`
		BlockRootsBranch           []string `json:"block_roots_branch"`
	}
	if err := json.Unmarshal(raw, &cp); err != nil {
		t.Fatalf("decode checkpoint: %v", err)
	}

	stateRoot := mustHash(t, cp.Header.StateRoot)
	slot := cp.Header.Slot

	committee := &SyncCommittee{
		PubKeys:         make([][]byte, len(cp.CurrentSyncCommittee.Pubkeys)),
		AggregatePubKey: to48(mustBytes(t, cp.CurrentSyncCommittee.AggregatePubkey)),
	}
	for i, pk := range cp.CurrentSyncCommittee.Pubkeys {
		committee.PubKeys[i] = mustBytes(t, pk)
	}

	const (
		gindexCurrentSyncCommittee = 2945
		gindexBlockRoots           = 352
	)
	if !verifyProof(t, committee.hashTreeRoot(), hashes(t, cp.CurrentSyncCommitteeBranch), gindexCurrentSyncCommittee, stateRoot) {
		t.Fatal("current_sync_committee branch does not verify against the state root")
	}
	if !verifyProof(t, mustHash(t, cp.BlockRootsRoot), hashes(t, cp.BlockRootsBranch), gindexBlockRoots, stateRoot) {
		t.Fatal("block_roots branch does not verify against the state root")
	}
	t.Logf("checkpoint at slot %d verifies: %d pubkeys, branches %d and %d deep",
		slot, len(committee.PubKeys), len(cp.CurrentSyncCommitteeBranch), len(cp.BlockRootsBranch))
}

func mustBytes(t *testing.T, s string) []byte {
	t.Helper()
	if len(s) > 1 && s[0] == '0' && (s[1] == 'x' || s[1] == 'X') {
		s = s[2:]
	}
	b, err := hex.DecodeString(s)
	if err != nil {
		t.Fatalf("decode %q: %v", s, err)
	}
	return b
}

func mustHash(t *testing.T, s string) [32]byte {
	t.Helper()
	var out [32]byte
	copy(out[:], mustBytes(t, s))
	return out
}

func to48(b []byte) [48]byte {
	var out [48]byte
	copy(out[:], b)
	return out
}

func hashes(t *testing.T, list []string) [][32]byte {
	t.Helper()
	out := make([][32]byte, len(list))
	for i, h := range list {
		out[i] = mustHash(t, h)
	}
	return out
}
