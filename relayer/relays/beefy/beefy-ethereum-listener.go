package beefy

import (
	"context"
	"errors"
	"fmt"
	"math/big"

	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"

	gethTypes "github.com/ethereum/go-ethereum/core/types"
	"github.com/sirupsen/logrus"
	"golang.org/x/sync/errgroup"

	"github.com/snowfork/snowbridge/relayer/chain"
	"github.com/snowfork/snowbridge/relayer/chain/ethereum"
	"github.com/snowfork/snowbridge/relayer/contracts/beefylightclient"
	"github.com/snowfork/snowbridge/relayer/relays/beefy/store"

	log "github.com/sirupsen/logrus"
)

// Listener streams the Ethereum blockchain for application events
type BeefyEthereumListener struct {
	config           *SinkConfig
	ethereumConn     *ethereum.Connection
	beefyDB          *store.Database
	beefyLightClient *beefylightclient.Contract
	beefyMessages    chan<- store.BeefyRelayInfo
	dbMessages       chan<- store.DatabaseCmd
	headers          chan<- chain.Header
	blockWaitPeriod  uint64
}

func NewBeefyEthereumListener(
	config *SinkConfig,
	ethereumConn *ethereum.Connection,
	beefyDB *store.Database,
	beefyMessages chan<- store.BeefyRelayInfo,
	dbMessages chan<- store.DatabaseCmd,
	headers chan<- chain.Header,
) *BeefyEthereumListener {
	return &BeefyEthereumListener{
		config:          config,
		ethereumConn:    ethereumConn,
		beefyDB:         beefyDB,
		dbMessages:      dbMessages,
		beefyMessages:   beefyMessages,
		headers:         headers,
		blockWaitPeriod: 0,
	}
}

