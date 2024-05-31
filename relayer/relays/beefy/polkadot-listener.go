package beefy

import (
	"context"
	"fmt"

	log "github.com/sirupsen/logrus"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"golang.org/x/sync/errgroup"

	"github.com/snowfork/snowbridge/relayer/chain/relaychain"
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
