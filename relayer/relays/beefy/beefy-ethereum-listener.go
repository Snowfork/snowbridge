package beefy

import (
	"context"
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
	config   *SinkConfig
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
		config:  config,
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
		log.Info(fmt.Sprintf("Syncing Relayer from block %d...", li.config.StartBlock))
		err := li.pollHistoricEventsAndHeaders(ctx, uint64(li.config.DescendantsUntilFinal))
		if err != nil {
			return err
		}
		log.Info(fmt.Sprintf("Relayer fully synced. Starting live processing on block number %d...", blockNumber))
	}

	// In live mode the relayer processes blocks as they're mined and broadcast
	eg.Go(func() error {
		err := li.pollEventsAndHeaders(ctx, uint64(li.config.DescendantsUntilFinal))
		close(li.headers)
		return err
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
	li.processHistoricalInitialVerificationSuccessfulEvents(ctx, blockNumber, latestBlockNumber)
	li.processHistoricalFinalVerificationSuccessfulEvents(ctx, blockNumber, latestBlockNumber)
	// Send transactions for items in database based on their statuses
	li.forwardWitnessedBeefyJustifications()
	li.forwardReadyToCompleteItems(ctx, blockNumber, descendantsUntilFinal)
	return nil
}

func (li *BeefyEthereumListener) pollEventsAndHeaders(ctx context.Context, descendantsUntilFinal uint64) error {
	headers := make(chan *gethTypes.Header, 5)

	sub, err := li.ethereumConn.GetClient().SubscribeNewHead(ctx, headers)
	if err != nil {
		return err
	}
	defer sub.Unsubscribe()

	for {
		select {
		case <-ctx.Done():
			log.WithField("reason", ctx.Err()).Info("Shutting down ethereum listener")
			return nil
		case err := <-sub.Err():
			log.WithError(err).Error("Error with ethereum header subscription")
			return err
		case gethheader := <-headers:
			blockNumber := gethheader.Number.Uint64()
			li.forwardWitnessedBeefyJustifications()
			li.processInitialVerificationSuccessfulEvents(ctx, blockNumber)
			li.forwardReadyToCompleteItems(ctx, blockNumber, descendantsUntilFinal)
			li.processFinalVerificationSuccessfulEvents(ctx, blockNumber)
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
func (li *BeefyEthereumListener) processHistoricalInitialVerificationSuccessfulEvents(ctx context.Context,
	blockNumber, latestBlockNumber uint64) {

	// Query previous InitialVerificationSuccessful events and update the status of BEEFY justifications in database
	events, err := li.queryInitialVerificationSuccessfulEvents(ctx, blockNumber, &latestBlockNumber)
	if err != nil {
		log.WithError(err).Error("Failure fetching event logs")
	}

	log.Info(fmt.Sprintf(
		"Found %d InitialVerificationSuccessful events between blocks %d-%d",
		len(events), blockNumber, latestBlockNumber),
	)

	for _, event := range events {
		// Fetch validation data from contract using event.ID
		validationData, err := li.beefyLightClient.ContractCaller.ValidationData(nil, event.Id)
		if err != nil {
			log.WithError(err).Error(fmt.Sprintf("Error querying validation data for ID %d", event.Id))
		}

		// Attempt to match items in database based on their payload
		itemFoundInDatabase := false
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
				updateCmd := store.NewDatabaseCmd(item, store.Update, instructions)
				li.dbMessages <- updateCmd

				itemFoundInDatabase = true
				break
			}
		}
		if !itemFoundInDatabase {
			// Don't have an existing item to update, therefore we won't be able to build the completion tx
			log.Error("BEEFY justification data not found in database for InitialVerificationSuccessful event. Ignoring event.")
		}
	}
}

// processInitialVerificationSuccessfulEvents transitions matched database items from status
// InitialVerificationTxSent to InitialVerificationTxConfirmed
func (li *BeefyEthereumListener) processInitialVerificationSuccessfulEvents(ctx context.Context,
	blockNumber uint64) {

	events, err := li.queryInitialVerificationSuccessfulEvents(ctx, blockNumber, &blockNumber)
	if err != nil {
		log.WithError(err).Error("Failure fetching event logs")
	}

	if len(events) > 0 {
		log.Info(fmt.Sprintf("Found %d InitialVerificationSuccessful events on block %d", len(events), blockNumber))
	}

	for _, event := range events {
		log.WithFields(logrus.Fields{
			"blockHash":   event.Raw.BlockHash.Hex(),
			"blockNumber": event.Raw.BlockNumber,
			"txHash":      event.Raw.TxHash.Hex(),
		}).Info("event information")

		// Only process events emitted by transactions sent from our node
		if event.Prover != li.ethereumConn.GetKP().CommonAddress() {
			continue
		}

		item := li.beefyDB.GetItemByInitialVerificationTxHash(event.Raw.TxHash)
		if item.Status != store.InitialVerificationTxSent {
			continue
		}

		log.Infof(
			"3: Updating item %s status from 'InitialVerificationTxSent' to 'InitialVerificationTxConfirmed'",
			event.Id,
		)
		instructions := map[string]interface{}{
			"contract_id":       event.Id.Int64(),
			"status":            store.InitialVerificationTxConfirmed,
			"complete_on_block": event.Raw.BlockNumber + li.blockWaitPeriod,
		}
		updateCmd := store.NewDatabaseCmd(item, store.Update, instructions)
		li.dbMessages <- updateCmd
	}
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

// processHistoricalFinalVerificationSuccessfulEvents processes historical FinalVerificationSuccessful
// events, updating the status of matched BEEFY justifications in the database
func (li *BeefyEthereumListener) processHistoricalFinalVerificationSuccessfulEvents(ctx context.Context,
	blockNumber, latestBlockNumber uint64) {
	// Query previous FinalVerificationSuccessful events and update the status of BEEFY justifications in database
	events, err := li.queryFinalVerificationSuccessfulEvents(ctx, blockNumber, &latestBlockNumber)
	if err != nil {
		log.WithError(err).Error("Failure fetching event logs")
	}
	log.Info(fmt.Sprintf(
		"Found %d FinalVerificationSuccessful events between blocks %d-%d",
		len(events), blockNumber, latestBlockNumber),
	)

	for _, event := range events {
		item := li.beefyDB.GetItemByID(event.Id.Int64())
		if int64(item.ID) == event.Id.Int64() {
			log.Infof(
				"Deleting finalized item %s from the database",
				event.Id,
			)
			deleteCmd := store.NewDatabaseCmd(item, store.Delete, nil)
			li.dbMessages <- deleteCmd
		} else {
			log.Error("BEEFY justification data not found in database for FinalVerificationSuccessful event. Ignoring event.")
		}
	}
}

// processFinalVerificationSuccessfulEvents removes finalized commitments from the relayer's BEEFY justification database
func (li *BeefyEthereumListener) processFinalVerificationSuccessfulEvents(ctx context.Context,
	blockNumber uint64) {
	events, err := li.queryFinalVerificationSuccessfulEvents(ctx, blockNumber, &blockNumber)
	if err != nil {
		log.WithError(err).Error("Failure fetching event logs")
	}

	if len(events) > 0 {
		log.Info(fmt.Sprintf("Found %d FinalVerificationSuccessful events on block %d", len(events), blockNumber))
	}

	for _, event := range events {
		log.WithFields(logrus.Fields{
			"blockHash":   event.Raw.BlockHash.Hex(),
			"blockNumber": event.Raw.BlockNumber,
			"txHash":      event.Raw.TxHash.Hex(),
		}).Info("event information")

		if event.Prover != li.ethereumConn.GetKP().CommonAddress() {
			continue
		}

		log.Infof(
			"6: Deleting finalized item %s from the database",
			event.Id,
		)

		item := li.beefyDB.GetItemByID(event.Id.Int64())
		deleteCmd := store.NewDatabaseCmd(item, store.Delete, nil)
		li.dbMessages <- deleteCmd
	}
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
func (li *BeefyEthereumListener) forwardWitnessedBeefyJustifications() {
	witnessedItems := li.beefyDB.GetItemsByStatus(store.CommitmentWitnessed)
	for _, item := range witnessedItems {
		li.beefyMessages <- *item
	}
}

// forwardReadyToCompleteItems updates the status of items in the database to ReadyToComplete if the
// current block number has passed their CompleteOnBlock number
func (li *BeefyEthereumListener) forwardReadyToCompleteItems(ctx context.Context, blockNumber, descendantsUntilFinal uint64) {
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
			li.beefyMessages <- *item
		}
	}
}
