package parachaincommitmentrelayer

import (
	"context"
	"fmt"

	"github.com/sirupsen/logrus"
	rpcOffchain "github.com/snowfork/go-substrate-rpc-client/v2/rpc/offchain"
	"github.com/snowfork/go-substrate-rpc-client/v2/types"
	"golang.org/x/sync/errgroup"

	"github.com/snowfork/polkadot-ethereum/relayer/chain/parachain"
	"github.com/snowfork/polkadot-ethereum/relayer/chain/relaychain"
	chainTypes "github.com/snowfork/polkadot-ethereum/relayer/substrate"
	"github.com/snowfork/polkadot-ethereum/relayer/workers/beefyrelayer/store"
)

// This file is intended to be temporary, just as a way for this worker to watch for beefy commitments
// and get their block number to kickoff the parachain relay process. In a follow up PR, it will be replaced
// with a listener that listens to those proofs once they're confirmed on ethereum, rather than directly on the
// parachain. This can't be done yet, as we still need to add block numbers to the Ethereum proofs being submitted
// to the relay chain light client.

type MessagePackage struct {
	channelID          chainTypes.ChannelID
	commitmentHash     types.H256
	commitmentMessages []chainTypes.CommitmentMessage
	paraHeadProof      string
	mmrProof           types.GenerateMMRProofResponse
}

type BeefyListener struct {
	relaychainConfig    *relaychain.Config
	relaychainConn      *relaychain.Connection
	parachainConnection *parachain.Connection
	messages            chan<- MessagePackage
	log                 *logrus.Entry
}

func NewBeefyListener(
	relaychainConfig *relaychain.Config,
	relaychainConn *relaychain.Connection,
	parachainConnection *parachain.Connection,
	messages chan<- MessagePackage,
	log *logrus.Entry) *BeefyListener {
	return &BeefyListener{
		relaychainConfig:    relaychainConfig,
		relaychainConn:      relaychainConn,
		parachainConnection: parachainConnection,
		messages:            messages,
		log:                 log,
	}
}

func (li *BeefyListener) Start(ctx context.Context, eg *errgroup.Group) error {

	eg.Go(func() error {
		return li.subBeefyJustifications(ctx)
	})

	return nil
}

func (li *BeefyListener) onDone(ctx context.Context) error {
	li.log.Info("Shutting down listener...")
	if li.messages != nil {
		close(li.messages)
	}
	return ctx.Err()
}

func (li *BeefyListener) subBeefyJustifications(ctx context.Context) error {
	ch := make(chan interface{})

	li.log.Info("Subscribing to beefy justifications")
	sub, err := li.relaychainConn.GetAPI().Client.Subscribe(context.Background(), "beefy", "subscribeJustifications", "unsubscribeJustifications", "justifications", ch)
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

			blockNumber := signedCommitment.Commitment.BlockNumber

			li.log.WithField("commitmentBlockNumber", blockNumber).Info("Witnessed a new BEEFY commitment: \n")
			if len(signedCommitment.Signatures) == 0 {
				li.log.Info("BEEFY commitment has no signatures, skipping...")
				continue
			}
			li.log.WithField("blockNumber", blockNumber+1).Info("Getting hash for next block")
			nextBlockHash, err := li.relaychainConn.GetAPI().RPC.Chain.GetBlockHash(uint64(blockNumber + 1))
			if err != nil {
				li.log.WithError(err).Error("Failed to get block hash")
			}
			li.log.WithField("blockHash", nextBlockHash.Hex()).Info("Got blockhash")
			mmrProof := li.GetMMRLeafForBlock(uint64(blockNumber), nextBlockHash)
			allParaHeads := li.GetAllParaheads(nextBlockHash)

			// Todo get our actual head, not 0, and then put it into a types.Header
			ourParaHead := allParaHeads[0]
			ourParaHeadProof := createParachainHeaderProof(allParaHeads, ourParaHead)

			channelID, commitmentHash, commitmentMessages, err := li.extractCommitment(ourParaHead)
			if err != nil {
				li.log.WithError(err).Error("Failed to extract commitment and messages")
			}
			if commitmentMessages == nil {
				li.log.Info("Parachain header has no commitment with messages, skipping...")
				continue
			}

			messagePackage := MessagePackage{
				channelID,
				commitmentHash,
				commitmentMessages,
				ourParaHeadProof,
				mmrProof,
			}

			li.messages <- messagePackage
		}
	}
}

