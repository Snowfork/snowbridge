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

	"github.com/sirupsen/logrus"
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

		err = li.subscribeBeefyJustifications(ctx)
		log.WithField("reason", err).Info("Shutting down polkadot listener")
		if err != nil && !errors.Is(err, context.Canceled) {
			return err
		}
		return nil
	})
	return nil
}

func (li *PolkadotListener) scanHistoricalBeefyJustifications(ctx context.Context, latestBeefyBlock uint64) error {
	log.WithFields(
		log.Fields{
			"latestBeefyBlock": latestBeefyBlock,
		}).Info("Synchronizing beefy relaychain listener")

	storageKey, err := types.CreateStorageKey(li.conn.Metadata(), "Session", "CurrentIndex", nil, nil)
	if err != nil {
		return err
	}

	blockHash, err := li.conn.API().RPC.Chain.GetBlockHash(latestBeefyBlock)
	if err != nil {
		log.WithError(err).Error("Failed to fetch block hash")
		return err
	}

	var lastSessionIndex uint32

	_, err = li.conn.API().RPC.State.GetStorage(storageKey, &lastSessionIndex, blockHash)
	if err != nil {
		return err
	}

	current := latestBeefyBlock
	for {
		log.WithField("block", current).Info("Probing block for new session")

		finalizedHash, err := li.conn.API().RPC.Chain.GetFinalizedHead()
		if err != nil {
			log.WithError(err).Error("Failed to fetch finalized head")
			return err
		}

		finalizedHeader, err := li.conn.API().RPC.Chain.GetHeader(finalizedHash)
		if err != nil {
			log.WithError(err).WithField("finalizedBlockHash", finalizedHash.Hex()).Error("Failed to fetch header for finalised head")
			return err
		}

		finalizedBlockNumber := uint64(finalizedHeader.Number)
		if current > finalizedBlockNumber {
			log.Info("Synchronization complete")
			return nil
		}

		blockHash, err := li.conn.API().RPC.Chain.GetBlockHash(current)
		if err != nil {
			log.WithError(err).Error("Failed to fetch block hash")
			return err
		}

		var sessionIndex uint32

		_, err = li.conn.API().RPC.State.GetStorage(storageKey, &sessionIndex, blockHash)
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
			log.WithError(err).Error("failed to fetch block hash")
			return err
		}

		commitments := []types.SignedCommitment{}
		for j := range block.Justifications {
			sc := types.OptionalSignedCommitment{}
			if block.Justifications[j].EngineID() == "BEEF" {
				err := types.DecodeFromBytes(block.Justifications[j].Payload(), &sc)
				if err != nil {
					log.WithError(err).Error("Failed to decode BEEFY signed commitment")
					return err
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

		lastSessionIndex = sessionIndex
		current++
	}
}

func (li *PolkadotListener) subscribeBeefyJustifications(ctx context.Context) error {
	sub, err := li.conn.API().RPC.Beefy.SubscribeJustifications()
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
			err = li.processBeefyJustifications(ctx, &sc)
			if err != nil {
				return err
			}
		}
	}
}

func (li *PolkadotListener) processBeefyJustifications(ctx context.Context, signedCommitment *types.SignedCommitment) error {
	log.WithFields(log.Fields{
		"commitment": logrus.Fields{
			"BlockNumber":    signedCommitment.Commitment.BlockNumber,
			"ValidatorSetID": signedCommitment.Commitment.ValidatorSetID,
		},
	}).Info("Witnessed a BEEFY commitment")

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

	blockHash, err := li.conn.API().RPC.Chain.GetBlockHash(blockNumber)
	if err != nil {
		log.WithError(err).Error("Failed to get block hash")
		return err
	}

	// we can use any block except the latest beefy block
	blockToProve := blockNumber - 1
	response, err := li.conn.GenerateProofForBlock(blockToProve, blockHash, li.config.Source.BeefyActivationBlock)
	if err != nil {
		log.WithFields(log.Fields{
			"blockNumber":          blockToProve,
			"latestBeefyBlock":     blockHash,
			"beefyActivationBlock": li.config.Source.BeefyActivationBlock},
		).WithError(err).Error("Failed to generate proof for block")
		return err
	}

	proof, err := merkle.ConvertToSimplifiedMMRProof(response.BlockHash, uint64(response.Proof.LeafIndex),
		response.Leaf, uint64(response.Proof.LeafCount), response.Proof.Items)
	if err != nil {
		log.WithError(err).Error("Failed conversion to simplified proof")
		return err
	}

	task := Task{
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
