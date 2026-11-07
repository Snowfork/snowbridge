package state

import (
	"testing"
)

// The execution proof must verify at the generalized index the pallet hardcodes, at the depth
// it derives from it. A wrong composition fails here rather than on chain.
func TestGloasExecutionProofVerifiesAt2856(t *testing.T) {
	body := &BeaconBlockBodyGloas{
		RandaoReveal: make([]byte, 96),
		Eth1Data:     &Eth1Data{DepositRoot: make([]byte, 32), BlockHash: make([]byte, 32)},
		Graffiti:     make([]byte, 32),
		SignedExecutionPayloadBid: &SignedExecutionPayloadBidGloas{
			Signature: make([]byte, 96),
			Message: &ExecutionPayloadBid{
				ParentBlockHash:       []byte{0xaa, 0xbb, 0xcc},
				ParentBlockRoot:       make([]byte, 32),
				BlockHash:             make([]byte, 32),
				PrevRandao:            make([]byte, 32),
				FeeRecipient:          make([]byte, 20),
				ExecutionRequestsRoot: make([]byte, 32),
				BlobKzgCommitments:    [][]byte{make([]byte, 48), make([]byte, 48)},
			},
		},
	}

	root, err := body.HashTreeRoot()
	if err != nil {
		t.Fatalf("body root: %v", err)
	}
	leaf, branch, err := body.ExecutionProof()
	if err != nil {
		t.Fatalf("execution proof: %v", err)
	}
	if leaf != bytesRoot(body.SignedExecutionPayloadBid.Message.ParentBlockHash) {
		t.Fatal("leaf is not the bid's parent_block_hash")
	}
	// generalized_index_length(2856) == 11
	if len(branch) != 11 {
		t.Fatalf("branch depth %d, expected 11", len(branch))
	}
	if !verifyProof(t, leaf, branch, GloasExecutionCommitmentGindex, root) {
		t.Fatalf("branch does not verify at gindex %d", GloasExecutionCommitmentGindex)
	}
}

// Changing the committed hash must break the proof: that is what stops a submitter pairing a
// genuine beacon proof with an execution header of their own.
func TestGloasExecutionProofRejectsAlteredHash(t *testing.T) {
	body := &BeaconBlockBodyGloas{
		RandaoReveal: make([]byte, 96),
		Eth1Data:     &Eth1Data{DepositRoot: make([]byte, 32), BlockHash: make([]byte, 32)},
		Graffiti:     make([]byte, 32),
		SignedExecutionPayloadBid: &SignedExecutionPayloadBidGloas{
			Signature: make([]byte, 96),
			Message: &ExecutionPayloadBid{
				ParentBlockHash: []byte{0x01}, ParentBlockRoot: make([]byte, 32),
				BlockHash: make([]byte, 32), PrevRandao: make([]byte, 32),
				FeeRecipient: make([]byte, 20), ExecutionRequestsRoot: make([]byte, 32),
			},
		},
	}
	root, _ := body.HashTreeRoot()
	_, branch, err := body.ExecutionProof()
	if err != nil {
		t.Fatalf("execution proof: %v", err)
	}
	altered := bytesRoot([]byte{0x02})
	if verifyProof(t, altered, branch, GloasExecutionCommitmentGindex, root) {
		t.Fatal("an altered parent_block_hash still verified")
	}
}