func (li *BeefyListener) GetMMRLeafForBlock(
	blockNumber uint64,
	blockHash types.Hash,
) types.GenerateMMRProofResponse {
	li.log.WithFields(logrus.Fields{
		"blockNumber": blockNumber,
		"blockHash":   blockHash.Hex(),
	}).Info("Getting MMR Leaf for block...")
	proofResponse, err := li.relaychainConn.GetAPI().RPC.MMR.GenerateProof(blockNumber, blockHash)
	if err != nil {
		li.log.WithError(err).Error("Failed to generate mmr proof")
	}

	li.log.WithFields(logrus.Fields{
		"BlockHash":                       proofResponse.BlockHash.Hex(),
		"Leaf.ParentNumber":               proofResponse.Leaf.ParentNumberAndHash.ParentNumber,
		"Leaf.Hash":                       proofResponse.Leaf.ParentNumberAndHash.Hash.Hex(),
		"Leaf.ParachainHeads":             proofResponse.Leaf.ParachainHeads.Hex(),
		"Leaf.BeefyNextAuthoritySet.ID":   proofResponse.Leaf.BeefyNextAuthoritySet.ID,
		"Leaf.BeefyNextAuthoritySet.Len":  proofResponse.Leaf.BeefyNextAuthoritySet.Len,
		"Leaf.BeefyNextAuthoritySet.Root": proofResponse.Leaf.BeefyNextAuthoritySet.Root.Hex(),
		"Proof.LeafIndex":                 proofResponse.Proof.LeafIndex,
		"Proof.LeafCount":                 proofResponse.Proof.LeafCount,
	}).Info("Generated MMR Proof")
	return proofResponse
}

func (li *BeefyListener) GetAllParaheads(blockHash types.Hash) []string {
	none := types.NewOptionU32Empty()
	encoded, err := types.EncodeToBytes(none)
	if err != nil {
		li.log.WithError(err).Error("Error")
	}

	baseParaHeadsStorageKey, err := types.CreateStorageKey(
		li.relaychainConn.GetMetadata(),
		"Paras",
		"Heads", encoded, nil)
	if err != nil {
		li.log.WithError(err).Error("Failed to create parachain header storage key")
	}

	//TODO The above does not give the same base key as polkadotjs needs for getKeys. It has some extra bytes.
	// maybe from the none u32 in golang being wrong, or maybe slightly off CreateStorageKey call? we slice it
	// here as a hack.
	actualBaseParaHeadsStorageKey := baseParaHeadsStorageKey[:32]
	li.log.WithField("actualBaseParaHeadsStorageKey", actualBaseParaHeadsStorageKey.Hex()).Info("actualBaseParaHeadsStorageKey")

	keysResponse, err := li.relaychainConn.GetAPI().RPC.State.GetKeys(actualBaseParaHeadsStorageKey, blockHash)
	if err != nil {
		li.log.WithError(err).Error("Failed to get all parachain keys")
	}

	li.log.WithField("parachainKeys", keysResponse).Info("Got all parachain header keys")

	headersResponse, err := li.relaychainConn.GetAPI().RPC.State.QueryStorage(keysResponse, blockHash, blockHash)
	if err != nil {
		li.log.WithError(err).Error("Failed to get all parachain headers")
	}

	var headers []string

	for _, header := range headersResponse {
		for _, change := range header.Changes {
			// TODO2 - the above query returns some extra bytes on each header, related the the HeadData type (try this state query in polkadotjs
			// webapp for example). These extra bytes I think are for the option or maybe the parachain ID, so the response type needs to account for
			// this properly. the below is just a hack to get the actual header out. It's also not clear to me if the response
			// contains the entire header, or just a hash of the header, or some truncated header? If it's the entire header,
			// then great we can use it entirely instead of querying for it in a follow up call
			header := change.StorageData.Hex()
			actualHeader := fmt.Sprintf("%s%s", "0x", header[6:70])

			li.log.WithFields(logrus.Fields{
				"header.change.StorageKey":  change.StorageKey.Hex(),
				"header.change.StorageData": actualHeader,
			}).Info("Response contains parachain header")

			headers = append(headers, actualHeader)
		}
	}

	return headers
}

