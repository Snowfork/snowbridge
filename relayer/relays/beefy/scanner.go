package beefy

import (
	"context"
	"fmt"
	"time"

	gsrpc "github.com/snowfork/go-substrate-rpc-client/v4"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/crypto/keccak"
	"github.com/snowfork/snowbridge/relayer/crypto/merkle"
)

type ScanBlocksResult struct {
	BlockNumber uint64
	BlockHash   types.Hash
	Error       error
}

func ScanBlocks(ctx context.Context, meta *types.Metadata, api *gsrpc.SubstrateAPI, startBlock uint64) (chan ScanBlocksResult, error) {
	results := make(chan ScanBlocksResult)
	go scanBlocks(ctx, meta, api, startBlock, results)
	return results, nil
}

func scanBlocks(ctx context.Context, meta *types.Metadata, api *gsrpc.SubstrateAPI, startBlock uint64, out chan<- ScanBlocksResult) {
	defer close(out)

	emitError := func(err error) {
		select {
		case <-ctx.Done():
			return
		case out <- ScanBlocksResult{Error: err}:
		}
	}

	fetchFinalizedBeefyHeader := func() (*types.Header, error) {
		finalizedHash, err := api.RPC.Beefy.GetFinalizedHead()
		if err != nil {
			return nil, fmt.Errorf("fetch finalized head: %w", err)
		}

		finalizedHeader, err := api.RPC.Chain.GetHeader(finalizedHash)
		if err != nil {
			return nil, fmt.Errorf("fetch header for finalised head %v: %w", finalizedHash.Hex(), err)
		}

		return finalizedHeader, nil
	}

	sessionCurrentIndexKey, err := types.CreateStorageKey(meta, "Session", "CurrentIndex", nil, nil)
	if err != nil {
		emitError(fmt.Errorf("create storage key: %w", err))
		return
	}

	blockHash, err := api.RPC.Chain.GetBlockHash(max(startBlock-1, 0))
	if err != nil {
		emitError(fmt.Errorf("fetch block hash: %w", err))
		return
	}

	// Get session index of block before start block
	var currentSessionIndex uint32
	_, err = api.RPC.State.GetStorage(sessionCurrentIndexKey, &currentSessionIndex, blockHash)
	if err != nil {
		emitError(fmt.Errorf("fetch session index: %w", err))
		return
	}

	finalizedHeader, err := fetchFinalizedBeefyHeader()
	if err != nil {
		emitError(err)
		return
	}
	current := startBlock
	for {
		finalizedBlockNumber := uint64(finalizedHeader.Number)
		if current > finalizedBlockNumber {
			select {
			case <-ctx.Done():
				return
			case <-time.After(3 * time.Second):
			}
			finalizedHeader, err = fetchFinalizedBeefyHeader()
			if err != nil {
				emitError(err)
				return
			}
			continue
		}

		if current > uint64(finalizedHeader.Number) {
			return
		}

		blockHash, err := api.RPC.Chain.GetBlockHash(current)
		if err != nil {
			emitError(fmt.Errorf("fetch block hash: %w", err))
			return
		}

		var sessionIndex uint32
		_, err = api.RPC.State.GetStorage(sessionCurrentIndexKey, &sessionIndex, blockHash)
		if err != nil {
			emitError(fmt.Errorf("fetch session index: %w", err))
			return
		}

		if sessionIndex > currentSessionIndex {
			currentSessionIndex = sessionIndex
		} else {
			current++
			continue
		}

		select {
		case <-ctx.Done():
			return
		case out <- ScanBlocksResult{BlockNumber: current, BlockHash: blockHash}:
		}

		current++
	}
}

type ScanCommitmentsResult struct {
	SignedCommitment types.SignedCommitment
	Proof            merkle.SimplifiedMMRProof
	BlockHash        types.Hash
	Error            error
}

func ScanCommitments(ctx context.Context, meta *types.Metadata, api *gsrpc.SubstrateAPI, startBlock uint64) (<-chan ScanCommitmentsResult, error) {
	out := make(chan ScanCommitmentsResult)
	go scanCommitments(ctx, meta, api, startBlock, out)
	return out, nil
}

