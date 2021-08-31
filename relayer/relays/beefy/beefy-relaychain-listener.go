package beefy

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"time"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/crypto"
	"github.com/snowfork/go-substrate-rpc-client/v3/types"
	"golang.org/x/sync/errgroup"

	"github.com/snowfork/snowbridge/relayer/chain/relaychain"
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

func (li *BeefyRelaychainListener) Start(ctx context.Context, eg *errgroup.Group) error {
	startingBlock := li.config.Source.Polkadot.BeefyStartingBlock
	pollSkipBlockCount := li.config.Source.PollSkipBlockCount
	pollInterval := li.config.Source.PollIntervalSeconds

	log.WithFields(
		log.Fields{
			"startingBlock":      startingBlock,
			"pollSkipBlockCount": pollSkipBlockCount,
			"pollInterval":       pollInterval}).Info("Starting beefy relaychain listener")

	if pollSkipBlockCount < 1 {
		return errors.New("poll skip block count must be greater than 0")
	}

	commitments := make(chan store.SignedCommitment)
	eg.Go(func() error {
		defer close(commitments)
		return li.produceSignedCommitments(ctx, commitments, startingBlock, pollInterval, pollSkipBlockCount)
	})

	eg.Go(func() error {
		return li.consumeSignedCommitments(ctx, commitments)
	})

	return nil
}

func (li *BeefyRelaychainListener) produceSignedCommitments(ctx context.Context, beefyCommitments chan<- store.SignedCommitment, pollStartingBlock, pollInterval, pollSkipBlockCount uint64) error {
	current := pollStartingBlock
	pollDuration := time.Duration(pollInterval) * time.Second
	syncComplete := false
	for {
		select {
		case <-ctx.Done():
			log.WithError(ctx.Err()).Error("Shutting down beefy relaychain commitment producer.")
			return ctx.Err()
		default:
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

			finalizedBlockNumber := uint64(finalizedHeader.Number)
			if current > finalizedBlockNumber {
				log.WithFields(log.Fields{
					"blockNumber":          current,
					"finalizedBlockNumber": finalizedHeader.Number,
					"syncComplete":         syncComplete}).Info("Current block is not finalized.")
				if !syncComplete {
					syncComplete = true
					current = finalizedBlockNumber
				} else {
					sleep(ctx, pollDuration)
					continue
				}
			}

			logFields := log.Fields{
				"blockNumber":          current,
				"finalizedBlockNumber": finalizedHeader.Number,
				"syncComplete":         syncComplete}
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
				}).Info("Witnessed a BEEFY commitment")

				select {
				case <-ctx.Done():
					log.WithError(ctx.Err()).Error("Shutting down beefy relaychain commitment producer.")
					return ctx.Err()
				case beefyCommitments <- commitments[c]:
				}
			}

			if len(commitments) > 0 {
				log.WithFields(logFields).Info("Justifications found.")
				if !syncComplete {
					// The beefy relayer can skip a certain amount of blocks provided it does not skip a whole
					// validator sessions worth of blocks.
					current += pollSkipBlockCount
				} else {
					current += 1
				}
			} else {
				log.WithFields(logFields).Info("Justifications not found.")
				current += 1
			}

		}
	}
}

func sleep(ctx context.Context, delay time.Duration) {
	select {
	case <-ctx.Done():
	case <-time.After(delay):
	}
}

func (li *BeefyRelaychainListener) consumeSignedCommitments(ctx context.Context, beefyCommitments <-chan store.SignedCommitment) error {
	for {
		select {
		case <-ctx.Done():
			log.WithError(ctx.Err()).Error("Shutting down beefy relaychain commitment consumer.")
			return ctx.Err()
		case commitment := <-beefyCommitments:
			err := li.processBeefyJustifications(ctx, &commitment)
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
		log.WithField("SignedCommitment", signedCommitment).WithError(err).Error("Failed to marshal signed commitment.")
		return err
	}

	blockNumber := uint64(signedCommitment.Commitment.BlockNumber)

	beefyAuthorities, err := li.getBeefyAuthorities(blockNumber)
	if err != nil {
		log.WithError(err).Error("Failed to get Beefy authorities from on-chain storage")
		return err
	}

	beefyAuthoritiesBytes, err := json.Marshal(beefyAuthorities)
	if err != nil {
		log.WithField("BeefyAuthorities", beefyAuthorities).WithError(err).Error("Failed to marshal BEEFY authorities.")
		return err
	}

	blockHash, err := li.relaychainConn.API().RPC.Chain.GetBlockHash(uint64(blockNumber))
	if err != nil {
		log.WithError(err).Error("Failed to get block hash")
	}
	log.WithField("blockHash", blockHash.Hex()).Info("Got next blockhash")

	latestMMRProof, err := li.relaychainConn.GetMMRLeafForBlock(blockNumber-1, blockHash, li.config.Source.Polkadot.BeefyStartingBlock)
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

	serializedProof, err := types.EncodeToBytes(latestMMRProof)
	if err != nil {
		log.WithError(err).Error("Failed to serialize MMR Proof")
		return err
	}
	log.WithField("latestMMRProof", latestMMRProof.Leaf.Version).Info("Got latestMMRProof")

	info := store.BeefyRelayInfo{
		ValidatorAddresses:       beefyAuthoritiesBytes,
		SignedCommitment:         signedCommitmentBytes,
		Status:                   store.CommitmentWitnessed,
		SerializedLatestMMRProof: serializedProof,
		MMRLeafCount:             mmrLeafCount,
	}

	select {
	case <-ctx.Done():
		log.WithError(ctx.Err()).Error("Relayer halting.")
		if li.beefyMessages != nil {
			close(li.beefyMessages)
		}
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
