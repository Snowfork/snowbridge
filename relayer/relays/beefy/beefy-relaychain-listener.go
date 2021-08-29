package beefy

import (
	"context"
	"encoding/json"
	"fmt"

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

	eg.Go(func() error {
		err := li.syncBeefyJustifications(ctx, li.config.Source.Polkadot.BeefyStartingBlock, li.config.Source.SyncBlockNumberJump)
		if err != nil {
			return err
		}
		return li.subBeefyJustifications(ctx)
	})

	return nil
}

func (li *BeefyRelaychainListener) syncBeefyJustifications(ctx context.Context, startBlockNumber, syncBlockNumberJump uint64) error {
	log.WithFields(log.Fields{"StartingBlockNumber": startBlockNumber, "SyncBlockNumberJump": syncBlockNumberJump}).Info("Syncing BEEFY justifications.")

	blockNumber := startBlockNumber + syncBlockNumberJump
	for {
		log.WithField("BlockNumber", blockNumber).Info("Probing block.")
		blockHash, err := li.relaychainConn.API().RPC.Chain.GetBlockHash(blockNumber)
		if err != nil {
			if err.Error() == "required result to be 32 bytes, but got 0" {
				log.WithField("BlockNumber", blockNumber).WithError(err).Info("Block must be the final block.")
				break
			}
			log.WithError(err).WithField("BlockNumber", blockNumber).Error("Failed getting block hash for block.")
			return err
		}

		logFields := log.Fields{"BlockNumber": blockNumber, "BlockHash": blockHash.Hex()}
		log.WithFields(logFields).Info("Fetching block.")

		block, err := li.relaychainConn.API().RPC.Chain.GetBlock(blockHash)
		if err != nil {
			log.WithFields(logFields).WithError(err).Error("Error fetching block.")
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
			log.WithFields(log.Fields{
				"signedCommitment.Commitment.BlockNumber":    commitments[c].Commitment.BlockNumber,
				"signedCommitment.Commitment.Payload":        commitments[c].Commitment.Payload.Hex(),
				"signedCommitment.Commitment.ValidatorSetID": commitments[c].Commitment.ValidatorSetID,
				"signedCommitment.Signatures":                commitments[c].Signatures,
			}).Info("Synchronizing a BEEFY commitment")

			err = li.processBeefyJustifications(ctx, &commitments[c])
			if err != nil {
				log.WithFields(logFields).WithError(err).Error("Failed to synchronise BEEFY commitment.")
				return err
			}
		}

		if len(commitments) > 0 {
			log.WithFields(logFields).Info("Sync complete for block.")
			blockNumber += syncBlockNumberJump
		} else {
			log.WithFields(logFields).Info("BEEFY justifications NOT found for block.")
			blockNumber++
		}
	}

	log.Info("Syncing BEEFY justifications complete. Resuming subcription.")
	return nil
}

func (li *BeefyRelaychainListener) subBeefyJustifications(ctx context.Context) error {
	ch := make(chan interface{})

	sub, err := li.relaychainConn.API().Client.Subscribe(context.Background(), "beefy", "subscribeJustifications", "unsubscribeJustifications", "justifications", ch)
	if err != nil {
		panic(err)
	}
	defer sub.Unsubscribe()

	for {
		select {
		case <-ctx.Done():
			log.WithField("reason", ctx.Err()).Info("Shutting down polkadot listener")
			if li.beefyMessages != nil {
				close(li.beefyMessages)
			}
			return nil
		case msg := <-ch:
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
			}).Info("Witnessed a new BEEFY commitment: ", msg.(string))

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
		log.WithField("SignedCommitment", signedCommitment).WithError(err).Error("Failed to marshal signed commitment.")
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
		log.WithField("BeefyAuthorities", beefyAuthorities).WithError(err).Error("Failed to marshal BEEFY authorities.")
		return nil
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
