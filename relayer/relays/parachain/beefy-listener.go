package parachain

import (
	"context"
	"fmt"

	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"
	gethTypes "github.com/ethereum/go-ethereum/core/types"
	"github.com/sirupsen/logrus"
	"golang.org/x/sync/errgroup"

	"github.com/snowfork/go-substrate-rpc-client/v3/types"
	"github.com/snowfork/snowbridge/relayer/chain/ethereum"
	"github.com/snowfork/snowbridge/relayer/chain/parachain"
	"github.com/snowfork/snowbridge/relayer/chain/relaychain"
	"github.com/snowfork/snowbridge/relayer/contracts/beefylightclient"

	log "github.com/sirupsen/logrus"
)

type BeefyListener struct {
	config              *SourceConfig
	ethereumConn        *ethereum.Connection
	beefyLightClient    *beefylightclient.Contract
	relaychainConn      *relaychain.Connection
	parachainConnection *parachain.Connection
	paraID              uint32
	messages            chan<- MessagePackage
}

func NewBeefyListener(
	config *SourceConfig,
	ethereumConn *ethereum.Connection,
	relaychainConn *relaychain.Connection,
	parachainConnection *parachain.Connection,
	messages chan<- MessagePackage,
) *BeefyListener {
	return &BeefyListener{
		config:              config,
		ethereumConn:        ethereumConn,
		relaychainConn:      relaychainConn,
		parachainConnection: parachainConnection,
		messages:            messages,
	}
}

func (li *BeefyListener) Start(ctx context.Context, eg *errgroup.Group) error {

	// Set up light client bridge contract
	address := common.HexToAddress(li.config.Contracts.BeefyLightClient)
	beefyLightClientContract, err := beefylightclient.NewContract(address, li.ethereumConn.GetClient())
	if err != nil {
		return err
	}
	li.beefyLightClient = beefyLightClientContract

	// Fetch ParaId
	storageKeyForParaID, err := types.CreateStorageKey(li.parachainConnection.Metadata(), "ParachainInfo", "ParachainId", nil, nil)
	if err != nil {
		return err
	}

	var paraID uint32
	ok, err := li.parachainConnection.API().RPC.State.GetStorageLatest(storageKeyForParaID, &paraID)
	if err != nil {
		log.WithError(err).Error("Failed to get para id for snowbridge")
		return err
	}
	if !ok {
		log.Error("Expected parachain but chain does not provide a parachain ID")
		return fmt.Errorf("invalid parachain")
	}

	log.WithField("paraId", paraID).Info("Fetched parachain id")
	li.paraID = paraID

	eg.Go(func() error {
		beefyBlockNumber, beefyBlockHash, err := li.fetchLatestBeefyBlock(ctx)
		if err != nil {
			log.WithError(err).Error("Failed to get latest relay chain block number and hash")
			return err
		}

		log.WithFields(logrus.Fields{
			"blockHash": beefyBlockHash.Hex(),
			"blockNumber": beefyBlockNumber,
		}).Info("Fetched latest verified polkadot block")

		paraHead, err := li.relaychainConn.FetchFinalizedParaHead(beefyBlockHash, paraID)
		if err != nil {
			log.WithError(err).Error("Parachain not registered")
			return err
		}

		log.WithFields(logrus.Fields{
			"header.ParentHash":     paraHead.ParentHash.Hex(),
			"header.Number":         paraHead.Number,
			"header.StateRoot":      paraHead.StateRoot.Hex(),
			"header.ExtrinsicsRoot": paraHead.ExtrinsicsRoot.Hex(),
			"header.Digest":         paraHead.Digest,
			"parachainId":           paraID,
		}).Info("Fetched finalized header for parachain")

		paraBlockNumber := uint64(paraHead.Number)

		paraBlockHash, err := li.parachainConnection.API().RPC.Chain.GetBlockHash(paraBlockNumber)
		if err != nil {
			log.WithError(err).Error("Failed to get latest finalized para block hash")
			return err
		}

		messagePackages, err := li.buildMissedMessagePackages(ctx, beefyBlockNumber, paraBlockNumber, paraBlockHash)
		if err != nil {
			log.WithError(err).Error("Failed to build missed message package")
			return err
		}

		li.emitMessagePackages(messagePackages)

		err = li.subBeefyJustifications(ctx)
		return err
	})

	return nil
}

