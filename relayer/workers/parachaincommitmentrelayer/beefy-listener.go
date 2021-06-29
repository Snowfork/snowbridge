package parachaincommitmentrelayer

import (
	"context"
	"fmt"

	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"
	gethTypes "github.com/ethereum/go-ethereum/core/types"
	"github.com/sirupsen/logrus"
	rpcOffchain "github.com/snowfork/go-substrate-rpc-client/v2/rpc/offchain"
	"github.com/snowfork/go-substrate-rpc-client/v2/types"
	"golang.org/x/sync/errgroup"

	"github.com/snowfork/polkadot-ethereum/relayer/chain/ethereum"
	"github.com/snowfork/polkadot-ethereum/relayer/chain/parachain"
	"github.com/snowfork/polkadot-ethereum/relayer/chain/relaychain"
	"github.com/snowfork/polkadot-ethereum/relayer/contracts/beefylightclient"
	chainTypes "github.com/snowfork/polkadot-ethereum/relayer/substrate"
)

//TODO - put in config
const OUR_PARACHAIN_ID = 200

type MessagePackage struct {
	channelID      chainTypes.ChannelID
	commitmentHash types.H256
	commitmentData types.StorageDataRaw
	paraHead       types.Header
	paraHeadProof  string
	mmrProof       types.GenerateMMRProofResponse
}

type BeefyListener struct {
	ethereumConfig      *ethereum.Config
	ethereumConn        *ethereum.Connection
	beefyLightClient    *beefylightclient.Contract
	relaychainConn      *relaychain.Connection
	parachainConnection *parachain.Connection
	messages            chan<- MessagePackage
	log                 *logrus.Entry
}

func NewBeefyListener(
	ethereumConfig *ethereum.Config,
	ethereumConn *ethereum.Connection,
	relaychainConn *relaychain.Connection,
	parachainConnection *parachain.Connection,
	messages chan<- MessagePackage,
	log *logrus.Entry) *BeefyListener {
	return &BeefyListener{
		ethereumConfig:      ethereumConfig,
		ethereumConn:        ethereumConn,
		relaychainConn:      relaychainConn,
		parachainConnection: parachainConnection,
		messages:            messages,
		log:                 log,
	}
}

