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
	"github.com/snowfork/snowbridge/relayer/crypto/merkle"
	"github.com/snowfork/snowbridge/relayer/substrate"

	log "github.com/sirupsen/logrus"
)

type PolkadotListener struct {
	config         *Config
	relaychainConn *relaychain.Connection
	tasks  chan<- Task
}

func NewPolkadotListener(
	config *Config,
	relaychainConn *relaychain.Connection,
	tasks chan<- Task,
) *PolkadotListener {
	return &PolkadotListener{
		config:         config,
		relaychainConn: relaychainConn,
		tasks:  tasks,
	}
}

func (li *PolkadotListener) Start(ctx context.Context, eg *errgroup.Group, startingBeefyBlock uint64) error {
	eg.Go(func() error {
		defer close(li.tasks)

		err := li.syncBeefyJustifications(ctx, startingBeefyBlock)
		if err != nil {
			if errors.Is(err, context.Canceled) {
				return nil
			}
			return err
		}

		err = li.subBeefyJustifications(ctx)
		log.WithField("reason", err).Info("Shutting down polkadot listener")
		if err != nil && !errors.Is(err, context.Canceled) {
			return err
		}
		return nil
	})
	return nil
}

func (li *PolkadotListener) syncBeefyJustifications(ctx context.Context, latestBeefyBlock uint64) error {
	beefySkipPeriod := li.config.Source.BeefySkipPeriod

	log.WithFields(
		log.Fields{
			"latestBeefyBlock": latestBeefyBlock,
			"beefSkipPeriod":   beefySkipPeriod,
		}).Info("Synchronizing beefy relaychain listener")

	current := latestBeefyBlock
	for {
		finalizedHash, err := li.relaychainConn.API().RPC.Chain.GetFinalizedHead()
		if err != nil {
			log.WithError(err).Error("Failed to fetch finalized head.")
			return err
		}
		finalizedHeader, err := li.relaychainConn.API().RPC.Chain.GetHeader(finalizedHash)
		if err != nil {
			log.WithError(err).WithField("finalizedBlockHash", finalizedHash.Hex()).Error("Failed to fetch header for finalised head.")
			return err
		}

		logFields := log.Fields{
			"blockNumber":          current,
			"finalizedBlockNumber": finalizedHeader.Number}

		finalizedBlockNumber := uint64(finalizedHeader.Number)
		if current > finalizedBlockNumber {
			log.WithFields(logFields).Info("Beefy relaychain listener synchronizing complete.")
			return nil
		}

		log.WithFields(logFields).Info("Probing block.")

		hash, err := li.relaychainConn.API().RPC.Chain.GetBlockHash(current)
		if err != nil {
			log.WithError(err).WithFields(logFields).Error("Failed to fetch block hash.")
			return err
		}
		logFields["blockHash"] = hash.Hex()

		block, err := li.relaychainConn.API().RPC.Chain.GetBlock(hash)
		if err != nil {
			log.WithError(err).WithFields(logFields).Error("Failed to fetch block hash.")
			return err
		}

		commitments := []types.SignedCommitment{}
		for j := range block.Justifications {
			sc := types.OptionalSignedCommitment{}
			if block.Justifications[j].EngineID() == "BEEF" {
				err := types.DecodeFromBytes(block.Justifications[j].Payload(), &sc)
				if err != nil {
					log.WithFields(logFields).WithError(err).Error("Failed to decode BEEFY commitment messages")
					return err
				}
				ok, value := sc.Unwrap()
				if ok {
					commitments = append(commitments, value)
				}
			}
		}

		for c := range commitments {
			log.WithFields(logFields).WithFields(log.Fields{
				"Commitment.BlockNumber":    commitments[c].Commitment.BlockNumber,
				"Commitment.ValidatorSetID": commitments[c].Commitment.ValidatorSetID,
			}).Info("Synchronizing a BEEFY commitment")

			err = li.processBeefyJustifications(ctx, &commitments[c])
			if err != nil {
				return err
			}
		}

		if len(commitments) > 0 {
			log.WithFields(logFields).Info("Justifications found.")
			// The beefy relayer can skip a certain amount of blocks provided it does not skip a whole
			// validator sessions worth of blocks.
			current += beefySkipPeriod
		} else {
			log.WithFields(logFields).Info("Justifications not found.")
			current++
		}
	}
}

