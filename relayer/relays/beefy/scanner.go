package beefy

import (
	"context"
	"fmt"
	"time"

	gsrpc "github.com/snowfork/go-substrate-rpc-client/v4"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
)

type Result struct {
	BlockNumber uint64
	BlockHash   types.Hash
	Error       *error
}

func closeWithError(err error, results chan<- Result) {
	results <- Result{Error: &err}
	close(results)
}

func ScanBlocks(ctx context.Context, api *gsrpc.SubstrateAPI, startBlock uint64) (chan Result, error) {
	results := make(chan Result)
	go scanBlocks(ctx, api, startBlock, results)
	return results, nil
}

func scanBlocks(ctx context.Context, api *gsrpc.SubstrateAPI, startBlock uint64, results chan<- Result) {
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
			select {
			case <-ctx.Done():
				closeWithError(ctx.Err(), results)
			case <-time.After(2 * time.Second):
			}
			continue
		}

		blockHash, err := api.RPC.Chain.GetBlockHash(current)
		if err != nil {
			closeWithError(fmt.Errorf("fetch block hash: %w", err),
				results,
			)
		}

		results <- Result{BlockNumber: current, BlockHash: blockHash}

		current++
	}
}
