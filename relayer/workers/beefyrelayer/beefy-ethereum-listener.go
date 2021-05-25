package beefyrelayer

import (
	"context"
	"fmt"
	"math/big"

	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"

	gethTypes "github.com/ethereum/go-ethereum/core/types"
	"github.com/sirupsen/logrus"
	"golang.org/x/sync/errgroup"

	"github.com/snowfork/polkadot-ethereum/relayer/chain"
	"github.com/snowfork/polkadot-ethereum/relayer/chain/ethereum"
	"github.com/snowfork/polkadot-ethereum/relayer/contracts/lightclientbridge"
	"github.com/snowfork/polkadot-ethereum/relayer/workers/beefyrelayer/store"
)

const MaxMessagesPerSend = 10

// Listener streams the Ethereum blockchain for application events
type BeefyEthereumListener struct {
	ethereumConfig    *ethereum.Config
	ethereumConn      *ethereum.Connection
	beefyDB           *store.Database
	lightClientBridge *lightclientbridge.Contract
	beefyMessages     chan<- store.BeefyRelayInfo
	dbMessages        chan<- store.DatabaseCmd
	headers           chan<- chain.Header
	blockWaitPeriod   uint64
	log               *logrus.Entry
}

func NewBeefyEthereumListener(ethereumConfig *ethereum.Config, ethereumConn *ethereum.Connection, beefyDB *store.Database,
	beefyMessages chan<- store.BeefyRelayInfo, dbMessages chan<- store.DatabaseCmd, headers chan<- chain.Header,
	log *logrus.Entry) *BeefyEthereumListener {
	return &BeefyEthereumListener{
		ethereumConfig:  ethereumConfig,
		ethereumConn:    ethereumConn,
		beefyDB:         beefyDB,
		dbMessages:      dbMessages,
		beefyMessages:   beefyMessages,
		headers:         headers,
		blockWaitPeriod: 0,
		log:             log,
	}
}

func (li *BeefyEthereumListener) Start(cxt context.Context, eg *errgroup.Group, descendantsUntilFinal uint64) error {

	// Set up light client bridge contract
	lightClientBridgeContract, err := lightclientbridge.NewContract(common.HexToAddress(li.ethereumConfig.LightClientBridge), li.ethereumConn.GetClient())
	if err != nil {
		return err
	}
	li.lightClientBridge = lightClientBridgeContract

	// Fetch BLOCK_WAIT_PERIOD from light client bridge contract
	blockWaitPeriod, err := li.lightClientBridge.ContractCaller.BLOCKWAITPERIOD(nil)
	if err != nil {
		return err
	}
	li.blockWaitPeriod = blockWaitPeriod.Uint64()
	eg.Go(func() error {
		err := li.pollEventsAndHeaders(cxt, descendantsUntilFinal)
		close(li.headers)
		return err
	})

	return nil
}

func (li *BeefyEthereumListener) pollEventsAndHeaders(
	ctx context.Context,
	descendantsUntilFinal uint64,
) error {
	headers := make(chan *gethTypes.Header, 5)

	li.ethereumConn.GetClient().SubscribeNewHead(ctx, headers)

	for {
		select {
		case <-ctx.Done():
			li.log.Info("Shutting down listener...")
			return ctx.Err()
		case gethheader := <-headers:

			// Query LightClientBridge contract's InitialVerificationSuccessful events
			blockNumber := gethheader.Number.Uint64()
			var lightClientBridgeInitialVerificationEvents []*lightclientbridge.ContractInitialVerificationSuccessful
			var lightClientBridgeFianlVerificationEvents []*lightclientbridge.ContractFinalVerificationSuccessful

			// Fetch and process initial verification events
			contractInitialVerificationEvents, err := li.queryLightClientInitialVerificationEvents(ctx, blockNumber, &blockNumber)
			if err != nil {
				li.log.WithError(err).Error("Failure fetching event logs")
				return err
			}
			lightClientBridgeInitialVerificationEvents = append(lightClientBridgeInitialVerificationEvents, contractInitialVerificationEvents...)

			if len(lightClientBridgeInitialVerificationEvents) > 0 {
				li.log.Info(fmt.Sprintf(
					"Found %d LightClientBridge contract events on block %d",
					len(lightClientBridgeInitialVerificationEvents),
					blockNumber,
				))
			}
			li.processLightClientInitialVerificationEvents(ctx, lightClientBridgeInitialVerificationEvents)

			// Fetch and process final verification events
			contractFinalVerificationEvents, err := li.queryLightClientFinalVerificationEvents(ctx, blockNumber, &blockNumber)
			if err != nil {
				li.log.WithError(err).Error("Failure fetching event logs")
				return err
			}
			lightClientBridgeFianlVerificationEvents = append(lightClientBridgeFianlVerificationEvents, contractFinalVerificationEvents...)

			if len(lightClientBridgeFianlVerificationEvents) > 0 {
				li.log.Info(fmt.Sprintf(
					"Found %d LightClientBridge contract events on block %d",
					len(lightClientBridgeFianlVerificationEvents),
					blockNumber,
				))
			}
			li.processLightClientFinalVerificationEvents(ctx, lightClientBridgeFianlVerificationEvents)

			// Mark items ReadyToComplete if the current block number has passed their CompleteOnBlock number
			items := li.beefyDB.GetItemsByStatus(store.InitialVerificationTxConfirmed)
			if len(items) > 0 {
				li.log.Info(fmt.Sprintf("Found %d item(s) in database awaiting completion block", len(items)))
			}
			for _, item := range items {
				if item.CompleteOnBlock+descendantsUntilFinal <= blockNumber {
					// Fetch intended completion block's hash
					block, err := li.ethereumConn.GetClient().BlockByNumber(ctx, big.NewInt(int64(item.CompleteOnBlock)))
					if err != nil {
						li.log.WithError(err).Error("Failure fetching inclusion block")
					}

					li.log.Info("3: Updating item status from 'InitialVerificationTxConfirmed' to 'ReadyToComplete'")
					item.Status = store.ReadyToComplete
					item.RandomSeed = block.Hash()
					li.beefyMessages <- *item
				}
			}
		}
	}
}

