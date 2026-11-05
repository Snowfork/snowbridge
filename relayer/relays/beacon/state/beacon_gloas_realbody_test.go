package state

import (
	"encoding/hex"
	"testing"

)

func hexBytes(s string) []byte {
	b, err := hex.DecodeString(s)
	if err != nil {
		panic(err)
	}
	return b
}

func hex32(s string) [32]byte {
	var out [32]byte
	copy(out[:], hexBytes(s))
	return out
}

// The body of Plataberget block 115968, which the node reports as body root
// 3b72fb69... The four structured fields are the real values; the nine the relayer never
// inspects are injected as the roots that same block produces.
//
// Reproducing the body root proves two things the synthetic tests cannot: that the 13 fields
// are assembled in the right order, and that the bid's own 12-field progressive container
// merkleizes correctly -- which is what gindex 2856 descends into.
func realBodySlot115968() *BeaconBlockBodyGloas {
	return &BeaconBlockBodyGloas{
		RandaoReveal: hexBytes("b985c64ad4ec366f52db14d247588ee9a78a184e40c555aa9cdbe02cef7329d80421a82a08da59e14b9e27778cc1356f0b645ced3c4def776bc54f5b4889b5951650db4ddbae97a38d3f0c093df0086a40e87dfb52baa09aa9eb3180ca567128"),
		Eth1Data: &Eth1Data{
			DepositRoot:  hexBytes("0000000000000000000000000000000000000000000000000000000000000000"),
			DepositCount: 0,
			BlockHash:    hexBytes("0000000000000000000000000000000000000000000000000000000000000000"),
		},
		Graffiti:                  hexBytes("6e696d6275732d6e696d627573656c2d31000000000000000000000000000000"),
		ProposerSlashingsRoot:     hex32("f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"),
		AttesterSlashingsRoot:     hex32("f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"),
		AttestationsRoot:          hex32("d8b5d6075c9a531cf80f112d078df95249a260eef7b143558ab13c963979865e"),
		DepositsRoot:              hex32("f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"),
		VoluntaryExitsRoot:        hex32("f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"),
		SyncAggregateRoot:         hex32("54bb8f69222b00d7081d812919207bc3622184c41c513047afbbcf9214518b94"),
		BlsToExecutionChangesRoot: hex32("f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"),
		PayloadAttestationsRoot:   hex32("30deed299e2c1cafa6a2c00e26b22cb2d9b431cd17e31e6a833812f21bfcb52a"),
		ParentExecutionRequests:   hex32("87b69a306c8e430d0857f7c4ac5e27cecffa1108d43c2e5df7388056fea7a423"),
		SignedExecutionPayloadBid: &SignedExecutionPayloadBidGloas{
			Signature: hexBytes("8560397d2a187b1bc8898f0c10685cc0ee08c9a430cafe3233a620a586e4776be297db45054f8fcfa414e07b2dfe4e240b8b56c1a50ea7d4bef7c4a0d739dd2ad6479b049e1d1c84d7b30f2a787cdc215601bdfdf1fb874b7d0af73325ff28eb"),
			Message: &ExecutionPayloadBid{
				ParentBlockHash:       hexBytes("9af0c93804597fe505c22f3b15abb47acb01f7c6106474b196b52f592fae0e39"),
				ParentBlockRoot:       hexBytes("84a3016f712471145043f63fe36f9c9dbf7c16b3a709ce4a2b22678b77acbc80"),
				BlockHash:             hexBytes("271de4ebc15297bc52208f1b902bcb781a85cd51e1a545783dbe2da8faf01ff1"),
				PrevRandao:            hexBytes("44a4f402a3c3c4c25dce631deca21dd284d210d3d81e13317e827d890d809161"),
				FeeRecipient:          hexBytes("f97e180c050e5ab072211ad2c213eb5aee4df134"),
				GasLimit:              199609569,
				BuilderIndex:          9994,
				Slot:                  115968,
				Value:                 165110802,
				ExecutionPayment:      0,
				ExecutionRequestsRoot: hexBytes("87b69a306c8e430d0857f7c4ac5e27cecffa1108d43c2e5df7388056fea7a423"),
				BlobKzgCommitments:    [][]byte{},
			},
		},
	}
}

const (
	realBodyRoot = "3b72fb6996412d59a20d92dde86abe3a12b94232ba62a61ac301625f1f8950f0"
	// Bid root for the same Platåberget slot-115968 bid (independent of body assembly).
	realBidRoot = "83fcc67866c359c93d52588201c83b8f09e394fcf02401434717520f9aee380e"
)

func TestGloasRealBidRootMatchesNode(t *testing.T) {
	root, err := realBodySlot115968().SignedExecutionPayloadBid.Message.hashTreeRoot()
	if err != nil {
		t.Fatalf("bid root: %v", err)
	}
	if hex.EncodeToString(root[:]) != realBidRoot {
		t.Fatalf("bid root mismatch\n got  %x\n want %s", root, realBidRoot)
	}
}

func TestGloasRealBodyRootMatchesNode(t *testing.T) {
	root, err := realBodySlot115968().HashTreeRoot()
	if err != nil {
		t.Fatalf("body root: %v", err)
	}
	if hex.EncodeToString(root[:]) != realBodyRoot {
		t.Fatalf("body root mismatch\n got  %x\n want %s", root, realBodyRoot)
	}
}

// With the body root confirmed against the node, the execution proof is now anchored to real
// data rather than to a body of our own construction.
func TestGloasRealExecutionProofVerifies(t *testing.T) {
	body := realBodySlot115968()
	root, err := body.HashTreeRoot()
	if err != nil {
		t.Fatalf("body root: %v", err)
	}
	leaf, branch, err := body.ExecutionProof()
	if err != nil {
		t.Fatalf("execution proof: %v", err)
	}
	if len(branch) != 11 {
		t.Fatalf("branch depth %d, expected 11", len(branch))
	}
	if !verifyProof(t, leaf, branch, GloasExecutionCommitmentGindex, root) {
		t.Fatalf("branch does not verify at gindex %d", GloasExecutionCommitmentGindex)
	}
	want := "9af0c93804597fe505c22f3b15abb47acb01f7c6106474b196b52f592fae0e39"
	if hex.EncodeToString(leaf[:]) != want {
		t.Fatalf("leaf is %x, expected the bid's parent_block_hash %s", leaf, want)
	}
}
