package beefy

import (
	"context"
	"errors"
	"fmt"
	"time"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/crypto"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"golang.org/x/sync/errgroup"

	"github.com/snowfork/snowbridge/relayer/chain/relaychain"
	"github.com/snowfork/snowbridge/relayer/crypto/keccak"
	"github.com/snowfork/snowbridge/relayer/crypto/merkle"
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

		err := li.scanHistoricalBeefyJustifications(ctx, startingBeefyBlock)
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

var ErrFoo = errors.New("Leaf has invalid proof")

func (li *PolkadotListener) scanHistoricalBeefyJustifications(ctx context.Context, latestBeefyBlock uint64) error {
	log.WithFields(log.Fields{
		"latestBeefyBlock": latestBeefyBlock,
	}).
		Info("Synchronizing beefy relaychain listener")

	sessionIndexKey, err := types.CreateStorageKey(li.conn.Metadata(), "Session", "CurrentIndex", nil, nil)
	if err != nil {
		return err
	}

	blockHash, err := li.conn.API().RPC.Chain.GetBlockHash(latestBeefyBlock)
	if err != nil {
		log.WithError(err).Error("Failed to fetch block hash")
		return err
	}

	var lastSessionIndex uint32

	_, err = li.conn.API().RPC.State.GetStorage(sessionIndexKey, &lastSessionIndex, blockHash)
	if err != nil {
		return err
	}

	current := latestBeefyBlock + 1
	for {
		finalizedHash, err := li.conn.API().RPC.Beefy.GetFinalizedHead()
		if err != nil {
			return fmt.Errorf("fetch finalized head: %w", err)
		}

		finalizedHeader, err := li.conn.API().RPC.Chain.GetHeader(finalizedHash)
		if err != nil {
			return fmt.Errorf("fetch header for finalised head %v: %w", finalizedHash.Hex(), err)
		}

		finalizedBlockNumber := uint64(finalizedHeader.Number)
		if current > finalizedBlockNumber {
			select {
			case <-ctx.Done():
				return ctx.Err()
			case <-time.After(2 * time.Second):
			}
			continue
		}

		log.WithField("block", current).Info("Probing block")

		blockHash, err := li.conn.API().RPC.Chain.GetBlockHash(current)
		if err != nil {
			return fmt.Errorf("fetch block hash: %w", err)
		}

		var sessionIndex uint32

		_, err = li.conn.API().RPC.State.GetStorage(sessionIndexKey, &sessionIndex, blockHash)
		if err != nil {
			return err
		}

		if sessionIndex == lastSessionIndex {
			current++
			continue
		}

		// This block starts a new session and always contains a BEEFY justification
		log.WithField("block", current).Info("New session detected")

		block, err := li.conn.API().RPC.Chain.GetBlock(blockHash)
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
			current++
			continue
		}

		var task *Task

		task, err = li.processBeefyJustifications(ctx, commitment)
		if err != nil {
			if errors.Is(err, ErrFoo) {
				task, err = li.scanNextValidInSession(ctx, sessionIndex, current+1)
				if err != nil {
					return fmt.Errorf("scan next valid in session: %w", err)
				}
			} else {
				return err
			}
		}

		if task == nil {
			current++
			continue
		}

		select {
		case <-ctx.Done():
			return ctx.Err()
		case li.tasks <- *task:
		}

		current++
	}
}

func (li *PolkadotListener) scanNextValidInSession(
	ctx context.Context,
	sessionIndex uint32,
	sessionInitialBlock uint64,
) (*Task, error) {
	sessionIndexKey, err := types.CreateStorageKey(li.conn.Metadata(), "Session", "CurrentIndex", nil, nil)
	if err != nil {
		return nil, err
	}

	current := sessionInitialBlock + 1
	for {
		finalizedHash, err := li.conn.API().RPC.Beefy.GetFinalizedHead()
		if err != nil {
			return nil, fmt.Errorf("fetch finalized head: %w", err)
		}

		finalizedHeader, err := li.conn.API().RPC.Chain.GetHeader(finalizedHash)
		if err != nil {
			return nil, fmt.Errorf("fetch header for finalised head %v: %w", finalizedHash.Hex(), err)
		}

		finalizedBlockNumber := uint64(finalizedHeader.Number)
		if current > finalizedBlockNumber {
			select {
			case <-ctx.Done():
				return nil, ctx.Err()
			case <-time.After(2 * time.Second):
			}
			continue
		}

		log.WithField("block", current).Info("Probing block")

		blockHash, err := li.conn.API().RPC.Chain.GetBlockHash(current)
		if err != nil {
			return nil, fmt.Errorf("fetch block hash: %w", err)
		}

		var currentSessionIndex uint32

		_, err = li.conn.API().RPC.State.GetStorage(sessionIndexKey, &sessionIndex, blockHash)
		if err != nil {
			return nil, err
		}

		if currentSessionIndex > sessionIndex {
			return nil, nil
		}

		block, err := li.conn.API().RPC.Chain.GetBlock(blockHash)
		if err != nil {
			return nil, fmt.Errorf("fetch block: %w", err)
		}

		var commitment *types.SignedCommitment

		for j := range block.Justifications {
			sc := types.OptionalSignedCommitment{}
			if block.Justifications[j].EngineID() == "BEEF" {
				err := types.DecodeFromBytes(block.Justifications[j].Payload(), &sc)
				if err != nil {
					return nil, fmt.Errorf("decode BEEFY signed commitment: %w", err)
				}
				ok, value := sc.Unwrap()
				if ok {
					commitment = &value
				}
			}
		}

		if commitment == nil {
			current++
			continue
		}

		task, err := li.processBeefyJustifications(ctx, commitment)
		if err != nil {
			if errors.Is(err, ErrFoo) {
				current++
				continue
			} else {
				return nil, err
			}
		}

		if task == nil {
			current++
			continue
		}

		current++
	}
}