// queryLightClientEvents queries ContractInitialVerificationSuccessful events from the LightClientBridge contract
func (li *BeefyEthereumListener) queryLightClientInitialVerificationEvents(ctx context.Context, start uint64,
	end *uint64) ([]*lightclientbridge.ContractInitialVerificationSuccessful, error) {
	var events []*lightclientbridge.ContractInitialVerificationSuccessful
	filterOps := bind.FilterOpts{Start: start, End: end, Context: ctx}

	iter, err := li.lightClientBridge.FilterInitialVerificationSuccessful(&filterOps)
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

// queryLightClientEvents queries ContractInitialVerificationSuccessful events from the LightClientBridge contract
func (li *BeefyEthereumListener) queryLightClientFinalVerificationEvents(
	ctx context.Context,
	start uint64,
	end *uint64,
) ([]*lightclientbridge.ContractFinalVerificationSuccessful, error) {
	var events []*lightclientbridge.ContractFinalVerificationSuccessful
	filterOps := bind.FilterOpts{Start: start, End: end, Context: ctx}

	iter, err := li.lightClientBridge.FilterFinalVerificationSuccessful(&filterOps)
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

// processLightClientInitialVerificationEvents matches events to BEEFY commitment info by transaction hash
func (li *BeefyEthereumListener) processLightClientInitialVerificationEvents(
	ctx context.Context,
	events []*lightclientbridge.ContractInitialVerificationSuccessful,
) {
	for _, event := range events {
		// Only process events emitted by transactions sent from our node
		if event.Prover != li.ethereumConn.GetKP().CommonAddress() {
			continue
		}

		li.log.WithFields(logrus.Fields{
			"blockHash":   event.Raw.BlockHash.Hex(),
			"blockNumber": event.Raw.BlockNumber,
			"txHash":      event.Raw.TxHash.Hex(),
		}).Info("event information")

		item := li.beefyDB.GetItemByInitialVerificationTxHash(event.Raw.TxHash)
		if item.Status != store.InitialVerificationTxSent {
			continue
		}

		li.log.Info("2: Updating item status from 'InitialVerificationTxSent' to 'InitialVerificationTxConfirmed'")
		instructions := map[string]interface{}{
			"status":            store.InitialVerificationTxConfirmed,
			"complete_on_block": event.Raw.BlockNumber + li.blockWaitPeriod,
		}
		updateCmd := store.NewDatabaseCmd(item, store.Update, instructions)
		li.dbMessages <- updateCmd
	}
}

// processLightClientFinalVerificationEvents matches events to BEEFY commitment info by transaction hash
func (li *BeefyEthereumListener) processLightClientFinalVerificationEvents(
	ctx context.Context,
	events []*lightclientbridge.ContractFinalVerificationSuccessful,
) {
	for _, event := range events {
		// Only process events emitted by transactions sent from our node
		if event.Prover != li.ethereumConn.GetKP().CommonAddress() {
			continue
		}

		li.log.WithFields(logrus.Fields{
			"blockHash":   event.Raw.BlockHash.Hex(),
			"blockNumber": event.Raw.BlockNumber,
			"txHash":      event.Raw.TxHash.Hex(),
		}).Info("event information")

		item := li.beefyDB.GetItemByInitialVerificationId(event.Id)
		if item.Status != store.CompleteVerificationTxSent {
			continue
		}

		li.log.Info("2: Updating item status from 'CompleteVerificationTxSent' to 'CompleteVerificationTxConfirmed'")
		instructions := map[string]interface{}{
			"status":            store.CompleteVerificationTxConfirmed,
			"complete_on_block": event.Raw.BlockNumber + li.blockWaitPeriod,
		}
		updateCmd := store.NewDatabaseCmd(item, store.Update, instructions)
		li.dbMessages <- updateCmd
	}
}