func createParachainHeaderProof(allParaHeads []string, ourParaHead string) string {
	//TODO: implement
	return ""
}

func (li *BeefyListener) extractCommitment(header types.Header) (
	chainTypes.ChannelID,
	types.H256,
	[]chainTypes.CommitmentMessage,
	error) {

	li.log.WithFields(logrus.Fields{
		"blockNumber": header.Number,
	}).Debug("Extracting commitment from parachain header")

	digestItem, err := getAuxiliaryDigestItem(header.Digest)
	if err != nil {
		return chainTypes.ChannelID{}, types.H256{}, nil, err
	}

	if digestItem == nil || !digestItem.IsCommitment {
		return chainTypes.ChannelID{}, types.H256{}, nil, nil
	}

	li.log.WithFields(logrus.Fields{
		"block":          header.Number,
		"channelID":      digestItem.AsCommitment.ChannelID,
		"commitmentHash": digestItem.AsCommitment.Hash.Hex(),
	}).Debug("Found commitment hash in header digest")

	channelID := digestItem.AsCommitment.ChannelID
	commitmentHash := digestItem.AsCommitment.Hash
	commitmentMessages, err := li.getMessagesForDigestItem(digestItem)
	if err != nil {
		return chainTypes.ChannelID{}, types.H256{}, nil, err
	}

	return channelID, commitmentHash, commitmentMessages, nil
}

func getAuxiliaryDigestItem(digest types.Digest) (*chainTypes.AuxiliaryDigestItem, error) {
	for _, digestItem := range digest {
		if digestItem.IsOther {
			var auxDigestItem chainTypes.AuxiliaryDigestItem
			err := types.DecodeFromBytes(digestItem.AsOther, &auxDigestItem)
			if err != nil {
				return nil, err
			}
			return &auxDigestItem, nil
		}
	}
	return nil, nil
}

func (li *BeefyListener) getMessagesForDigestItem(digestItem *chainTypes.AuxiliaryDigestItem) ([]chainTypes.CommitmentMessage, error) {
	storageKey, err := parachain.MakeStorageKey(digestItem.AsCommitment.ChannelID, digestItem.AsCommitment.Hash)
	if err != nil {
		return nil, err
	}

	data, err := li.parachainConnection.GetAPI().RPC.Offchain.LocalStorageGet(rpcOffchain.Persistent, storageKey)
	if err != nil {
		li.log.WithError(err).Error("Failed to read commitment from offchain storage")
		return nil, err
	}

	if data != nil {
		li.log.WithFields(logrus.Fields{
			"commitmentSizeBytes": len(*data),
		}).Debug("Retrieved commitment from offchain storage")
	} else {
		li.log.WithError(err).Error("Commitment not found in offchain storage")
		return nil, err
	}

	var messages []chainTypes.CommitmentMessage

	err = types.DecodeFromBytes(*data, &messages)
	if err != nil {
		li.log.WithError(err).Error("Failed to decode commitment messages")
		return nil, err
	}

	return messages, nil
}
