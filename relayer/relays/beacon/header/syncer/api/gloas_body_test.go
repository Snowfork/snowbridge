package api

import (
	"encoding/hex"
	"encoding/json"
	"errors"
	"os"
	"testing"

	"github.com/snowfork/snowbridge/relayer/relays/beacon/state"
)

// Slot 115968 of the Platåberget Gloas testnet, fetched from a Prysm node. Its body root is
// taken from the node's own block header, so this asserts our merkleization against the
// network's, across every field the body has.
const (
	blockFixture         = "testdata/block_gloas_115968.json"
	blockBodyRoot        = "3b72fb6996412d59a20d92dde86abe3a12b94232ba62a61ac301625f1f8950f0"
	blockParentBlockHash = "9af0c93804597fe505c22f3b15abb47acb01f7c6106474b196b52f592fae0e39"
)

func loadBlockFixture(t *testing.T) *BeaconBlockResponse {
	t.Helper()
	raw, err := os.ReadFile(blockFixture)
	if err != nil {
		t.Fatalf("read fixture: %v", err)
	}
	var resp BeaconBlockResponse
	if err := json.Unmarshal(raw, &resp); err != nil {
		t.Fatalf("unmarshal fixture: %v", err)
	}
	return &resp
}

// The whole point of the exercise: a body assembled from what a beacon node serves must
// merkleize to the root that node published.
func TestBuildGloasBodyReproducesRealBodyRoot(t *testing.T) {
	resp := loadBlockFixture(t)
	body, err := BuildGloasBody(&resp.Data.Message.Body)
	if err != nil {
		t.Fatalf("build body: %v", err)
	}
	root, err := body.HashTreeRoot()
	if err != nil {
		t.Fatalf("body root: %v", err)
	}
	if got := hex.EncodeToString(root[:]); got != blockBodyRoot {
		t.Fatalf("body root mismatch\n got  %s\n want %s", got, blockBodyRoot)
	}
}

// The execution commitment proof must verify against that same body root, since the light
// client is given nothing else to check it with.
func TestBuildGloasBodyExecutionProofVerifies(t *testing.T) {
	resp := loadBlockFixture(t)
	body, err := BuildGloasBody(&resp.Data.Message.Body)
	if err != nil {
		t.Fatalf("build body: %v", err)
	}
	root, err := body.HashTreeRoot()
	if err != nil {
		t.Fatalf("body root: %v", err)
	}
	leaf, branch, err := body.ExecutionProof()
	if err != nil {
		t.Fatalf("execution proof: %v", err)
	}
	if got := hex.EncodeToString(leaf[:]); got != blockParentBlockHash {
		t.Fatalf("leaf mismatch\n got  %s\n want %s", got, blockParentBlockHash)
	}
	if !verifyGloasExecProof(t, leaf, branch, state.GloasExecutionCommitmentGindex, root) {
		t.Fatal("execution proof does not verify against the body root")
	}
}

// A block carrying an operation we cannot merkleize must fail, not produce a wrong root.
func TestBuildGloasBodyRefusesUnsupportedOperations(t *testing.T) {
	resp := loadBlockFixture(t)
	resp.Data.Message.Body.Deposits = make([]DepositResponse, 1)
	_, err := BuildGloasBody(&resp.Data.Message.Body)
	var unsupported state.ErrUnsupportedBodyList
	if !errors.As(err, &unsupported) {
		t.Fatalf("expected ErrUnsupportedBodyList, got %v", err)
	}
}

// A pre-Gloas block has no bid and must be rejected rather than silently mis-built.
func TestBuildGloasBodyRejectsBlockWithoutBid(t *testing.T) {
	resp := loadBlockFixture(t)
	resp.Data.Message.Body.SignedExecutionPayloadBid = nil
	if _, err := BuildGloasBody(&resp.Data.Message.Body); err == nil {
		t.Fatal("expected an error for a block with no bid")
	}
}
