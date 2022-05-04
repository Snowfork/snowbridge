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

		// err = li.subscribeBeefyJustifications(ctx)
		// log.WithField("reason", err).Info("Shutting down polkadot listener")
		// if err != nil && !errors.Is(err, context.Canceled) {
		// 	return err
		// }
		return nil
	})
	return nil
}

func (li *PolkadotListener) scanHistoricalBeefyJustifications(ctx context.Context, latestBeefyBlock uint64) error {
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

func (li *PolkadotListener) processBeefyJustifications(ctx context.Context, signedCommitment *types.SignedCommitment) error {
	log.WithFields(log.Fields{
		"commitment": log.Fields{
			"BlockNumber":    signedCommitment.Commitment.BlockNumber,
			"ValidatorSetID": signedCommitment.Commitment.ValidatorSetID,
		},
	}).Info("Witnessed a BEEFY commitment")

	blockNumber := uint64(signedCommitment.Commitment.BlockNumber)
	if blockNumber == 1 {
		return nil
	}

	blockHash, err := li.conn.API().RPC.Chain.GetBlockHash(blockNumber)
	if err != nil {
		return fmt.Errorf("fetch hash for block %v: %w", blockNumber, err)
	}

	validators, err := li.getBeefyAuthorities(blockNumber)
	if err != nil {
		return fmt.Errorf("fetch beefy authorities: %w", err)
	}

	// we can use any block except the latest beefy block
	blockToProve := blockNumber - 1
	proof, err := li.conn.GenerateProofForBlock(blockToProve, blockHash, li.config.Source.BeefyActivationBlock)
	if err != nil {
		return fmt.Errorf("proof generation for %v: %w", blockToProve, err)
	}

	p, err := merkle.ConvertToSimplifiedMMRProof(
		proof.BlockHash,
		uint64(proof.Proof.LeafIndex),
		proof.Leaf,
		uint64(proof.Proof.LeafCount),
		proof.Proof.Items,
	)
	if err != nil {
		return fmt.Errorf("simplified proof conversion for block %v: %w", proof.BlockHash.Hex(), err)
	}

	proofIsValid, err := li.verifyProof(p)
	if err != nil {
		return err
	}

	task := Task{
		Validators:       validators,
		SignedCommitment: *signedCommitment,
		Proof:            p,
		ProofIsValid:     proofIsValid,
	}

	select {
	case <-ctx.Done():
		return ctx.Err()
	case li.tasks <- task:
		return nil
	}
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
