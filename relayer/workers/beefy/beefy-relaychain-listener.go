package beefy

import (
	"context"
	"encoding/json"
	"fmt"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/crypto"
	"github.com/sirupsen/logrus"
	"github.com/snowfork/go-substrate-rpc-client/v3/types"
	"golang.org/x/sync/errgroup"

	"github.com/snowfork/polkadot-ethereum/relayer/chain/relaychain"
	"github.com/snowfork/polkadot-ethereum/relayer/substrate"
	"github.com/snowfork/polkadot-ethereum/relayer/workers/beefy/store"
)

type BeefyRelaychainListener struct {
	relaychainConfig *relaychain.Config
	relaychainConn   *relaychain.Connection
	beefyMessages    chan<- store.BeefyRelayInfo
	log              *logrus.Entry
}

func NewBeefyRelaychainListener(relaychainConfig *relaychain.Config,
	relaychainConn *relaychain.Connection, beefyMessages chan<- store.BeefyRelayInfo,
	log *logrus.Entry) *BeefyRelaychainListener {
	return &BeefyRelaychainListener{
		relaychainConfig: relaychainConfig,
		relaychainConn:   relaychainConn,
		beefyMessages:    beefyMessages,
		log:              log,
	}
}

func (li *BeefyRelaychainListener) Start(ctx context.Context, eg *errgroup.Group) error {

	eg.Go(func() error {
		return li.subBeefyJustifications(ctx)
	})

	return nil
}

func (li *BeefyRelaychainListener) onDone(ctx context.Context) error {
	li.log.Info("Shutting down listener...")
	if li.beefyMessages != nil {
		close(li.beefyMessages)
	}
	return ctx.Err()
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
			return li.onDone(ctx)
		case msg := <-ch:

			signedCommitment := &store.SignedCommitment{}
			err := types.DecodeFromHexString(msg.(string), signedCommitment)
			if err != nil {
				li.log.WithError(err).Error("Failed to decode BEEFY commitment messages")
			}

			li.log.WithFields(logrus.Fields{
				"signedCommitment.Commitment.BlockNumber":    signedCommitment.Commitment.BlockNumber,
				"signedCommitment.Commitment.Payload":        signedCommitment.Commitment.Payload.Hex(),
				"signedCommitment.Commitment.ValidatorSetID": signedCommitment.Commitment.ValidatorSetID,
				"signedCommitment.Signatures":                signedCommitment.Signatures,
			}).Info("Witnessed a new BEEFY commitment: ", msg.(string))
			if len(signedCommitment.Signatures) == 0 {
				li.log.Info("BEEFY commitment has no signatures, skipping...")
				continue
			}

			signedCommitmentBytes, err := json.Marshal(signedCommitment)
			if err != nil {
				li.log.WithError(err).Error("Failed to marshal signed commitment:", signedCommitment)
				continue
			}

			blockNumber := uint64(signedCommitment.Commitment.BlockNumber)

			beefyAuthorities, err := li.getBeefyAuthorities(blockNumber)
			if err != nil {
				li.log.WithError(err).Error("Failed to get Beefy authorities from on-chain storage")
				return err
			}

			beefyAuthoritiesBytes, err := json.Marshal(beefyAuthorities)
			if err != nil {
				li.log.WithError(err).Error("Failed to marshal BEEFY authorities:", beefyAuthorities)
				continue
			}

			blockHash, err := li.relaychainConn.API().RPC.Chain.GetBlockHash(uint64(blockNumber))
			if err != nil {
				li.log.WithError(err).Error("Failed to get block hash")
			}
			li.log.WithField("blockHash", blockHash.Hex()).Info("Got next blockhash")

			latestMMRProof, err := li.relaychainConn.GetMMRLeafForBlock(blockNumber-1, blockHash)
			if err != nil {
				li.log.WithError(err).Error("Failed get MMR Leaf")
				return err
			}
			serializedProof, err := types.EncodeToBytes(latestMMRProof)
			if err != nil {
				li.log.WithError(err).Error("Failed to serialize MMR Proof")
				return err
			}

			info := store.BeefyRelayInfo{
				ValidatorAddresses:       beefyAuthoritiesBytes,
				SignedCommitment:         signedCommitmentBytes,
				Status:                   store.CommitmentWitnessed,
				SerializedLatestMMRProof: serializedProof,
			}
			li.beefyMessages <- info
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
