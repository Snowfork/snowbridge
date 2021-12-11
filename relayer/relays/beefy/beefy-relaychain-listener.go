package beefy

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/crypto"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"golang.org/x/sync/errgroup"

	"github.com/snowfork/snowbridge/relayer/chain/relaychain"
	"github.com/snowfork/snowbridge/relayer/crypto/merkle"
	"github.com/snowfork/snowbridge/relayer/relays/beefy/store"
	"github.com/snowfork/snowbridge/relayer/substrate"

	log "github.com/sirupsen/logrus"
)

type BeefyRelaychainListener struct {
	config         *Config
	relaychainConn *relaychain.Connection
	beefyMessages  chan<- store.BeefyRelayInfo
}

func NewBeefyRelaychainListener(
	config *Config,
	relaychainConn *relaychain.Connection,
	beefyMessages chan<- store.BeefyRelayInfo,
) *BeefyRelaychainListener {
	return &BeefyRelaychainListener{
		config:         config,
		relaychainConn: relaychainConn,
		beefyMessages:  beefyMessages,
	}
}

func (li *BeefyRelaychainListener) Start(ctx context.Context, eg *errgroup.Group, startingBeefyBlock uint64) error {
	eg.Go(func() error {
		defer close(li.beefyMessages)

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

func (li *BeefyRelaychainListener) syncBeefyJustifications(ctx context.Context, latestBeefyBlock uint64) error {
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

		commitments := []store.SignedCommitment{}
		for j := range block.Justifications {
			sc := store.OptionalSignedCommitment{}
			if block.Justifications[j].EngineID() == "BEEF" {
				err := types.DecodeFromBytes(block.Justifications[j].Payload(), &sc)
				if err != nil {
					log.WithFields(logFields).WithError(err).Error("Failed to decode BEEFY commitment messages")
				} else if sc.IsSome() {
					commitments = append(commitments, sc.Value)
				}
			}
		}

		for c := range commitments {
			log.WithFields(logFields).WithFields(log.Fields{
				"signedCommitment.Commitment.BlockNumber":    commitments[c].Commitment.BlockNumber,
				"signedCommitment.Commitment.Payload":        commitments[c].Commitment.Payload.Hex(),
				"signedCommitment.Commitment.ValidatorSetID": commitments[c].Commitment.ValidatorSetID,
				"signedCommitment.Signatures":                commitments[c].Signatures,
			}).Info("Synchronizing a BEEFY commitment.")

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
			current += 1
		}
	}
}

func (li *BeefyRelaychainListener) subBeefyJustifications(ctx context.Context) error {
	ch := make(chan interface{})

	sub, err := li.relaychainConn.API().Client.Subscribe(
		context.Background(),
		"beefy",
		"subscribeJustifications",
		"unsubscribeJustifications",
		"justifications",
		ch,
	)
	if err != nil {
		return err
	}
	defer sub.Unsubscribe()

	for {
		select {
		case <-ctx.Done():
			return ctx.Err()
		case msg, ok := <-ch:
			if !ok {
				return nil
			}

			signedCommitment := &store.SignedCommitment{}
			err := types.DecodeFromHexString(msg.(string), signedCommitment)
			if err != nil {
				log.WithError(err).Error("Failed to decode BEEFY commitment messages")
			}

			log.WithFields(log.Fields{
				"signedCommitment.Commitment.BlockNumber":    signedCommitment.Commitment.BlockNumber,
				"signedCommitment.Commitment.Payload":        signedCommitment.Commitment.Payload.Hex(),
				"signedCommitment.Commitment.ValidatorSetID": signedCommitment.Commitment.ValidatorSetID,
				"signedCommitment.Signatures":                signedCommitment.Signatures,
				"rawMessage":                                 msg.(string),
			}).Info("Witnessed a new BEEFY commitment.")

			err = li.processBeefyJustifications(ctx, signedCommitment)
			if err != nil {
				return err
			}
		}
	}
}

func (li *BeefyRelaychainListener) processBeefyJustifications(ctx context.Context, signedCommitment *store.SignedCommitment) error {
	if len(signedCommitment.Signatures) == 0 {
		log.Info("BEEFY commitment has no signatures, skipping...")
		return nil
	}

	signedCommitmentBytes, err := json.Marshal(signedCommitment)
	if err != nil {
		log.WithField("signedCommitment", signedCommitment).WithError(err).Error("Failed to marshal signed commitment.")
		return nil
	}

	blockNumber := uint64(signedCommitment.Commitment.BlockNumber)

	beefyAuthorities, err := li.getBeefyAuthorities(blockNumber)
	if err != nil {
		log.WithError(err).Error("Failed to get Beefy authorities from on-chain storage")
		return err
	}

	beefyAuthoritiesBytes, err := json.Marshal(beefyAuthorities)
	if err != nil {
		log.WithField("beefyAuthorities", beefyAuthorities).WithError(err).Error("Failed to marshal BEEFY authorities.")
		return err
	}

	blockHash, err := li.relaychainConn.API().RPC.Chain.GetBlockHash(uint64(blockNumber))
	if err != nil {
		log.WithError(err).Error("Failed to get block hash")
		return err
	}
	log.WithField("blockHash", blockHash.Hex()).Info("Got next blockhash")

	latestMMRProof, err := li.relaychainConn.GenerateProofForBlock(blockNumber, blockHash, li.config.Source.BeefyActivationBlock)
	if err != nil {
		log.WithError(err).Error("Failed get MMR Leaf")
		return err
	}

	mmrLeafCount, err := li.relaychainConn.FetchMMRLeafCount(blockHash)
	if err != nil {
		log.WithError(err).Error("Failed get MMR Leaf Count")
		return err
	}

	if mmrLeafCount == 0 {
		err := fmt.Errorf("MMR is empty and has no leaves")
		log.WithError(err)
		return err
	}

	log.WithField("latestMMRProof", latestMMRProof.Leaf.Version).Info("Got latestMMRProof")

	simplifiedProof, err := merkle.ConvertToSimplifiedMMRProof(latestMMRProof.BlockHash, uint64(latestMMRProof.Proof.LeafIndex), latestMMRProof.Leaf, uint64(latestMMRProof.Proof.LeafCount), latestMMRProof.Proof.Items)
	log.WithField("simplifiedProof", simplifiedProof).Info("Converted latestMMRProof to simplified proof")

	serializedProof, err := types.EncodeToBytes(simplifiedProof)
	if err != nil {
		log.WithError(err).Error("Failed to serialize MMR Proof")
		return err
	}

	info := store.BeefyRelayInfo{
		ValidatorAddresses:       beefyAuthoritiesBytes,
		SignedCommitment:         signedCommitmentBytes,
		Status:                   store.CommitmentWitnessed,
		SerializedLatestMMRProof: serializedProof,
		MMRLeafCount:             mmrLeafCount,
	}

	select {
	case <-ctx.Done():
		return ctx.Err()
	case li.beefyMessages <- info:
		return nil
	}
}

func (li *BeefyRelaychainListener) getBeefyAuthorities(blockNumber uint64) ([]common.Address, error) {
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
