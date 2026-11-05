package syncer

import (
	"errors"
	"os"
	"testing"

	"github.com/ethereum/go-ethereum/common"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/api"
)

// Plataberget block 115968 carries a bid naming execution block 0x9af0c938... as its parent,
// which was produced at slot 115967. Finding 115968 from that pair is the whole successor
// search: post-Gloas the block committing to an execution block is a later one, because the
// bid commits to the parent hash.
const (
	successorPayloadSlot   = 115967
	successorExecutionHash = "0x9af0c93804597fe505c22f3b15abb47acb01f7c6106474b196b52f592fae0e39"
	successorExpectedSlot  = 115968
)

func gloasBeaconURL() string {
	if u := os.Getenv("GLOAS_BEACON_URL"); u != "" {
		return u
	}
	return "https://beacon.plataberget.ethpandaops.io"
}

func newNetworkSyncer(t *testing.T) *Syncer {
	t.Helper()
	client := api.NewBeaconClient([]string{gloasBeaconURL()})
	if _, err := client.GetHeaderBySlot(successorExpectedSlot); err != nil {
		t.Skipf("testnet unreachable or block pruned: %v", err)
	}
	return &Syncer{Client: client}
}

func TestFindSuccessorBeaconHeader(t *testing.T) {
	s := newNetworkSyncer(t)

	header, err := s.FindSuccessorBeaconHeader(successorPayloadSlot, common.HexToHash(successorExecutionHash), 32)
	if err != nil {
		t.Fatalf("find successor: %v", err)
	}
	if header.Slot != successorExpectedSlot {
		t.Fatalf("found slot %d, expected %d", header.Slot, successorExpectedSlot)
	}
	if len(header.BodyRoot) != 32 {
		t.Fatalf("body root is %d bytes", len(header.BodyRoot))
	}
}

// A hash no bid names must fail cleanly rather than scanning forever. This is the withheld
// payload case: the builder never revealed, so latest_block_hash never advanced past it.
func TestFindSuccessorGivesUpOnUnknownHash(t *testing.T) {
	s := newNetworkSyncer(t)

	_, err := s.FindSuccessorBeaconHeader(successorPayloadSlot, common.HexToHash("0xdead"), 4)
	if !errors.Is(err, ErrNoSuccessorBlock) {
		t.Fatalf("expected ErrNoSuccessorBlock, got %v", err)
	}
}
