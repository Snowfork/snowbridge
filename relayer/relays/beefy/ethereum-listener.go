package beefy

import (
	"context"
	"errors"

	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"
	"github.com/jinzhu/gorm"

	gethTypes "github.com/ethereum/go-ethereum/core/types"
	"github.com/sirupsen/logrus"
	"golang.org/x/sync/errgroup"

	"github.com/snowfork/snowbridge/relayer/chain"
	"github.com/snowfork/snowbridge/relayer/chain/ethereum"
	"github.com/snowfork/snowbridge/relayer/contracts/beefylightclient"

	log "github.com/sirupsen/logrus"
)

// Listener streams the Ethereum blockchain for application events
type EthereumListener struct {
	config           *SinkConfig
	ethereumConn     *ethereum.Connection
	store          *Database
	beefyLightClient *beefylightclient.Contract
	tasks    chan<- Task
	dbMessages       chan<- DatabaseCmd
	headers          chan<- chain.Header
	blockWaitPeriod  uint64
}

func NewEthereumListener(
	config *SinkConfig,
	ethereumConn *ethereum.Connection,
	store *Database,
	tasks chan<- Task,
	dbMessages chan<- DatabaseCmd,
	headers chan<- chain.Header,
) *EthereumListener {
	return &EthereumListener{
		config:          config,
		ethereumConn:    ethereumConn,
		store:         store,
		dbMessages:      dbMessages,
		tasks:   tasks,
		headers:         headers,
		blockWaitPeriod: 0,
	}
}

func (li *EthereumListener) Start(ctx context.Context, eg *errgroup.Group) (uint64, error) {
	// Set up light client bridge contract
	address := common.HexToAddress(li.config.Contracts.BeefyLightClient)
	beefyLightClientContract, err := beefylightclient.NewContract(address, li.ethereumConn.GetClient())
	if err != nil {
		return 0, err
	}
	li.beefyLightClient = beefyLightClientContract

	latestBeefyBlock, err := li.beefyLightClient.ContractCaller.LatestBeefyBlock(&bind.CallOpts{
		Pending: false,
		Context: ctx,
	})
	if err != nil {
		return 0, err
	}

	// Fetch BLOCK_WAIT_PERIOD from light client bridge contract
	blockWaitPeriod, err := li.beefyLightClient.ContractCaller.BLOCKWAITPERIOD(nil)
	if err != nil {
		return 0, err
	}
	li.blockWaitPeriod = blockWaitPeriod

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

	return latestBeefyBlock, nil
}

func (li *EthereumListener) pollEventsAndHeaders(ctx context.Context, descendantsUntilFinal uint64) error {
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
func (li *EthereumListener) queryInitialVerificationSuccessfulEvents(ctx context.Context, start uint64,
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

// processInitialVerificationSuccessfulEvents transitions matched database items from status
// InitialVerificationTxSent to InitialVerificationTxConfirmed
func (li *EthereumListener) processInitialVerificationSuccessfulEvents(
	ctx context.Context,
	blockNumber uint64,
) error {
	events, err := li.queryInitialVerificationSuccessfulEvents(ctx, blockNumber, &blockNumber)
	if err != nil {
		log.WithError(err).Error("Failure querying InitialVerificationSuccessful events")
		return err
	}

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

		task, err := li.store.GetTaskByInitialVerificationTx(event.Raw.TxHash)
		if err != nil {
			if errors.Is(err, gorm.ErrRecordNotFound) {
				log.WithField("txHash", event.Raw.TxHash.Hex()).Info("Query for items by InitialVerificationTx returned no results")
				continue
			}
			log.WithError(err).Error("Failed to query Beefy DB")
			return err
		}

		instructions := map[string]interface{}{
			"validation_id":       event.Id.Int64(),
			"status":            InitialVerificationTxConfirmed,
			"complete_on_block": event.Raw.BlockNumber + li.blockWaitPeriod,
		}

		log.WithFields(log.Fields{
			"task": log.Fields{
				"ID": task.ID,
				"ValidationID": event.Id.Int64(),
				"CompleteOnBlock": event.Raw.BlockNumber + li.blockWaitPeriod,
			},
		}).Debug("Task completed initial signature commitment")

		select {
		case <-ctx.Done():
			return ctx.Err()
		case li.dbMessages <- NewDatabaseCmd(task, Update, instructions):
		}
	}

	return nil
}

// queryFinalVerificationSuccessfulEvents queries ContractFinalVerificationSuccessful events from the BeefyLightClient contract
func (li *EthereumListener) queryFinalVerificationSuccessfulEvents(
	ctx context.Context,
	start uint64,
	end *uint64,
) ([]*beefylightclient.ContractFinalVerificationSuccessful, error) {
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
func (li *EthereumListener) processFinalVerificationSuccessfulEvents(
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
	}).Trace("Queried for FinalVerificationSuccessful events")

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
			continue
		}

		task, err := li.store.GetTaskByValidationID(event.Id.Int64())
		if err != nil {
			if errors.Is(err, gorm.ErrRecordNotFound) {
				log.WithField("ID", event.Id.Int64()).Info("Query for items by ID returned no results")
				continue
			}
			log.WithError(err).Error("Failed to query Beefy DB")
			return err
		}

		select {
		case <-ctx.Done():
			return ctx.Err()
		case li.dbMessages <- NewDatabaseCmd(task, Delete, nil):
		}
	}

	return nil
}

// forwardWitnessedBeefyJustifications forwards witnessed BEEFY commitments to the Ethereum writer
func (li *EthereumListener) forwardWitnessedBeefyJustifications(ctx context.Context) error {
	witnessedItems, err := li.store.GetTasksByStatus(CommitmentWitnessed)
	if err != nil {
		log.WithError(err).Error("Failure querying beefy DB for items by CommitmentWitnessed status")
		return err
	}
	for _, item := range witnessedItems {
		select {
		case <-ctx.Done():
			return ctx.Err()
		case li.tasks <- *item:
		}
	}

	return nil
}

// forwardReadyToCompleteItems updates the status of items in the database to ReadyToComplete if the
// current block number has passed their CompleteOnBlock number
func (li *EthereumListener) forwardReadyToCompleteItems(ctx context.Context, blockNumber, descendantsUntilFinal uint64) error {
	// Mark items ReadyToComplete if the current block number has passed their CompleteOnBlock number
	tasks, err := li.store.GetTasksByStatus(InitialVerificationTxConfirmed)
	if err != nil {
		log.WithError(err).Error("Failure querying beefy DB for items by InitialVerificationTxConfirmed status")
		return err
	}

	for _, task := range tasks {
		if task.CompleteOnBlock+descendantsUntilFinal <= blockNumber {
			task.Status = ReadyToComplete
			log.WithFields(log.Fields{
				"task": log.Fields{
					"ID": task.ID,
					"ValidationID": task.ValidationID,
				},
			}).Debug("Task is now ready for final signature commitment")

			select {
			case <-ctx.Done():
				return ctx.Err()
			case li.tasks <- *task:
			}
		}
	}

	return nil
}
