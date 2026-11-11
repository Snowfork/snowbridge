package syncer

import (
	"errors"
	"fmt"
	"strings"

	"github.com/ethereum/go-ethereum/common"
	log "github.com/sirupsen/logrus"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/api"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/state"
)

// ErrNoSuccessorBlock is returned when no beacon block within the search window names the
// target execution block as its bid's parent. The usual cause is a withheld payload: the
// builder never revealed it, so state.latest_block_hash never advanced past it and no later
// bid can name it. Such an event is not provable and the caller should not retry forever.
var ErrNoSuccessorBlock = errors.New("no beacon block commits to this execution block")

// FindSuccessorBeaconHeader locates the beacon block whose execution payload bid names
// executionBlockHash as its parent.
//
// Before Gloas the beacon block at slot S carried the payload for S, so proving an execution
// block meant proving the block it sat in. EIP-7732 removes the payload from the body and
// leaves a bid committing to the *parent* execution block hash, so the block that commits to
// execution block N is a successor of the one that produced it.
//
// The search walks forward from the payload's own slot, tolerating missed slots the same way
// FindBeaconHeaderWithBlockIncluded does. It stops at the first block whose bid matches, and
// gives up after maxSlots rather than scanning indefinitely.
func (s *Syncer) FindSuccessorBeaconHeader(payloadSlot uint64, executionBlockHash common.Hash, maxSlots uint64) (state.BeaconBlockHeader, error) {
	target := strings.ToLower(executionBlockHash.Hex())

	for offset := uint64(1); offset <= maxSlots; offset++ {
		slot := payloadSlot + offset
		block, err := s.Client.GetBeaconBlockBySlot(slot)
		if errors.Is(err, api.ErrNotFound) {
			continue // missed slot, keep walking
		}
		if err != nil {
			return state.BeaconBlockHeader{}, fmt.Errorf("fetch block at slot %d: %w", slot, err)
		}

		bid := block.Data.Message.Body.SignedExecutionPayloadBid
		if bid == nil {
			// Pre-Gloas block: the payload lives in the body, so there is no successor to find
			// and the caller is using the wrong path for this fork.
			return state.BeaconBlockHeader{}, fmt.Errorf(
				"block at slot %d has no execution payload bid; it predates Gloas", slot)
		}
		if !strings.EqualFold(bid.Message.ParentBlockHash, target) {
			continue
		}

		header, err := s.Client.GetHeaderBySlot(slot)
		if err != nil {
			return state.BeaconBlockHeader{}, fmt.Errorf("fetch header at slot %d: %w", slot, err)
		}
		log.WithFields(log.Fields{
			"payloadSlot":        payloadSlot,
			"successorSlot":      slot,
			"executionBlockHash": target,
		}).Debug("found the beacon block committing to this execution block")

		return state.BeaconBlockHeader{
			Slot:          header.Slot,
			ProposerIndex: header.ProposerIndex,
			ParentRoot:    header.ParentRoot.Bytes(),
			StateRoot:     header.StateRoot.Bytes(),
			BodyRoot:      header.BodyRoot.Bytes(),
		}, nil
	}

	log.WithFields(log.Fields{
		"payloadSlot":        payloadSlot,
		"executionBlockHash": target,
		"searched":           maxSlots,
	}).Warn("no successor block found; the payload was most likely withheld")
	return state.BeaconBlockHeader{}, ErrNoSuccessorBlock
}
