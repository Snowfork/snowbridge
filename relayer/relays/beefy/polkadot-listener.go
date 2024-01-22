package beefy

import (
	"context"
	"fmt"
	"time"

	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"golang.org/x/sync/errgroup"

	"github.com/snowfork/snowbridge/relayer/chain/relaychain"
	"github.com/snowfork/snowbridge/relayer/substrate"

	log "github.com/sirupsen/logrus"
)

type PolkadotListener struct {
	config *SourceConfig
	conn   *relaychain.Connection
}

func NewPolkadotListener(
	config *SourceConfig,
	conn *relaychain.Connection,
) *PolkadotListener {
	return &PolkadotListener{
		config: config,
		conn:   conn,
	}
}

func (li *PolkadotListener) Start(
	ctx context.Context,
	eg *errgroup.Group,
	currentBeefyBlock uint64,
	currentValidatorSetID uint64,
) (<-chan Request, error) {
	requests := make(chan Request)

	eg.Go(func() error {
		defer close(requests)
		err := li.scanCommitments(ctx, currentBeefyBlock, currentValidatorSetID, requests)
		if err != nil {
			return err
		}
		return nil
	})

	return requests, nil
}

func (li *PolkadotListener) scanCommitments(
	ctx context.Context,
	currentBeefyBlock uint64,
	currentValidatorSet uint64,
	requests chan<- Request,
) error {
	in, err := ScanSafeCommitments(ctx, li.conn.Metadata(), li.conn.API(), currentBeefyBlock+1)
	if err != nil {
		return fmt.Errorf("scan commitments: %w", err)
	}
	lastSyncedBeefyBlock := currentBeefyBlock

	for {
		select {
		case <-ctx.Done():
			return nil
		case result, ok := <-in:
			if !ok {
				return nil
			}
			if result.Error != nil {
				return fmt.Errorf("scan safe commitments: %w", result.Error)
			}

			committedBeefyBlock := uint64(result.SignedCommitment.Commitment.BlockNumber)
			validatorSetID := result.SignedCommitment.Commitment.ValidatorSetID
			nextValidatorSetID := uint64(result.MMRProof.Leaf.BeefyNextAuthoritySet.ID)

			if validatorSetID != currentValidatorSet && validatorSetID != currentValidatorSet+1 {
				return fmt.Errorf("commitment has unexpected validatorSetID: blockNumber=%v validatorSetID=%v expectedValidatorSetID=%v",
					committedBeefyBlock,
					validatorSetID,
					currentValidatorSet,
				)
			}

			logEntry := log.WithFields(log.Fields{
				"commitment": log.Fields{
					"blockNumber":        committedBeefyBlock,
					"validatorSetID":     validatorSetID,
					"nextValidatorSetID": nextValidatorSetID,
				},
				"validatorSetID":       currentValidatorSet,
				"IsHandover":           validatorSetID == currentValidatorSet+1,
				"lastSyncedBeefyBlock": lastSyncedBeefyBlock,
			})

			validators, err := li.queryBeefyAuthorities(result.BlockHash)
			if err != nil {
				return fmt.Errorf("fetch beefy authorities at block %v: %w", result.BlockHash, err)
			}
			task := Request{
				Validators:       validators,
				SignedCommitment: result.SignedCommitment,
				Proof:            result.MMRProof,
			}

			if validatorSetID == currentValidatorSet+1 && validatorSetID == nextValidatorSetID-1 {
				task.IsHandover = true
				select {
				case <-ctx.Done():
					return ctx.Err()
				case requests <- task:
					logEntry.Info("New commitment with handover added to channel")
					currentValidatorSet++
					lastSyncedBeefyBlock = committedBeefyBlock
				}
			} else if validatorSetID == currentValidatorSet {
				if result.Depth > li.config.FastForwardDepth {
					logEntry.Warn("Discarded commitment with depth not fast forward")
					continue
				}
				if committedBeefyBlock < lastSyncedBeefyBlock+li.config.UpdatePeriod {
					logEntry.Info("Discarded commitment with sampling")
					continue
				}

				// drop task if it can't be processed immediately
				select {
				case <-ctx.Done():
					return ctx.Err()
				case requests <- task:
					lastSyncedBeefyBlock = committedBeefyBlock
					logEntry.Info("New commitment added to channel")
				default:
					logEntry.Warn("Discarded commitment fail adding to channel")
				}
			} else {
				logEntry.Warn("Discarded invalid commitment")
			}
		}
	}
}