func (li *BeefyListener) Start(ctx context.Context, eg *errgroup.Group) error {

	// Set up light client bridge contract
	beefyLightClientContract, err := beefylightclient.NewContract(common.HexToAddress(li.ethereumConfig.BeefyLightClient), li.ethereumConn.GetClient())
	if err != nil {
		return err
	}
	li.beefyLightClient = beefyLightClientContract

	eg.Go(func() error {
		err := li.subBeefyJustifications(ctx)
		return err
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
	headers := make(chan *gethTypes.Header, 5)

	li.ethereumConn.GetClient().SubscribeNewHead(ctx, headers)

	for {
		select {
		case <-ctx.Done():
			return li.onDone(ctx)
		case gethheader := <-headers:
			// Query LightClientBridge contract's ContractNewMMRRoot events
			blockNumber := gethheader.Number.Uint64()
			var beefyLightClientEvents []*beefylightclient.ContractNewMMRRoot

			contractEvents, err := li.queryBeefyLightClientEvents(ctx, blockNumber, &blockNumber)
			if err != nil {
				li.log.WithError(err).Error("Failure fetching event logs")
				return err
			}
			beefyLightClientEvents = append(beefyLightClientEvents, contractEvents...)

			if len(beefyLightClientEvents) > 0 {
				li.log.Info(fmt.Sprintf("Found %d BeefyLightClient ContractNewMMRRoot events on block %d", len(beefyLightClientEvents), blockNumber))
			}
			li.processBeefyLightClientEvents(ctx, beefyLightClientEvents)
		}
	}
}

// processLightClientEvents matches events to BEEFY commitment info by transaction hash
func (li *BeefyListener) processBeefyLightClientEvents(ctx context.Context, events []*beefylightclient.ContractNewMMRRoot) error {
	for _, event := range events {

		relayChainBlockNumber := event.BlockNumber

		li.log.WithFields(logrus.Fields{
			"relayChainBlockNumber": relayChainBlockNumber,
			"ethereumBlockNumber":   event.Raw.BlockNumber,
			"ethereumTxHash":        event.Raw.TxHash.Hex(),
		}).Info("Witnessed a new MMRRoot event")

		li.log.WithField("blockNumber", relayChainBlockNumber).Info("Getting hash for relay chain block")
		blockHash, err := li.relaychainConn.GetAPI().RPC.Chain.GetBlockHash(uint64(relayChainBlockNumber))
		if err != nil {
			li.log.WithError(err).Error("Failed to get block hash")
			return err
		}
		li.log.WithField("blockHash", blockHash.Hex()).Info("Got blockhash")

		// TODO this just queries the latest MMR leaf in the latest MMR and our latest parahead from the relaychain.
		// we should ideally be querying the latest and last few leaves in the latest MMR until we find
		// the first parachain block that has not yet been fully processed on ethereum,
		// and then package and relay all newer heads/commitments together with their corresponding leaf
		mmrProof := li.relaychainConn.GetMMRLeafForBlock(uint64(relayChainBlockNumber-1), blockHash)
		allParaHeads, ourParaHead := li.GetAllParaheads(blockHash, OUR_PARACHAIN_ID)

		ourParaHeadProof := createParachainHeaderProof(allParaHeads, ourParaHead)

		messagePackets, err := li.extractCommitments(ourParaHead, mmrProof, ourParaHeadProof)
		if err != nil {
			li.log.WithError(err).Error("Failed to extract commitment and messages")
			return err
		}
		if len(messagePackets) == 0 {
			li.log.Info("Parachain header has no commitment with messages, skipping...")
			continue
		}
		for _, messagePacket := range messagePackets {
			li.log.WithFields(logrus.Fields{
				"channelID":        messagePacket.channelID,
				"commitmentHash":   messagePacket.commitmentHash,
				"commitmentData":   messagePacket.commitmentData,
				"ourParaHeadProof": messagePacket.paraHeadProof,
				"mmrProof":         messagePacket.mmrProof,
			}).Info("Beefy Listener emitted new message packet")

			li.messages <- messagePacket
		}

	}
	return nil
}

// queryBeefyLightClientEvents queries ContractNewMMRRoot events from the BeefyLightClient contract
func (li *BeefyListener) queryBeefyLightClientEvents(ctx context.Context, start uint64,
	end *uint64) ([]*beefylightclient.ContractNewMMRRoot, error) {
	var events []*beefylightclient.ContractNewMMRRoot
	filterOps := bind.FilterOpts{Start: start, End: end, Context: ctx}

	iter, err := li.beefyLightClient.FilterNewMMRRoot(&filterOps)
	if err != nil {
		return nil, err
	}

	for {
		more := iter.Next()
		if !more {
			err = iter.Error()
			if err != nil {
				return nil, err
			}
			break
		}

		events = append(events, iter.Event)
	}

	return events, nil
}

func (li *BeefyListener) GetAllParaheads(blockHash types.Hash, ourParachainId uint32) ([]types.Header, types.Header) {
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

	//TODO fix this manual slice.
	// The above types.CreateStorageKey does not give the same base key as polkadotjs needs for getKeys.
	// It has some extra bytes.
	// maybe from the none u32 in golang being wrong, or maybe slightly off CreateStorageKey call? we slice it
	// here as a hack.
	actualBaseParaHeadsStorageKey := baseParaHeadsStorageKey[:32]
	li.log.WithField("actualBaseParaHeadsStorageKey", actualBaseParaHeadsStorageKey.Hex()).Info("actualBaseParaHeadsStorageKey")

	keysResponse, err := li.relaychainConn.GetAPI().RPC.State.GetKeys(actualBaseParaHeadsStorageKey, blockHash)
	if err != nil {
		li.log.WithError(err).Error("Failed to get all parachain keys")
	}

	headersResponse, err := li.relaychainConn.GetAPI().RPC.State.QueryStorage(keysResponse, blockHash, blockHash)
	if err != nil {
		li.log.WithError(err).Error("Failed to get all parachain headers")
	}

	li.log.Info("Got all parachain headers")
	var headers []types.Header
	var ourParachainHeader types.Header
	for _, headerResponse := range headersResponse {
		for _, change := range headerResponse.Changes {

			// TODO fix this manual slice with a proper type decode. only the last few bytes are for the ParaId,
			// not sure what the early ones are for.
			key := change.StorageKey[40:]
			var parachainID types.U32
			if err := types.DecodeFromBytes(key, &parachainID); err != nil {
				li.log.WithError(err).Error("Failed to decode parachain ID")
			}

			li.log.WithField("parachainId", parachainID).Info("Decoding header for parachain")
			var encodableOpaqueHeader types.Bytes
			if err := types.DecodeFromBytes(change.StorageData, &encodableOpaqueHeader); err != nil {
				li.log.WithError(err).Error("Failed to decode MMREncodableOpaqueLeaf")
			}

			var header types.Header
			if err := types.DecodeFromBytes(encodableOpaqueHeader, &header); err != nil {
				li.log.WithError(err).Error("Failed to decode Header")
			}
			li.log.WithFields(logrus.Fields{
				"headerBytes":           fmt.Sprintf("%#x", encodableOpaqueHeader),
				"header.ParentHash":     header.ParentHash.Hex(),
				"header.Number":         header.Number,
				"header.StateRoot":      header.StateRoot.Hex(),
				"header.ExtrinsicsRoot": header.ExtrinsicsRoot.Hex(),
				"header.Digest":         header.Digest,
				"parachainId":           parachainID,
			}).Info("Decoded header for parachain")
			headers = append(headers, header)

			if parachainID == types.U32(ourParachainId) {
				ourParachainHeader = header
			}
		}
	}
	return headers, ourParachainHeader
}

func createParachainHeaderProof(allParaHeads []types.Header, ourParaHead types.Header) string {
	//TODO: implement
	return ""
}

func (li *BeefyListener) extractCommitments(
	paraHeader types.Header,
	mmrProof types.GenerateMMRProofResponse,
	ourParaHeadProof string) ([]MessagePackage, error) {

	li.log.WithFields(logrus.Fields{
		"blockNumber": paraHeader.Number,
	}).Debug("Extracting commitment from parachain header")

	auxDigestItems, err := getAuxiliaryDigestItems(paraHeader.Digest)
	if err != nil {
		return nil, err
	}

	var messagePackages []MessagePackage
	for _, auxDigestItem := range auxDigestItems {
		li.log.WithFields(logrus.Fields{
			"block":          paraHeader.Number,
			"channelID":      auxDigestItem.AsCommitment.ChannelID,
			"commitmentHash": auxDigestItem.AsCommitment.Hash.Hex(),
		}).Debug("Found commitment hash in header digest")
		commitmentHash := auxDigestItem.AsCommitment.Hash
		commitmentData, err := li.getDataForDigestItem(&auxDigestItem)
		if err != nil {
			return nil, err
		}
		messagePackage := MessagePackage{
			auxDigestItem.AsCommitment.ChannelID,
			commitmentHash,
			commitmentData,
			paraHeader,
			ourParaHeadProof,
			mmrProof,
		}
		messagePackages = append(messagePackages, messagePackage)
	}

	return messagePackages, nil
}

func getAuxiliaryDigestItems(digest types.Digest) ([]chainTypes.AuxiliaryDigestItem, error) {
	var auxDigestItems []chainTypes.AuxiliaryDigestItem
	for _, digestItem := range digest {
		if digestItem.IsOther {
			var auxDigestItem chainTypes.AuxiliaryDigestItem
			err := types.DecodeFromBytes(digestItem.AsOther, &auxDigestItem)
			if err != nil {
				return nil, err
			}
			auxDigestItems = append(auxDigestItems, auxDigestItem)
		}
	}
	return auxDigestItems, nil
}

func (li *BeefyListener) getDataForDigestItem(digestItem *chainTypes.AuxiliaryDigestItem) (types.StorageDataRaw, error) {
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

	return *data, nil
}