func (li *PolkadotListener) scanBeefyJustifications(ctx context.Context, latestBeefyBlock uint64) error {
	log.WithFields(log.Fields{
		"latestBeefyBlock": latestBeefyBlock,
	}).
		Info("Synchronizing beefy relaychain listener")

	current := latestBeefyBlock + 1
	for {
		finalizedHash, err := li.conn.API().RPC.Beefy.GetFinalizedHead()
		if err != nil {
			return fmt.Errorf("fetch finalized head: %w", err)
		}

		finalizedHeader, err := li.conn.API().RPC.Chain.GetHeader(finalizedHash)
		if err != nil {
			return fmt.Errorf("fetch header for finalised head %v: %w", finalizedHash.Hex(), err)
		}

		finalizedBlockNumber := uint64(finalizedHeader.Number)
		if current > finalizedBlockNumber {
			select {
			case <-ctx.Done():
				return ctx.Err()
			case <-time.After(2 * time.Second):
			}
			continue
		}

		log.WithField("block", current).Info("Probing block")

		blockHash, err := li.conn.API().RPC.Chain.GetBlockHash(current)
		if err != nil {
			return fmt.Errorf("fetch block hash: %w", err)
		}

		block, err := li.conn.API().RPC.Chain.GetBlock(blockHash)
		if err != nil {
			return fmt.Errorf("fetch block: %w", err)
		}

		commitments := []types.SignedCommitment{}
		for j := range block.Justifications {
			sc := types.OptionalSignedCommitment{}
			if block.Justifications[j].EngineID() == "BEEF" {
				err := types.DecodeFromBytes(block.Justifications[j].Payload(), &sc)
				if err != nil {
					return fmt.Errorf("decode BEEFY signed commitment: %w", err)
				}
				ok, value := sc.Unwrap()
				if ok {
					commitments = append(commitments, value)
				}
			}
		}

		for c := range commitments {
			err = li.processBeefyJustifications(ctx, &commitments[c])
			if err != nil {
				return err
			}
		}

		current++
	}
}

func (li *PolkadotListener) verifyProof(proof merkle.SimplifiedMMRProof) (bool, error) {
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

	mmrRootKey, err := types.CreateStorageKey(li.conn.Metadata(), "Mmr", "RootHash", nil, nil)
	if err != nil {
		return false, err
	}

	_, err = li.conn.API().RPC.State.GetStorage(mmrRootKey, &expectedRoot, types.Hash(proof.Blockhash))
	if err != nil {
		return false, err
	}

	return actualRoot == expectedRoot, nil
}

func (li *PolkadotListener) processBeefyJustifications(ctx context.Context, signedCommitment *types.SignedCommitment) (*Task, error) {
	log.WithFields(log.Fields{
		"commitment": log.Fields{
			"BlockNumber":    signedCommitment.Commitment.BlockNumber,
			"ValidatorSetID": signedCommitment.Commitment.ValidatorSetID,
		},
	}).Info("Witnessed a BEEFY commitment")

	blockNumber := uint64(signedCommitment.Commitment.BlockNumber)
	if blockNumber == 1 {
		return nil, nil
	}

	blockHash, err := li.conn.API().RPC.Chain.GetBlockHash(blockNumber)
	if err != nil {
		return nil, fmt.Errorf("fetch hash for block %v: %w", blockNumber, err)
	}

	validators, err := li.getBeefyAuthorities(blockNumber)
	if err != nil {
		return nil, fmt.Errorf("fetch beefy authorities: %w", err)
	}

	// we can use any block except the latest beefy block
	blockToProve := blockNumber - 1
	proof, err := li.conn.GenerateProofForBlock(blockToProve, blockHash, li.config.Source.BeefyActivationBlock)
	if err != nil {
		return nil, fmt.Errorf("proof generation for %v: %w", blockToProve, err)
	}

	p, err := merkle.ConvertToSimplifiedMMRProof(
		proof.BlockHash,
		uint64(proof.Proof.LeafIndex),
		proof.Leaf,
		uint64(proof.Proof.LeafCount),
		proof.Proof.Items,
	)
	if err != nil {
		return nil, fmt.Errorf("simplified proof conversion for block %v: %w", proof.BlockHash.Hex(), err)
	}

	proofIsValid, err := li.verifyProof(p)
	if err != nil {
		return nil, err
	}

	if !proofIsValid {
		return nil, ErrFoo

	}

	task := Task{
		Validators:       validators,
		SignedCommitment: *signedCommitment,
		Proof:            p,
		ProofIsValid:     proofIsValid,
	}

	return &task, nil
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

func (li *PolkadotListener) getBeefyAuthorities(blockNumber uint64) ([]common.Address, error) {
	blockHash, err := li.conn.API().RPC.Chain.GetBlockHash(blockNumber)
	if err != nil {
		return nil, err
	}

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
