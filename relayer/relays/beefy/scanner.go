package beefy

import (
	"context"
	"fmt"
	"time"

	gsrpc "github.com/snowfork/go-substrate-rpc-client/v4"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/crypto/merkle"
)

type ScanBlocksResult struct {
	BlockNumber  uint64
	BlockHash    types.Hash
	SessionIndex uint32
	Error        *error
}

func closeWithError(err error, results chan<- ScanBlocksResult) {
	results <- ScanBlocksResult{Error: &err}
	close(results)
}

func ScanBlocks(ctx context.Context, meta *types.Metadata, api *gsrpc.SubstrateAPI, startBlock uint64, halt bool) (chan ScanBlocksResult, error) {
	results := make(chan ScanBlocksResult)
	sessionIndexKey, err := types.CreateStorageKey(meta, "Session", "CurrentIndex", nil, nil)
	if err != nil {
		return results, err
	}

	go scanBlocks(ctx, api, sessionIndexKey, startBlock, halt, results)
	return results, nil
}

func scanBlocks(ctx context.Context, api *gsrpc.SubstrateAPI, sessionIndexKey types.StorageKey, startBlock uint64, halt bool, results chan<- ScanBlocksResult) {
	current := startBlock
	for {
		finalizedHash, err := api.RPC.Beefy.GetFinalizedHead()
		if err != nil {
			closeWithError(fmt.Errorf("fetch finalized head: %w", err),
				results,
			)
			return
		}

		finalizedHeader, err := api.RPC.Chain.GetHeader(finalizedHash)
		if err != nil {
			closeWithError(fmt.Errorf("fetch header for finalised head %v: %w", finalizedHash.Hex(), err),
				results,
			)
		}

		finalizedBlockNumber := uint64(finalizedHeader.Number)
		if current > finalizedBlockNumber {
			if halt {
				close(results)
				return
			}

			select {
			case <-ctx.Done():
				closeWithError(ctx.Err(), results)
			case <-time.After(6 * time.Second):
			}
			continue
		}

		blockHash, err := api.RPC.Chain.GetBlockHash(current)
		if err != nil {
			closeWithError(fmt.Errorf("fetch block hash: %w", err),
				results,
			)
		}

		var sessionIndex uint32

		_, err = api.RPC.State.GetStorage(sessionIndexKey, &sessionIndex, blockHash)
		if err != nil {
			closeWithError(fmt.Errorf("fetch session index: %w", err),
				results,
			)
		}

		results <- ScanBlocksResult{BlockNumber: current, BlockHash: blockHash, SessionIndex: sessionIndex}

		current++
	}
}

type ScanCommitmentsResult struct {
	SignedCommitment types.SignedCommitment
	Proof            merkle.SimplifiedMMRProof
	BlockNumber      uint64
	BlockHash        types.Hash
	Error            *error
}

func ScanCommitments(ctx context.Context, meta *types.Metadata, api *gsrpc.SubstrateAPI, startBlock uint64, halt bool) (<-chan ScanCommitmentsResult, error) {
	out := make(chan ScanCommitmentsResult)
	go scanCommitments(ctx, meta, api, startBlock, halt, out)
	return out, nil
}

func scanCommitments(ctx context.Context, meta *types.Metadata, api *gsrpc.SubstrateAPI, startBlock uint64, halt bool, out chan<- ScanCommitmentsResult) error {
	in, err := ScanBlocks(ctx, meta, api, startBlock, halt)
	if err != nil {
		return err
	}

	for {
		select {
		case <-ctx.Done():
			return ctx.Err()
		case blockResult, ok := <-in:
			if !ok {
				return nil
			}

			if blockResult.Error != nil {
				out <- ScanCommitmentsResult{Error: blockResult.Error}
				close(out)
			}

			block, err := api.RPC.Chain.GetBlock(blockResult.BlockHash)
			if err != nil {
				return fmt.Errorf("fetch block: %w", err)
			}

			var commitment *types.SignedCommitment
			for j := range block.Justifications {
				sc := types.OptionalSignedCommitment{}
				if block.Justifications[j].EngineID() == "BEEF" {
					err := types.DecodeFromBytes(block.Justifications[j].Payload(), &sc)
					if err != nil {
						return fmt.Errorf("decode BEEFY signed commitment: %w", err)
					}
					ok, value := sc.Unwrap()
					if ok {
						commitment = &value
					}
				}
			}

			if commitment == nil {
				continue
			}



			out <- ScanCommitmentsResult{
				BlockNumber:      blockResult.BlockNumber,
				BlockHash:        blockResult.BlockHash,
				SignedCommitment: c,
			}

		}
	}
}

func getSessionIndex(blockNumber uint64, meta *types.Metadata, api *gsrpc.SubstrateAPI) (uint32, error) {
	var sessionIndex uint32

	sessionIndexKey, err := types.CreateStorageKey(meta, "Session", "CurrentIndex", nil, nil)
	if err != nil {
		return 0, err
	}

	blockHash, err := api.RPC.Chain.GetBlockHash(blockNumber)
	if err != nil {
		return 0, err
	}

	_, err = api.RPC.State.GetStorage(sessionIndexKey, &sessionIndex, blockHash)
	if err != nil {
		return 0, err
	}

	return sessionIndex, nil
}