func (li *BeefyListener) subBeefyJustifications(ctx context.Context) error {
	headers := make(chan *gethTypes.Header, 5)

	sub, err := li.ethereumConn.GetClient().SubscribeNewHead(ctx, headers)
	if err != nil {
		log.WithError(err).Error("Error creating ethereum header subscription")
		return err
	}
	defer sub.Unsubscribe()

	for {
		select {
		case <-ctx.Done():
			log.WithField("reason", ctx.Err()).Info("Shutting down beefy listener")
			if li.messages != nil {
				close(li.messages)
			}
			return nil
		case err := <-sub.Err():
			log.WithError(err).Error("Error with ethereum header subscription")
			return err
		case gethheader := <-headers:
			// Query LightClientBridge contract's ContractNewMMRRoot events
			blockNumber := gethheader.Number.Uint64()
			var beefyLightClientEvents []*beefylightclient.ContractNewMMRRoot

			contractEvents, err := li.queryBeefyLightClientEvents(ctx, blockNumber, &blockNumber)
			if err != nil {
				log.WithError(err).Error("Failure fetching event logs")
				return err
			}
			beefyLightClientEvents = append(beefyLightClientEvents, contractEvents...)

			if len(beefyLightClientEvents) > 0 {
				log.Info(fmt.Sprintf("Found %d BeefyLightClient ContractNewMMRRoot events on block %d", len(beefyLightClientEvents), blockNumber))
			}
			li.processBeefyLightClientEvents(ctx, beefyLightClientEvents)
		}
	}
}

// processLightClientEvents matches events to BEEFY commitment info by transaction hash
func (li *BeefyListener) processBeefyLightClientEvents(ctx context.Context, events []*beefylightclient.ContractNewMMRRoot) error {
	for _, event := range events {

		beefyBlockNumber := event.BlockNumber

		log.WithFields(logrus.Fields{
			"beefyBlockNumber":    beefyBlockNumber,
			"ethereumBlockNumber": event.Raw.BlockNumber,
			"ethereumTxHash":      event.Raw.TxHash.Hex(),
		}).Info("Witnessed a new MMRRoot event")

		log.WithField("beefyBlockNumber", beefyBlockNumber).Info("Getting hash for relay chain block")
		beefyBlockHash, err := li.relaychainConn.API().RPC.Chain.GetBlockHash(uint64(beefyBlockNumber))
		if err != nil {
			log.WithError(err).Error("Failed to get block hash")
			return err
		}
		log.WithField("beefyBlockHash", beefyBlockHash.Hex()).Info("Got relay chain blockhash")

		paraHead, err := li.relaychainConn.FetchFinalizedParaHead(beefyBlockHash, li.paraID)
		if err != nil {
			log.WithError(err).Error("Failed to get finalized para head from relay chain")
			return err
		}

		paraBlockNumber := uint64(paraHead.Number)

		paraBlockHash, err := li.parachainConnection.API().RPC.Chain.GetBlockHash(paraBlockNumber)
		if err != nil {
			log.WithError(err).Error("Failed to get latest finalized para block hash")
			return err
		}

		messagePackages, err := li.buildMissedMessagePackages(ctx, beefyBlockNumber, paraBlockNumber, paraBlockHash)
		if err != nil {
			log.WithError(err).Error("Failed to build missed message packages")
			return err
		}

		li.emitMessagePackages(messagePackages)

	}
	return nil
}

func (li *BeefyListener) emitMessagePackages(packages []MessagePackage) {
	for _, messagePackage := range packages {
		log.WithFields(logrus.Fields{
			"channelID":             messagePackage.channelID,
			"commitmentHash":        messagePackage.commitmentHash,
			"commitmentData":        messagePackage.commitmentData,
			"ourParaHeadProof":      messagePackage.paraHeadProof,
			"ourParaHeadProofPos":   messagePackage.paraHeadProofPos,
			"ourParaHeadProofWidth": messagePackage.paraHeadProofWidth,
			"mmrProof":              messagePackage.mmrProof,
		}).Info("Beefy Listener emitted new message package")

		li.messages <- messagePackage
	}
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

// Fetch the latest verified beefy block number and hash from Ethereum
func (li *BeefyListener) fetchLatestBeefyBlock(ctx context.Context) (uint64, types.Hash, error) {
	number, err := li.beefyLightClient.LatestBeefyBlock(&bind.CallOpts{
		Pending: false,
		Context: ctx,
	})
	if err != nil {
		log.WithError(err).Error("Failed to get latest verified beefy block number from ethereum")
		return 0, types.Hash{}, err
	}

	hash, err := li.relaychainConn.API().RPC.Chain.GetBlockHash(number)
	if err != nil {
		log.WithError(err).Error("Failed to get latest relay chain block hash from relay chain")
		return 0, types.Hash{}, err
	}

	return number, hash, nil
}
