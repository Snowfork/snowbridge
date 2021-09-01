package beefy

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/crypto"
	"github.com/sirupsen/logrus"
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
		defer close(li.beefyMessages)
		err := li.subBeefyJustifications(ctx)
		log.WithField("reason", err).Info("Shutting down polkadot listener")
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

			log.WithFields(logrus.Fields{
				"signedCommitment.Commitment.BlockNumber":    signedCommitment.Commitment.BlockNumber,
				"signedCommitment.Commitment.Payload":        signedCommitment.Commitment.Payload.Hex(),
				"signedCommitment.Commitment.ValidatorSetID": signedCommitment.Commitment.ValidatorSetID,
				"signedCommitment.Signatures":                signedCommitment.Signatures,
			}).Info("Witnessed a new BEEFY commitment: ", msg.(string))
			if len(signedCommitment.Signatures) == 0 {
				log.Info("BEEFY commitment has no signatures, skipping...")
				continue
			}

			signedCommitmentBytes, err := json.Marshal(signedCommitment)
			if err != nil {
				log.WithError(err).Error("Failed to marshal signed commitment:", signedCommitment)
				continue
			}

			blockNumber := uint64(signedCommitment.Commitment.BlockNumber)

			beefyAuthorities, err := li.getBeefyAuthorities(blockNumber)
			if err != nil {
				log.WithError(err).Error("Failed to get Beefy authorities from on-chain storage")
				return err
			}

			beefyAuthoritiesBytes, err := json.Marshal(beefyAuthorities)
			if err != nil {
				log.WithError(err).Error("Failed to marshal BEEFY authorities:", beefyAuthorities)
				return err
			}

			blockHash, err := li.relaychainConn.API().RPC.Chain.GetBlockHash(uint64(blockNumber))
			if err != nil {
				log.WithError(err).Error("Failed to get block hash")
				return err
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
				return ctx.Err()
			case li.beefyMessages <- info:
			}
		}
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