func (li *PolkadotListener) subBeefyJustifications(ctx context.Context) error {
	sub, err := li.relaychainConn.API().RPC.Beefy.SubscribeJustifications()
	if err != nil {
		return err
	}
	defer sub.Unsubscribe()

	for {
		select {
		case <-ctx.Done():
			return ctx.Err()
		case sc, ok := <-sub.Chan():
			if !ok {
				return nil
			}

			log.WithFields(log.Fields{
				"Commitment.BlockNumber":    sc.Commitment.BlockNumber,
				"Commitment.ValidatorSetID": sc.Commitment.ValidatorSetID,
			}).Info("Witnessed a new BEEFY commitment")

			err = li.processBeefyJustifications(ctx, &sc)
			if err != nil {
				return err
			}
		}
	}
}

func (li *PolkadotListener) processBeefyJustifications(ctx context.Context, signedCommitment *types.SignedCommitment) error {
	if len(signedCommitment.Signatures) == 0 {
		log.Info("BEEFY commitment has no signatures, skipping...")
		return nil
	}

	blockNumber := uint64(signedCommitment.Commitment.BlockNumber)
	if blockNumber == 1 {
		return nil
	}

	validators, err := li.getBeefyAuthorities(blockNumber)
	if err != nil {
		log.WithError(err).Error("Failed to get Beefy authorities from on-chain storage")
		return err
	}

	blockHash, err := li.relaychainConn.API().RPC.Chain.GetBlockHash(blockNumber)
	if err != nil {
		log.WithError(err).Error("Failed to get block hash")
		return err
	}

	// we can use any block except the latest beefy block
	blockToProve := blockNumber-1
	response, err := li.relaychainConn.GenerateProofForBlock(blockToProve, blockHash, li.config.Source.BeefyActivationBlock)
	if err != nil {
		log.WithFields(log.Fields{
			"blockNumber": blockToProve,
			"latestBeefyBlock": blockHash,
			"beefyActivationBlock": li.config.Source.BeefyActivationBlock}).WithError(err).Error("Failed to generate proof for block")
		return err
	}

	proof, err := merkle.ConvertToSimplifiedMMRProof(response.BlockHash, uint64(response.Proof.LeafIndex),
		response.Leaf, uint64(response.Proof.LeafCount), response.Proof.Items)
	if err != nil {
		log.WithError(err).Error("Failed conversion to simplified proof")
		return err
	}

	task := Task{
		TaskRecord: TaskRecord{Status: CommitmentWitnessed},
		Validators:       validators,
		SignedCommitment: *signedCommitment,
		Proof:            proof,
	}

	select {
	case <-ctx.Done():
		return ctx.Err()
	case li.tasks <- task:
		return nil
	}
}

func (li *PolkadotListener) getBeefyAuthorities(blockNumber uint64) ([]common.Address, error) {
	blockHash, err := li.relaychainConn.API().RPC.Chain.GetBlockHash(blockNumber)
	if err != nil {
		return nil, err
	}

	storageKey, err := types.CreateStorageKey(li.relaychainConn.Metadata(), "Beefy", "Authorities", nil, nil)
	if err != nil {
		return nil, err
	}

	var authorities substrate.Authorities

	ok, err := li.relaychainConn.API().RPC.State.GetStorage(storageKey, &authorities, blockHash)
	if err != nil {
		return nil, err
	}

	if !ok {
		return nil, fmt.Errorf("Beefy authorities not found")
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
