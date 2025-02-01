package beefy

import (
	"context"
	"fmt"
	"time"

	log "github.com/sirupsen/logrus"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"golang.org/x/sync/errgroup"

	"github.com/snowfork/snowbridge/relayer/chain/relaychain"
	"github.com/snowfork/snowbridge/relayer/substrate"
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
) (<-chan Request, error) {
	requests := make(chan Request, 1)

	eg.Go(func() error {
		defer close(requests)
		err := li.scanCommitments(ctx, currentBeefyBlock, requests)
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
	requests chan<- Request,
) error {
	in, err := ScanCommitments(ctx, li.conn.Metadata(), li.conn.API(), currentBeefyBlock+1)
	if err != nil {
		return fmt.Errorf("scan provable commitments: %w", err)
	}

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
			nextValidatorSetID := uint64(result.Proof.Leaf.BeefyNextAuthoritySet.ID)

			validators, err := li.queryBeefyAuthorities(result.BlockHash)
			if err != nil {
				return fmt.Errorf("fetch beefy authorities at block %v: %w", result.BlockHash, err)
			}

			task := Request{
				Validators:       validators,
				SignedCommitment: result.SignedCommitment,
				Proof:            result.Proof,
			}

			log.WithFields(log.Fields{
				"commitment": log.Fields{
					"blockNumber":        committedBeefyBlock,
					"validatorSetID":     validatorSetID,
					"nextValidatorSetID": nextValidatorSetID,
				},
			}).Info("Sending BEEFY commitment to ethereum writer")

			select {
			case <-ctx.Done():
				return ctx.Err()
			case requests <- task:
			}
		}
	}
}

func (li *PolkadotListener) queryBeefyAuthorities(blockHash types.Hash) ([]substrate.Authority, error) {
	storageKey, err := types.CreateStorageKey(li.conn.Metadata(), "Beefy", "Authorities", nil, nil)
	if err != nil {
		return nil, fmt.Errorf("create storage key: %w", err)
	}
	var authorities []substrate.Authority
	ok, err := li.conn.API().RPC.State.GetStorage(storageKey, &authorities, blockHash)
	if err != nil {
		return nil, err
	}
	if !ok {
		return nil, fmt.Errorf("beefy authorities not found")
	}

	return authorities, nil
}

func (li *PolkadotListener) generateBeefyUpdate(relayBlockNumber uint64) (Request, error) {
	api := li.conn.API()
	meta := li.conn.Metadata()
	var request Request
	beefyBlockHash, err := li.findNextBeefyBlock(relayBlockNumber)
	if err != nil {
		return request, fmt.Errorf("find match beefy block: %w", err)
	}

	commitment, proof, err := fetchCommitmentAndProof(meta, api, beefyBlockHash)
	if err != nil {
		return request, fmt.Errorf("fetch commitment and proof: %w", err)
	}

	committedBeefyBlockNumber := uint64(commitment.Commitment.BlockNumber)
	committedBeefyBlockHash, err := api.RPC.Chain.GetBlockHash(uint64(committedBeefyBlockNumber))

	validators, err := li.queryBeefyAuthorities(committedBeefyBlockHash)
	if err != nil {
		return request, fmt.Errorf("fetch beefy authorities at block %v: %w", committedBeefyBlockHash, err)
	}
	request = Request{
		Validators:       validators,
		SignedCommitment: *commitment,
		Proof:            *proof,
	}

	return request, nil
}

func (li *PolkadotListener) findNextBeefyBlock(blockNumber uint64) (types.Hash, error) {
	api := li.conn.API()
	var nextBeefyBlockHash, finalizedBeefyBlockHash types.Hash
	var err error
	nextBeefyBlockNumber := blockNumber
	for {
		finalizedBeefyBlockHash, err = api.RPC.Beefy.GetFinalizedHead()
		if err != nil {
			return nextBeefyBlockHash, fmt.Errorf("fetch beefy finalized head: %w", err)
		}
		finalizedBeefyBlockHeader, err := api.RPC.Chain.GetHeader(finalizedBeefyBlockHash)
		if err != nil {
			return nextBeefyBlockHash, fmt.Errorf("fetch block header: %w", err)
		}
		latestBeefyBlockNumber := uint64(finalizedBeefyBlockHeader.Number)
		if latestBeefyBlockNumber <= nextBeefyBlockNumber {
			// The relay block not finalized yet, just wait and retry
			time.Sleep(6 * time.Second)
			continue
		} else if latestBeefyBlockNumber <= nextBeefyBlockNumber+600 {
			// The relay block has been finalized not long ago(1 hour), just return the finalized block
			nextBeefyBlockHash = finalizedBeefyBlockHash
			break
		} else {
			// The relay block has been finalized for a long time, in this case return the next block
			// which contains a beefy justification
			for {
				if nextBeefyBlockNumber == latestBeefyBlockNumber {
					nextBeefyBlockHash = finalizedBeefyBlockHash
					break
				}
				nextBeefyBlockHash, err = api.RPC.Chain.GetBlockHash(nextBeefyBlockNumber)
				if err != nil {
					return nextBeefyBlockHash, fmt.Errorf("fetch block hash: %w", err)
				}
				block, err := api.RPC.Chain.GetBlock(nextBeefyBlockHash)
				if err != nil {
					return nextBeefyBlockHash, fmt.Errorf("fetch block: %w", err)
				}

				var commitment *types.SignedCommitment
				for j := range block.Justifications {
					sc := types.OptionalSignedCommitment{}
					if block.Justifications[j].EngineID() == "BEEF" {
						err := types.DecodeFromBytes(block.Justifications[j].Payload(), &sc)
						if err != nil {
							return nextBeefyBlockHash, fmt.Errorf("decode BEEFY signed commitment: %w", err)
						}
						ok, value := sc.Unwrap()
						if ok {
							commitment = &value
						}
					}
				}
				if commitment != nil {
					return nextBeefyBlockHash, nil
				}
				nextBeefyBlockNumber++
			}
		}
	}
	return nextBeefyBlockHash, nil
}
