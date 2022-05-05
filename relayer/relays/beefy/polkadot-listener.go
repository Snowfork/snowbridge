package beefy

import (
	"context"
	"errors"
	"fmt"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/crypto"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"golang.org/x/sync/errgroup"

	"github.com/snowfork/snowbridge/relayer/chain/relaychain"
	"github.com/snowfork/snowbridge/relayer/substrate"

	log "github.com/sirupsen/logrus"
)

type PolkadotListener struct {
	config *Config
	conn   *relaychain.Connection
	tasks  chan<- Task
}

func NewPolkadotListener(
	config *Config,
	conn *relaychain.Connection,
	tasks chan<- Task,
) *PolkadotListener {
	return &PolkadotListener{
		config: config,
		conn:   conn,
		tasks:  tasks,
	}
}

func (li *PolkadotListener) Start(ctx context.Context, eg *errgroup.Group, startingBeefyBlock uint64) error {
	eg.Go(func() error {
		defer close(li.tasks)

		err := li.scanCommitments(ctx, startingBeefyBlock)
		if err != nil {
			if errors.Is(err, context.Canceled) {
				return nil
			}
			return err
		}

		return nil
	})

	return nil
}

func (li *PolkadotListener) scanCommitments(ctx context.Context, latestBeefyBlock uint64) error {
	log.WithFields(log.Fields{
		"latestBeefyBlock": latestBeefyBlock,
	}).Info("Synchronizing beefy relaychain listener")

	in, err := ScanSafeCommitments(ctx, li.conn.Metadata(), li.conn.API(), latestBeefyBlock+1)
	if err != nil {
		return fmt.Errorf("scan safe commitments: %w", err)
	}

	for {
		select {
		case <-ctx.Done():
			return ctx.Err()
		case result, ok := <-in:
			if !ok {
				return nil
			}

			if result.Error != nil {
				return fmt.Errorf("scan safe commitments: %w", result.Error)
			}

			validators, err := li.getBeefyAuthorities(result.BlockHash)
			if err != nil {
				return fmt.Errorf("fetch beefy authorities: %w", err)
			}

			task := Task{
				Validators:       validators,
				SignedCommitment: result.SignedCommitment,
				Proof:            result.MMRProof,
			}

			select {
			case <-ctx.Done():
				return ctx.Err()
			case li.tasks <- task:
			}

		}
	}
}

func (li *PolkadotListener) isNewSession(blockNumber uint64, blockHash types.Hash) (bool, error) {
	var sessionIndex, prevSessionIndex uint32

	sessionIndexKey, err := types.CreateStorageKey(li.conn.Metadata(), "Session", "CurrentIndex", nil, nil)
	if err != nil {
		return false, err
	}

	_, err = li.conn.API().RPC.State.GetStorage(sessionIndexKey, &sessionIndex, blockHash)
	if err != nil {
		return false, err
	}

	prevBlockHash, err := li.conn.API().RPC.Chain.GetBlockHash(blockNumber - 1)
	if err != nil {
		return false, err
	}

	_, err = li.conn.API().RPC.State.GetStorage(sessionIndexKey, &prevSessionIndex, prevBlockHash)
	if err != nil {
		return false, err
	}

	return sessionIndex > prevSessionIndex, nil
}

func (li *PolkadotListener) getBeefyAuthorities(blockHash types.Hash) ([]common.Address, error) {
	storageKey, err := types.CreateStorageKey(li.conn.Metadata(), "Beefy", "Authorities", nil, nil)
	if err != nil {
		return nil, err
	}

	var authorities substrate.Authorities

	ok, err := li.conn.API().RPC.State.GetStorage(storageKey, &authorities, blockHash)
	if err != nil {
		return nil, err
	}

	if !ok {
		return nil, fmt.Errorf("beefy authorities not found")
	}

	// Convert from beefy authorities to ethereum addresses
	var authorityEthereumAddresses []common.Address
	for _, authority := range authorities {
		pub, err := crypto.DecompressPubkey(authority[:])
		if err != nil {
			return nil, err
		}
		ethereumAddress := crypto.PubkeyToAddress(*pub)
		if err != nil {
			return nil, err
		}
		authorityEthereumAddresses = append(authorityEthereumAddresses, ethereumAddress)
	}

	return authorityEthereumAddresses, nil
}
