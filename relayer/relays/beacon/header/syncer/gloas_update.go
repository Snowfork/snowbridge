package syncer

// The Gloas execution side of a header update.
//
// EIP-7732 takes the execution payload out of the beacon block, so there is no execution
// header to merkleize from consensus data. What the body commits to is the builder's bid, and
// within it the parent execution block hash. The relayer therefore proves that hash out of
// the body and submits the execution header whose Keccak hash it is, fetched from an
// execution node.

import (
	"context"
	"fmt"

	"github.com/ethereum/go-ethereum/common"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/cache"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/execution"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/api"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/scale"
)

func (s *Syncer) getGloasHeaderUpdate(blockRoot common.Hash, header api.BeaconHeader, checkpoint *cache.Proof) (scale.HeaderUpdatePayload, error) {
	if s.executionClient == nil {
		return scale.HeaderUpdatePayload{}, fmt.Errorf("gloas header update needs an execution endpoint, none is configured")
	}

	beaconHeader, err := header.ToScale()
	if err != nil {
		return scale.HeaderUpdatePayload{}, fmt.Errorf("beacon header to scale: %w", err)
	}

	block, err := s.Client.GetBeaconBlock(blockRoot)
	if err != nil {
		return scale.HeaderUpdatePayload{}, fmt.Errorf("fetch block %s: %w", blockRoot, err)
	}
	body, err := api.BuildGloasBody(&block.Data.Message.Body)
	if err != nil {
		return scale.HeaderUpdatePayload{}, fmt.Errorf("build gloas body: %w", err)
	}

	leaf, branch, err := body.ExecutionProof()
	if err != nil {
		return scale.HeaderUpdatePayload{}, fmt.Errorf("gloas execution proof: %w", err)
	}

	// The proven leaf is the execution block hash, so this is also a check that the body was
	// built correctly: a header that hashes to it cannot have come from anywhere else.
	headerRLP, err := execution.FetchHeaderRLP(context.Background(), s.executionClient, common.BytesToHash(leaf[:]))
	if err != nil {
		return scale.HeaderUpdatePayload{}, fmt.Errorf("execution header for %s: %w", common.BytesToHash(leaf[:]), err)
	}

	executionBranch := make([]types.H256, len(branch))
	for i, node := range branch {
		executionBranch[i] = types.NewH256(node[:])
	}
	executionHeader := scale.VersionedExecutionPayloadHeader{Gloas: headerRLP}

	// Without a checkpoint, or when this header is itself the finalized one, there is nothing
	// to prove ancestry against.
	if checkpoint == nil || uint64(beaconHeader.Slot) == checkpoint.Slot {
		return scale.HeaderUpdatePayload{
			Header:          beaconHeader,
			AncestryProof:   scale.OptionAncestryProof{HasValue: false},
			ExecutionHeader: executionHeader,
			ExecutionBranch: executionBranch,
		}, nil
	}

	ancestryProof, err := s.getBlockHeaderAncestryProof(int(beaconHeader.Slot), blockRoot, checkpoint.BlockRootsTree)
	if err != nil {
		return scale.HeaderUpdatePayload{}, err
	}

	return scale.HeaderUpdatePayload{
		Header: beaconHeader,
		AncestryProof: scale.OptionAncestryProof{
			HasValue: true,
			Value: scale.AncestryProof{
				HeaderBranch:       ancestryProof,
				FinalizedBlockRoot: types.NewH256(checkpoint.FinalizedBlockRoot.Bytes()),
			},
		},
		ExecutionHeader: executionHeader,
		ExecutionBranch: executionBranch,
	}, nil
}
