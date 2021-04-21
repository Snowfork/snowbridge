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
	"github.com/snowfork/polkadot-ethereum/relayer/contracts/lightclientbridge"
)

// Listener streams the Ethereum blockchain for application events
type EthereumLightClientListener struct {
	ethereumConfig    *ethereum.Config
	ethereumConn      *ethereum.Connection
	lightClientBridge *lightclientbridge.Contract
	blockWaitPeriod   uint64
	log               *logrus.Entry
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
	lightClientBridgeContract, err := lightclientbridge.NewContract(common.HexToAddress(li.ethereumConfig.LightClientBridge), li.ethereumConn.GetClient())
	if err != nil {
		return err
	}
	li.lightClientBridge = lightClientBridgeContract

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
			// Query LightClientBridge contract's ContractFinalVerificationSuccessful events
			blockNumber := gethheader.Number.Uint64()
			var lightClientBridgeEvents []*lightclientbridge.ContractFinalVerificationSuccessful

			contractEvents, err := li.queryLightClientEvents(ctx, blockNumber, &blockNumber)
			if err != nil {
				li.log.WithError(err).Error("Failure fetching event logs")
				return err
			}
			lightClientBridgeEvents = append(lightClientBridgeEvents, contractEvents...)

			if len(lightClientBridgeEvents) > 0 {
				li.log.Info(fmt.Sprintf("Found %d LightClientBridge contract events on block %d", len(lightClientBridgeEvents), blockNumber))
			}
			li.processLightClientEvents(ctx, lightClientBridgeEvents)
		}
	}
}

// queryLightClientEvents queries ContractFinalVerificationSuccessful events from the LightClientBridge contract
func (li *EthereumLightClientListener) queryLightClientEvents(ctx context.Context, start uint64,
	end *uint64) ([]*lightclientbridge.ContractFinalVerificationSuccessful, error) {
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

// processLightClientEvents matches events to BEEFY commitment info by transaction hash
func (li *EthereumLightClientListener) processLightClientEvents(ctx context.Context, events []*lightclientbridge.ContractFinalVerificationSuccessful) {
	for _, event := range events {
		li.log.WithFields(logrus.Fields{
			"blockHash":   event.Raw.BlockHash.Hex(),
			"blockNumber": event.Raw.BlockNumber,
			"txHash":      event.Raw.TxHash.Hex(),
		}).Info("event information")
	}
}