func (li *PolkadotListener) queryBeefyAuthorities(blockHash types.Hash) ([]substrate.Authority, error) {
	var authorities []substrate.Authority
	storageKey, err := types.CreateStorageKey(li.conn.Metadata(), "Beefy", "Authorities", nil, nil)
	if err != nil {
		return nil, fmt.Errorf("create storage key: %w", err)
	}
	ok, err := li.conn.API().RPC.State.GetStorage(storageKey, &authorities, blockHash)
	if err != nil {
		return nil, err
	}
	if !ok {
		return nil, fmt.Errorf("beefy authorities not found")
	}

	return authorities, nil
}

func (li *PolkadotListener) queryBeefyNextAuthoritySet(blockHash types.Hash) (types.BeefyNextAuthoritySet, error) {
	var nextAuthoritySet types.BeefyNextAuthoritySet
	storageKey, err := types.CreateStorageKey(li.conn.Metadata(), "MmrLeaf", "BeefyNextAuthorities", nil, nil)
	ok, err := li.conn.API().RPC.State.GetStorage(storageKey, &nextAuthoritySet, blockHash)
	if err != nil {
		return nextAuthoritySet, err
	}
	if !ok {
		return nextAuthoritySet, fmt.Errorf("beefy nextAuthoritySet not found")
	}

	return nextAuthoritySet, nil
}

func (li *PolkadotListener) generateNextBeefyUpdate(nextBlockNumber uint64) (Request, error) {
	api := li.conn.API()
	meta := li.conn.Metadata()
	var request Request
	var nextBeefyBlockNumber uint64
	var nextBeefyBlockHash types.Hash
	for {
		finalizedBeefyBlockHash, err := api.RPC.Beefy.GetFinalizedHead()
		if err != nil {
			return request, fmt.Errorf("fetch beefy finalized head: %w", err)
		}

		finalizedBeefyBlockHeader, err := api.RPC.Chain.GetHeader(finalizedBeefyBlockHash)
		if err != nil {
			return request, fmt.Errorf("fetch block header: %w", err)
		}

		nextBeefyBlockNumber = uint64(finalizedBeefyBlockHeader.Number)
		if nextBeefyBlockNumber < nextBlockNumber {
			time.Sleep(6 * time.Second)
			continue
		} else {
			nextBeefyBlockHash = finalizedBeefyBlockHash
			break
		}
	}

	nextFinalizedBeefyBlock, err := api.RPC.Chain.GetBlock(nextBeefyBlockHash)
	if err != nil {
		return request, fmt.Errorf("fetch block: %w", err)
	}

	var commitment *types.SignedCommitment
	for j := range nextFinalizedBeefyBlock.Justifications {
		sc := types.OptionalSignedCommitment{}
		if nextFinalizedBeefyBlock.Justifications[j].EngineID() == "BEEF" {
			err := types.DecodeFromBytes(nextFinalizedBeefyBlock.Justifications[j].Payload(), &sc)
			if err != nil {
				return request, fmt.Errorf("decode BEEFY signed commitment: %w", err)
			}
			ok, value := sc.Unwrap()
			if ok {
				commitment = &value
			}
		}
	}
	if commitment == nil {
		return request, fmt.Errorf("beefy block without a valid commitment")
	}

	proofIsValid, proof, err := makeProof(meta, api, uint32(nextBeefyBlockNumber), nextBeefyBlockHash)
	if err != nil {
		return request, fmt.Errorf("proof generation for block %v at hash %v: %w", nextBeefyBlockNumber, nextBeefyBlockHash.Hex(), err)
	}
	if !proofIsValid {
		return request, fmt.Errorf("Proof for leaf is invalid for block %v at hash %v: %w", nextBeefyBlockNumber, nextBeefyBlockHash.Hex(), err)
	}

	committedBeefyBlockNumber := uint64(commitment.Commitment.BlockNumber)
	committedBeefyBlockHash, err := api.RPC.Chain.GetBlockHash(uint64(committedBeefyBlockNumber))
	validatorSetID := commitment.Commitment.ValidatorSetID
	nextValidatorSetID := uint64(proof.Leaf.BeefyNextAuthoritySet.ID)

	validators, err := li.queryBeefyAuthorities(committedBeefyBlockHash)
	if err != nil {
		return request, fmt.Errorf("fetch beefy authorities at block %v: %w", committedBeefyBlockHash, err)
	}
	request = Request{
		Validators:       validators,
		SignedCommitment: *commitment,
		Proof:            proof,
		IsHandover:       nextValidatorSetID == validatorSetID+2,
	}

	return request, nil
}
