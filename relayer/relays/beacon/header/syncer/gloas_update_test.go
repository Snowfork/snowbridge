package syncer

import (
	"encoding/hex"
	"encoding/json"
	"os"
	"testing"

	"github.com/ethereum/go-ethereum/common"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/execution"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/api"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/state"
)

// The two halves of a Gloas execution proof are built from different sources: the leaf is
// merkleized out of the beacon block, and the bytes submitted alongside it come from an
// execution node. The proof is only sound if they meet, and nothing in either package checks
// that on its own. Both fixtures are slot 115968 of the Platåberget Gloas testnet.
func TestGloasProofLeafIsTheSubmittedExecutionHeader(t *testing.T) {
	blockJSON, err := os.ReadFile("api/testdata/block_gloas_115968.json")
	if err != nil {
		t.Fatalf("read block fixture: %v", err)
	}
	var block api.BeaconBlockResponse
	if err := json.Unmarshal(blockJSON, &block); err != nil {
		t.Fatalf("unmarshal block fixture: %v", err)
	}

	body, err := api.BuildGloasBody(&block.Data.Message.Body)
	if err != nil {
		t.Fatalf("build body: %v", err)
	}
	bodyRoot, err := body.HashTreeRoot()
	if err != nil {
		t.Fatalf("body root: %v", err)
	}
	leaf, branch, err := body.ExecutionProof()
	if err != nil {
		t.Fatalf("execution proof: %v", err)
	}
	if !verifyGloasExecProof(t, leaf, branch, state.GloasExecutionCommitmentGindex, bodyRoot) {
		t.Fatal("execution branch does not verify against the body root")
	}

	headerJSON, err := os.ReadFile("../../execution/testdata/header_gloas_113202.json")
	if err != nil {
		t.Fatalf("read header fixture: %v", err)
	}
	var fields map[string]string
	if err := json.Unmarshal(headerJSON, &fields); err != nil {
		t.Fatalf("unmarshal header fixture: %v", err)
	}

	// EncodeHeaderRLP checks the encoding against the hash it is given, so passing the proven
	// leaf asserts that the header the relayer submits is the one the beacon block commits to.
	rlp, err := execution.EncodeHeaderRLP(fields, common.BytesToHash(leaf[:]))
	if err != nil {
		t.Fatalf("the proven leaf and the execution header disagree: %v", err)
	}
	t.Logf("proved %s and submitted %d bytes of header rlp", hex.EncodeToString(leaf[:]), len(rlp))
}