func (li *BeefyEthereumListener) Start(ctx context.Context, eg *errgroup.Group) error {

	// Set up light client bridge contract
	address := common.HexToAddress(li.config.Contracts.BeefyLightClient)
	beefyLightClientContract, err := beefylightclient.NewContract(address, li.ethereumConn.GetClient())
	if err != nil {
		return err
	}
	li.beefyLightClient = beefyLightClientContract

	// Fetch BLOCK_WAIT_PERIOD from light client bridge contract
	blockWaitPeriod, err := li.beefyLightClient.ContractCaller.BLOCKWAITPERIOD(nil)
	if err != nil {
		return err
	}
	li.blockWaitPeriod = blockWaitPeriod

	// If starting block < latest block, sync the Relayer to the latest block
	blockNumber, err := li.ethereumConn.GetClient().BlockNumber(ctx)
	if err != nil {
		return err
	}

	// Relayer config StartBlock config variable must be updated to the latest Ethereum block number
	if uint64(li.config.StartBlock) < blockNumber {
		log.WithField("blockNumber", li.config.StartBlock).Info("Synchronizing relayer from historical block")
		err := li.pollHistoricEventsAndHeaders(ctx, uint64(li.config.DescendantsUntilFinal))
		if err != nil {
			return err
		}
		log.WithField("blockNumber", blockNumber).Info("Relayer fully synced")
	}

	// In live mode the relayer processes blocks as they're mined and broadcast
	eg.Go(func() error {
		defer close(li.headers)
		err := li.pollEventsAndHeaders(ctx, uint64(li.config.DescendantsUntilFinal))
		log.WithField("reason", err).Info("Shutting down ethereum listener")
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

func (li *BeefyEthereumListener) pollHistoricEventsAndHeaders(ctx context.Context, descendantsUntilFinal uint64) error {
	// Load starting block number and latest block number
	blockNumber := li.config.StartBlock
	latestBlockNumber, err := li.ethereumConn.GetClient().BlockNumber(ctx)
	if err != nil {
		return err
	}
	// Populate database
	err = li.processHistoricalInitialVerificationSuccessfulEvents(ctx, blockNumber, latestBlockNumber)
	if err != nil {
		return err
	}

	li.processFinalVerificationSuccessfulEvents(ctx, blockNumber, latestBlockNumber)
	if err != nil {
		return err
	}

	// Send transactions for items in database based on their statuses
	li.forwardWitnessedBeefyJustifications(ctx)
	if err != nil {
		return err
	}

	li.forwardReadyToCompleteItems(ctx, blockNumber, descendantsUntilFinal)
	if err != nil {
		return err
	}

	return nil
}

func (li *BeefyEthereumListener) pollEventsAndHeaders(ctx context.Context, descendantsUntilFinal uint64) error {
	headersIn := make(chan *gethTypes.Header)

	sub, err := li.ethereumConn.GetClient().SubscribeNewHead(ctx, headersIn)
	if err != nil {
		return err
	}
	defer sub.Unsubscribe()

	for {
		select {
		case <-ctx.Done():
			return ctx.Err()
		case err := <-sub.Err():
			log.WithError(err).Error("Subscription for ethereum headers failed")
			return err
		case header, ok := <-headersIn:
			if !ok {
				return nil
			}
			blockNumber := header.Number.Uint64()

			log.WithFields(log.Fields{
				"blockNumber": blockNumber,
			}).Debug("Processing new ethereum header")

			err := li.forwardWitnessedBeefyJustifications(ctx)
			if err != nil {
				return err
			}

			err = li.processInitialVerificationSuccessfulEvents(ctx, blockNumber)
			if err != nil {
				return err
			}

			err = li.forwardReadyToCompleteItems(ctx, blockNumber, descendantsUntilFinal)
			if err != nil {
				return err
			}

			err = li.processFinalVerificationSuccessfulEvents(ctx, blockNumber, blockNumber)
			if err != nil {
				return err
			}
		}
	}
}

// queryInitialVerificationSuccessfulEvents queries ContractInitialVerificationSuccessful events from the BeefyLightClient contract
func (li *BeefyEthereumListener) queryInitialVerificationSuccessfulEvents(ctx context.Context, start uint64,
	end *uint64) ([]*beefylightclient.ContractInitialVerificationSuccessful, error) {
	var events []*beefylightclient.ContractInitialVerificationSuccessful
	filterOps := bind.FilterOpts{Start: start, End: end, Context: ctx}

	iter, err := li.beefyLightClient.FilterInitialVerificationSuccessful(&filterOps)
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

// processHistoricalInitialVerificationSuccessfulEvents processes historical InitialVerificationSuccessful
// events, updating the status of matched BEEFY justifications in the database
func (li *BeefyEthereumListener) processHistoricalInitialVerificationSuccessfulEvents(
	ctx context.Context,
	blockNumber,
	latestBlockNumber uint64,
) error {

	// Query previous InitialVerificationSuccessful events and update the status of BEEFY justifications in database
	events, err := li.queryInitialVerificationSuccessfulEvents(ctx, blockNumber, &latestBlockNumber)
	if err != nil {
		log.WithError(err).Error("Failure querying InitialVerificationSuccessful events")
		return err
	}

	log.WithFields(log.Fields{
		"startBlock": blockNumber,
		"endBlock": latestBlockNumber,
		"count": len(events),
	}).Debug("Queried for InitialVerificationSuccessful events")

	for _, event := range events {
		log.WithFields(logrus.Fields{
			"blockHash":   event.Raw.BlockHash.Hex(),
			"blockNumber": event.Raw.BlockNumber,
			"txHash":      event.Raw.TxHash.Hex(),
			"Prover":      event.Prover.Hex(),
		}).Info("Processing InitialVerificationSuccessful event")

		// Only process events emitted by transactions sent from our node
		if event.Prover != li.ethereumConn.GetKP().CommonAddress() {
			log.WithFields(logrus.Fields{
				"Prover": event.Prover.Hex(),
			}).Info("Skipping InitialVerificationSuccessful event as it has an unknown prover address")
			continue
		}

		// Fetch validation data from contract using event.ID
		validationData, err := li.beefyLightClient.ContractCaller.ValidationData(nil, event.Id)
		if err != nil {
			log.WithError(err).Error(fmt.Sprintf("Error querying validation data for ID %d", event.Id))
			return err
		}

		// Attempt to match items in database based on their payload
		found := false
		items := li.beefyDB.GetItemsByStatus(store.CommitmentWitnessed)
		for _, item := range items {
			generatedPayload := li.simulatePayloadGeneration(*item)
			if generatedPayload == validationData.CommitmentHash {
				// Update existing database item
				log.Infof(
					"Updating item %s status from 'CommitmentWitnessed' to 'InitialVerificationTxConfirmed'",
					event.Id,
				)
				instructions := map[string]interface{}{
					"contract_id":             event.Id.Int64(),
					"status":                  store.InitialVerificationTxConfirmed,
					"initial_verification_tx": event.Raw.TxHash.Hex(),
					"complete_on_block":       event.Raw.BlockNumber + li.blockWaitPeriod,
				}

				select {
				case <-ctx.Done():
					return ctx.Err()
				case li.dbMessages <- store.NewDatabaseCmd(item, store.Update, instructions):
				}

				found = true
				break
			}
		}
		if !found {
			// Don't have an existing item to update, therefore we won't be able to build the completion tx
			return fmt.Errorf("item not found in database for InitialVerificationSuccessful event")
		}
	}

	return nil
}

// processInitialVerificationSuccessfulEvents transitions matched database items from status
// InitialVerificationTxSent to InitialVerificationTxConfirmed
func (li *BeefyEthereumListener) processInitialVerificationSuccessfulEvents(
	ctx context.Context,
	blockNumber uint64,
) error {
	events, err := li.queryInitialVerificationSuccessfulEvents(ctx, blockNumber, &blockNumber)
	if err != nil {
		log.WithError(err).Error("Failure querying InitialVerificationSuccessful events")
		return err
	}

	log.WithFields(log.Fields{
		"block": blockNumber,
		"count": len(events),
	}).Debug("Queried for InitialVerificationSuccessful events")

	for _, event := range events {
		log.WithFields(logrus.Fields{
			"blockHash":   event.Raw.BlockHash.Hex(),
			"blockNumber": event.Raw.BlockNumber,
			"txHash":      event.Raw.TxHash.Hex(),
			"Prover":      event.Prover.Hex(),
		}).Info("Processing InitialVerificationSuccessful event")

		// Only process events emitted by transactions sent from our node
		if event.Prover != li.ethereumConn.GetKP().CommonAddress() {
			log.WithFields(logrus.Fields{
				"Prover": event.Prover.Hex(),
			}).Info("Skipping InitialVerificationSuccessful event as it has an unknown prover address")
			continue
		}

		item, err := li.beefyDB.GetItemByInitialVerificationTxHash(event.Raw.TxHash)
		if err != nil {
			log.Error("Failed to retrieve item from Beefy DB")
			return err
		}

		instructions := map[string]interface{}{
			"contract_id":       event.Id.Int64(),
			"status":            store.InitialVerificationTxConfirmed,
			"complete_on_block": event.Raw.BlockNumber + li.blockWaitPeriod,
		}

		select {
		case <-ctx.Done():
			return ctx.Err()
		case li.dbMessages <- store.NewDatabaseCmd(item, store.Update, instructions):
		}
	}

	return nil
}

// queryFinalVerificationSuccessfulEvents queries ContractFinalVerificationSuccessful events from the BeefyLightClient contract
func (li *BeefyEthereumListener) queryFinalVerificationSuccessfulEvents(ctx context.Context, start uint64,
	end *uint64) ([]*beefylightclient.ContractFinalVerificationSuccessful, error) {
	var events []*beefylightclient.ContractFinalVerificationSuccessful
	filterOps := bind.FilterOpts{Start: start, End: end, Context: ctx}

	iter, err := li.beefyLightClient.FilterFinalVerificationSuccessful(&filterOps)
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

// processFinalVerificationSuccessfulEvents removes finalized commitments from the relayer's BEEFY justification database
func (li *BeefyEthereumListener) processFinalVerificationSuccessfulEvents(
	ctx context.Context,
	startBlock uint64,
	endBlock uint64,
) error {
	events, err := li.queryFinalVerificationSuccessfulEvents(ctx, startBlock, &endBlock)
	if err != nil {
		log.WithError(err).Error("Failure querying FinalVerificationSuccessful events")
		return err
	}

	log.WithFields(log.Fields{
		"startBlock": startBlock,
		"endBlock":   endBlock,
		"count":      len(events),
	}).Debug("Queried for FinalVerificationSuccessful events")

	for _, event := range events {
		log.WithFields(logrus.Fields{
			"blockHash":   event.Raw.BlockHash.Hex(),
			"blockNumber": event.Raw.BlockNumber,
			"txHash":      event.Raw.TxHash.Hex(),
			"ID":          event.Id.Int64(),
			"Prover":      event.Prover.Hex(),
		}).Info("Processing FinalVerificationSuccessful event")

		if event.Prover != li.ethereumConn.GetKP().CommonAddress() {
			log.WithFields(logrus.Fields{
				"ID":     event.Id.Int64(),
				"Prover": event.Prover.Hex(),
			}).Info("Skipping FinalVerificationSuccessful event as it has an unknown prover address")
			return nil
		}

		item, err := li.beefyDB.GetItemByID(event.Id.Int64())
		if err != nil {
			log.Error("Failed to retrieve item from Beefy DB")
			return err
		}

		select {
		case <-ctx.Done():
			return ctx.Err()
		case li.dbMessages <- store.NewDatabaseCmd(item, store.Delete, nil):
		}
	}

	return nil
}

// matchGeneratedPayload simulates msg building and payload generation
func (li *BeefyEthereumListener) simulatePayloadGeneration(item store.BeefyRelayInfo) [32]byte {
	beefyJustification, err := item.ToBeefyJustification()
	if err != nil {
		log.WithError(fmt.Errorf("Error converting BeefyRelayInfo to BeefyJustification: %s", err.Error()))
	}

	msg, err := beefyJustification.BuildNewSignatureCommitmentMessage(0, []*big.Int{})
	if err != nil {
		log.WithError(err).Error("Error building commitment message")
	}

	return msg.CommitmentHash
}

// forwardWitnessedBeefyJustifications forwards witnessed BEEFY commitments to the Ethereum writer
func (li *BeefyEthereumListener) forwardWitnessedBeefyJustifications(ctx context.Context) error {
	witnessedItems := li.beefyDB.GetItemsByStatus(store.CommitmentWitnessed)
	for _, item := range witnessedItems {
		select {
		case <-ctx.Done():
			return ctx.Err()
		case li.beefyMessages <- *item:
		}
	}

	return nil
}

// forwardReadyToCompleteItems updates the status of items in the database to ReadyToComplete if the
// current block number has passed their CompleteOnBlock number
func (li *BeefyEthereumListener) forwardReadyToCompleteItems(ctx context.Context, blockNumber, descendantsUntilFinal uint64) error {
	// Mark items ReadyToComplete if the current block number has passed their CompleteOnBlock number
	initialVerificationItems := li.beefyDB.GetItemsByStatus(store.InitialVerificationTxConfirmed)
	if len(initialVerificationItems) > 0 {
		log.Info(fmt.Sprintf("Found %d item(s) in database awaiting completion block", len(initialVerificationItems)))
	}
	for _, item := range initialVerificationItems {
		if item.CompleteOnBlock+descendantsUntilFinal <= blockNumber {
			// Fetch intended completion block's hash
			block, err := li.ethereumConn.GetClient().BlockByNumber(ctx, big.NewInt(int64(item.CompleteOnBlock)))
			if err != nil {
				log.WithError(err).Error("Failure fetching inclusion block")
			}

			log.Infof(
				"4: Updating item %v status from 'InitialVerificationTxConfirmed' to 'ReadyToComplete'",
				item.ID,
			)
			item.Status = store.ReadyToComplete
			item.RandomSeed = block.Hash()

			select {
			case <-ctx.Done():
				return ctx.Err()
			case li.beefyMessages <- *item:
			}
		}
	}

	return nil
}
