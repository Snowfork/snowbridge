package parachaincommitmentrelayer

import (
	"context"
	"fmt"

	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"

	gethTypes "github.com/ethereum/go-ethereum/core/types"
	"github.com/sirupsen/logrus"
	"golang.org/x/sync/errgroup"

	"github.com/snowfork/polkadot-ethereum/relayer/chain/ethereum"
	"github.com/snowfork/polkadot-ethereum/relayer/contracts/beefylightclient"
)

// Listener streams the Ethereum blockchain for application events
type EthereumLightClientListener struct {
	ethereumConfig   *ethereum.Config
	ethereumConn     *ethereum.Connection
	beefyLightClient *beefylightclient.Contract
	blockWaitPeriod  uint64
	log              *logrus.Entry
}

func NewEthereumLightClientListener(ethereumConfig *ethereum.Config, ethereumConn *ethereum.Connection,
	log *logrus.Entry) (*EthereumLightClientListener, error) {
	return &EthereumLightClientListener{
		ethereumConfig: ethereumConfig,
		ethereumConn:   ethereumConn,
		log:            log,
	}, nil
}

func (li *EthereumLightClientListener) Start(ctx context.Context, eg *errgroup.Group) error {

	// Set up light client bridge contract
	beefyLightClientContract, err := beefylightclient.NewContract(common.HexToAddress(li.ethereumConfig.BeefyLightClient), li.ethereumConn.GetClient())
	if err != nil {
		return err
	}
	li.beefyLightClient = beefyLightClientContract

	eg.Go(func() error {
		err := li.pollEventsAndHeaders(ctx)
		return err
	})

	return nil
}

func (li *EthereumLightClientListener) pollEventsAndHeaders(
	ctx context.Context,
) error {
	headers := make(chan *gethTypes.Header, 5)

	li.ethereumConn.GetClient().SubscribeNewHead(ctx, headers)

	for {
		select {
		case <-ctx.Done():
			li.log.Info("Shutting down listener...")
			return ctx.Err()
		case gethheader := <-headers:
			// Query BeefyLightClient contract's ContractFinalVerificationSuccessful events
			blockNumber := gethheader.Number.Uint64()
			var beefyLightClientEvents []*beefylightclient.ContractFinalVerificationSuccessful

			contractEvents, err := li.queryLightClientEvents(ctx, blockNumber, &blockNumber)
			if err != nil {
				li.log.WithError(err).Error("Failure fetching event logs")
				return err
			}
			beefyLightClientEvents = append(beefyLightClientEvents, contractEvents...)

			if len(beefyLightClientEvents) > 0 {
				li.log.Info(fmt.Sprintf("Found %d BeefyLightClient contract events on block %d", len(beefyLightClientEvents), blockNumber))
			}
			li.processLightClientEvents(ctx, beefyLightClientEvents)
		}
	}
}

// queryLightClientEvents queries ContractFinalVerificationSuccessful events from the BeefyLightClient contract
func (li *EthereumLightClientListener) queryLightClientEvents(ctx context.Context, start uint64,
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

// processLightClientEvents matches events to BEEFY commitment info by transaction hash
func (li *EthereumLightClientListener) processLightClientEvents(ctx context.Context, events []*beefylightclient.ContractFinalVerificationSuccessful) {
	for _, event := range events {
		li.log.WithFields(logrus.Fields{
			"blockHash":   event.Raw.BlockHash.Hex(),
			"blockNumber": event.Raw.BlockNumber,
			"txHash":      event.Raw.TxHash.Hex(),
		}).Info("event information")
	}
}