func scanCommitments(ctx context.Context, meta *types.Metadata, api *gsrpc.SubstrateAPI, startBlock uint64, out chan<- ScanCommitmentsResult) {
	defer close(out)

	emitError := func(err error) {
		select {
		case <-ctx.Done():
			return
		case out <- ScanCommitmentsResult{Error: err}:
		}
	}

	in, err := ScanBlocks(ctx, meta, api, startBlock)
	if err != nil {
		emitError(err)
		return
	}

	for {
		select {
		case <-ctx.Done():
			emitError(err)
			return
		case result, ok := <-in:
			if !ok {
				return
			}

			if result.Error != nil {
				emitError(result.Error)
				return
			}

			commitment, proof, err := fetchCommitmentAndProof(meta, api, result.BlockHash)
			if err != nil {
				emitError(fmt.Errorf("fetch commitment and proof: %w", err))
				return
			}

			select {
			case <-ctx.Done():
				return
			case out <- ScanCommitmentsResult{BlockHash: result.BlockHash, SignedCommitment: *commitment, Proof: *proof}:
			}
		}
	}
}

func makeProof(meta *types.Metadata, api *gsrpc.SubstrateAPI, blockNumber uint32, blockHash types.Hash) (bool, merkle.SimplifiedMMRProof, error) {
	proof1, err := api.RPC.MMR.GenerateProof(blockNumber, blockHash)
	if err != nil {
		return false, merkle.SimplifiedMMRProof{}, fmt.Errorf("mmr_generateProof(%v, %v): %w", blockNumber, blockHash.Hex(), err)
	}

	proof2, err := merkle.ConvertToSimplifiedMMRProof(
		proof1.BlockHash,
		uint64(proof1.Proof.LeafIndex),
		proof1.Leaf,
		uint64(proof1.Proof.LeafCount),
		proof1.Proof.Items,
	)
	if err != nil {
		return false, merkle.SimplifiedMMRProof{}, fmt.Errorf("simplified proof conversion for block %v: %w", proof1.BlockHash.Hex(), err)
	}

	proofIsValid, err := verifyProof(meta, api, proof2)
	if err != nil {
		return false, merkle.SimplifiedMMRProof{}, fmt.Errorf("proof verification: %w", err)
	}

	return proofIsValid, proof2, nil
}

// Verify the actual MMR Root we calculated is same as value in storage of relaychain
func verifyProof(meta *types.Metadata, api *gsrpc.SubstrateAPI, proof merkle.SimplifiedMMRProof) (bool, error) {
	leafEncoded, err := types.EncodeToBytes(proof.Leaf)
	if err != nil {
		return false, err
	}
	leafHashBytes := (&keccak.Keccak256{}).Hash(leafEncoded)

	var leafHash types.H256
	copy(leafHash[:], leafHashBytes[0:32])

	actualRoot := merkle.CalculateMerkleRoot(&proof, leafHash)
	if err != nil {
		return false, err
	}

	var expectedRoot types.H256

	mmrRootKey, err := types.CreateStorageKey(meta, "Mmr", "RootHash", nil, nil)
	if err != nil {
		return false, err
	}

	_, err = api.RPC.State.GetStorage(mmrRootKey, &expectedRoot, types.Hash(proof.Blockhash))
	if err != nil {
		return false, err
	}

	return actualRoot == expectedRoot, nil
}

func fetchCommitmentAndProof(meta *types.Metadata, api *gsrpc.SubstrateAPI, beefyBlockHash types.Hash) (*types.SignedCommitment, *merkle.SimplifiedMMRProof, error) {
	beefyHeader, err := api.RPC.Chain.GetHeader(beefyBlockHash)
	if err != nil {
		return nil, nil, fmt.Errorf("fetch header: %w", err)
	}
	beefyBlock, err := api.RPC.Chain.GetBlock(beefyBlockHash)
	if err != nil {
		return nil, nil, fmt.Errorf("fetch block: %w", err)
	}

	var commitment *types.SignedCommitment
	for j := range beefyBlock.Justifications {
		sc := types.OptionalSignedCommitment{}
		if beefyBlock.Justifications[j].EngineID() == "BEEF" {
			err := types.DecodeFromBytes(beefyBlock.Justifications[j].Payload(), &sc)
			if err != nil {
				return nil, nil, fmt.Errorf("decode BEEFY signed commitment: %w", err)
			}
			ok, value := sc.Unwrap()
			if ok {
				commitment = &value
			}
		}
	}
	if commitment == nil {
		return nil, nil, fmt.Errorf("beefy block without a valid commitment")
	}

	proofIsValid, proof, err := makeProof(meta, api, uint32(beefyHeader.Number), beefyBlockHash)
	if err != nil {
		return nil, nil, fmt.Errorf("proof generation for block %v at hash %v: %w", beefyHeader.Number, beefyBlockHash.Hex(), err)
	}
	if !proofIsValid {
		return nil, nil, fmt.Errorf("Proof for leaf is invalid for block %v at hash %v: %w", beefyHeader.Number, beefyBlockHash.Hex(), err)
	}
	return commitment, &proof, nil
}
