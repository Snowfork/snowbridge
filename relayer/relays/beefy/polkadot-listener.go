package beefy

import (
	"context"
	"fmt"

	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"golang.org/x/sync/errgroup"

	"github.com/snowfork/snowbridge/relayer/chain/relaychain"
	"github.com/snowfork/snowbridge/relayer/substrate"

	log "github.com/sirupsen/logrus"
)

type PolkadotListener struct {
	config *Config
	conn   *relaychain.Connection
	beefyAuthoritiesKey types.StorageKey
}

func NewPolkadotListener(
	config *Config,
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
	storageKey, err := types.CreateStorageKey(li.conn.Metadata(), "Beefy", "Authorities", nil, nil)
	if err != nil {
		return nil, err
	}
	li.beefyAuthoritiesKey = storageKey

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
	in, err := ScanSafeCommitments(ctx, li.conn.Metadata(), li.conn.API(), currentBeefyBlock+1, li.config.Source.BeefyActivationBlock)
	if err != nil {
		return fmt.Errorf("scan commitments: %w", err)
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

			if result.SignedCommitment.Commitment.ValidatorSetID == currentValidatorSet+1 {
				currentValidatorSet++

				validators, err := li.queryBeefyAuthorities(result.BlockHash)
				if err != nil {
					return fmt.Errorf("fetch beefy authorities at block %v: %w", result.BlockHash, err)
				}

				task := Request{
					Validators:       validators,
					SignedCommitment: result.SignedCommitment,
					Proof:            result.MMRProof,
					IsHandover:       true,
				}

				select {
				case <-ctx.Done():
					return ctx.Err()
				case requests <- task:
				}
			} else if result.SignedCommitment.Commitment.ValidatorSetID == currentValidatorSet {
				if result.Depth > li.config.Source.FastForwardDepth {
					log.WithFields(log.Fields{
						"commitment": log.Fields{
							"blockNumber":    result.SignedCommitment.Commitment.BlockNumber,
							"validatorSetID": result.SignedCommitment.Commitment.ValidatorSetID,
						},
					}).Info("Discarded commitment")
					continue
				}

				validators, err := li.queryBeefyAuthorities(result.BlockHash)
				if err != nil {
					return fmt.Errorf("fetch beefy authorities at block %v: %w", result.BlockHash, err)
				}

				task := Request{
					Validators:       validators,
					SignedCommitment: result.SignedCommitment,
					Proof:            result.MMRProof,
					IsHandover:       false,
				}

				// drop task if it can't be processed immediately
				select {
				case requests <- task:
				default:
					log.WithFields(log.Fields{
						"commitment": log.Fields{
							"blockNumber":    result.SignedCommitment.Commitment.BlockNumber,
							"validatorSetID": result.SignedCommitment.Commitment.ValidatorSetID,
						},
					}).Info("Discarded commitment")
				}
			} else {
				return fmt.Errorf("commitment has unexpected validatorSetID: blockNumber=%v validatorSetID=%v expectedValidatorSetID=%v",
					result.SignedCommitment.Commitment.BlockNumber,
					result.SignedCommitment.Commitment.ValidatorSetID,
					currentValidatorSet,
				)
			}
		}
	}
}

func (li *PolkadotListener) queryBeefyAuthorities(blockHash types.Hash) ([]substrate.Authority, error) {
	var authorities []substrate.Authority
	ok, err := li.conn.API().RPC.State.GetStorage(li.beefyAuthoritiesKey, &authorities, blockHash)
	if err != nil {
		return nil, err
	}
	if !ok {
		return nil, fmt.Errorf("beefy authorities not found")
	}

	return authorities, nil
}
