package state

// The Gloas BeaconBlockBody and the execution proof taken out of it.
//
// EIP-7732 removes execution_payload from the body and replaces it with a builder's signed
// bid. The value Snowbridge proves is the bid's parent_block_hash, at generalized index 2856,
// which composes three containers:
//
//	body field 10 (signed_execution_payload_bid)  -> gindex 357
//	  SignedExecutionPayloadBid.message           x2 -> 714
//	    ExecutionPayloadBid.parent_block_hash     x4 -> 2856, depth 11
//
// Progressive merkleization and proofs use dynamic-ssz (see gloas_dynssz.go).

import (
	"fmt"
)

const (
	// GloasExecutionCommitmentGindex is the bid's parent_block_hash within BeaconBlockBody.
	GloasExecutionCommitmentGindex = 2856
)

// BeaconBlockBodyGloas carries the 13 body fields as a ProgressiveContainer.
// Fields the relayer never inspects are held as their hash tree roots ([32]byte).
type BeaconBlockBodyGloas struct {
	RandaoReveal              []byte                         `ssz-index:"0" ssz-size:"96"`
	Eth1Data                  *Eth1Data                      `ssz-index:"1"`
	Graffiti                  []byte                         `ssz-index:"2" ssz-size:"32"`
	ProposerSlashingsRoot     [32]byte                       `ssz-index:"3"`
	AttesterSlashingsRoot     [32]byte                       `ssz-index:"4"`
	AttestationsRoot          [32]byte                       `ssz-index:"5"`
	DepositsRoot              [32]byte                       `ssz-index:"6"`
	VoluntaryExitsRoot        [32]byte                       `ssz-index:"7"`
	SyncAggregateRoot         [32]byte                       `ssz-index:"8"`
	BlsToExecutionChangesRoot [32]byte                       `ssz-index:"9"`
	SignedExecutionPayloadBid *SignedExecutionPayloadBidGloas `ssz-index:"10"`
	PayloadAttestationsRoot   [32]byte                       `ssz-index:"11"`
	ParentExecutionRequests   [32]byte                       `ssz-index:"12"`
}

// SignedExecutionPayloadBidGloas is a classic 2-field Container (message + signature).
type SignedExecutionPayloadBidGloas struct {
	Message   *ExecutionPayloadBid
	Signature []byte `ssz-size:"96"`
}

// bodySSZ is BeaconBlockBodyGloas without methods. dynamic-ssz hybrid-delegates to an
// exported HashTreeRoot when present, which would recurse into this wrapper forever.
type bodySSZ BeaconBlockBodyGloas

// HashTreeRoot computes the body root the beacon header commits to.
func (b *BeaconBlockBodyGloas) HashTreeRoot() ([32]byte, error) {
	return gloasSSZ.HashTreeRoot((*bodySSZ)(b))
}

// ExecutionProof returns the leaf and branch proving the bid's parent_block_hash at gindex
// 2856 against the body root.
func (b *BeaconBlockBodyGloas) ExecutionProof() (leaf [32]byte, branch [][32]byte, err error) {
	tree, err := gloasSSZ.GetTree((*bodySSZ)(b))
	if err != nil {
		return leaf, nil, fmt.Errorf("gloas body tree: %w", err)
	}
	proof, err := tree.Prove(GloasExecutionCommitmentGindex)
	if err != nil {
		return leaf, nil, fmt.Errorf("prove gindex %d: %w", GloasExecutionCommitmentGindex, err)
	}
	if len(proof.Leaf) != 32 {
		return leaf, nil, fmt.Errorf("execution proof leaf length %d, want 32", len(proof.Leaf))
	}
	copy(leaf[:], proof.Leaf)
	branch = make([][32]byte, len(proof.Hashes))
	for i, h := range proof.Hashes {
		if len(h) != 32 {
			return leaf, nil, fmt.Errorf("execution proof hash %d length %d, want 32", i, len(h))
		}
		copy(branch[i][:], h)
	}
	return leaf, branch, nil
}
