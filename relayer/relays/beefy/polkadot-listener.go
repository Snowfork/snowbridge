package beefy

import (
	"context"
	"fmt"

	log "github.com/sirupsen/logrus"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"golang.org/x/sync/errgroup"

	"github.com/snowfork/snowbridge/relayer/chain/relaychain"
	"github.com/snowfork/snowbridge/relayer/substrate"
)

type PolkadotListener struct {
	config              *SourceConfig
	conn                *relaychain.Connection
	beefyAuthoritiesKey types.StorageKey
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
	storageKey, err := types.CreateStorageKey(li.conn.Metadata(), "Beefy", "Authorities", nil, nil)
	if err != nil {
		return nil, fmt.Errorf("create storage key: %w", err)
	}
	li.beefyAuthoritiesKey = storageKey

	requests := make(chan Request, 1)

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
	in, err := ScanProvableCommitments(ctx, li.conn.Metadata(), li.conn.API(), currentBeefyBlock+1)
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
			nextValidatorSetID := uint64(result.MMRProof.Leaf.BeefyNextAuthoritySet.ID)

			validators, err := li.queryBeefyAuthorities(result.BlockHash)
			if err != nil {
				return fmt.Errorf("fetch beefy authorities at block %v: %w", result.BlockHash, err)
			}

			currentAuthoritySet, err := li.queryBeefyAuthoritySet(result.BlockHash)
			if err != nil {
				return fmt.Errorf("fetch beefy authority set at block %v: %w", result.BlockHash, err)
			}

			nextAuthoritySet, err := li.queryBeefyNextAuthoritySet(result.BlockHash)
			if err != nil {
				return fmt.Errorf("fetch beefy next authority set at block %v: %w", result.BlockHash, err)
			}

			task := Request{
				Validators:          validators,
				SignedCommitment:    result.SignedCommitment,
				Proof:               result.MMRProof,
				CurrentAuthoritySet: currentAuthoritySet,
				NextAuthoritySet:    nextAuthoritySet,
				Depth:               result.Depth,
			}

			log.WithFields(log.Fields{
				"commitment": log.Fields{
					"blockNumber":        committedBeefyBlock,
					"validatorSetID":     validatorSetID,
					"nextValidatorSetID": nextValidatorSetID,
				},
				"validatorSetID": currentValidatorSet,
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

func (li *PolkadotListener) queryBeefyAuthoritySet(blockHash types.Hash) (BeefyAuthoritySet, error) {
	var authoritySet BeefyAuthoritySet
	storageKey, err := types.CreateStorageKey(li.conn.Metadata(), "MmrLeaf", "BeefyAuthorities", nil, nil)
	ok, err := li.conn.API().RPC.State.GetStorage(storageKey, &authoritySet, blockHash)
	if err != nil {
		return authoritySet, err
	}
	if !ok {
		return authoritySet, fmt.Errorf("beefy authoritySet not found")
	}

	return authoritySet, nil
}

func (li *PolkadotListener) queryBeefyNextAuthoritySet(blockHash types.Hash) (BeefyAuthoritySet, error) {
	var nextAuthoritySet BeefyAuthoritySet
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
