// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package ethereum

import (
	"context"
	"math/big"

	geth "github.com/ethereum/go-ethereum"
	gethCommon "github.com/ethereum/go-ethereum/common"
	gethTypes "github.com/ethereum/go-ethereum/core/types"
	"github.com/sirupsen/logrus"
	"golang.org/x/sync/errgroup"

	"github.com/snowfork/polkadot-ethereum/relayer/chain"
)

// Listener streams the Ethereum blockchain for application events
type Listener struct {
	conn      *Connection
	contracts []Contract
	messages  chan<- chain.Message
	headers   chan<- chain.Header
	log       *logrus.Entry
}

func NewListener(conn *Connection, messages chan<- chain.Message, headers chan<- chain.Header, contracts []Contract, log *logrus.Entry) (*Listener, error) {
	return &Listener{
		conn:      conn,
		contracts: contracts,
		messages:  messages,
		headers:   headers,
		log:       log,
	}, nil
}

func (li *Listener) Start(cxt context.Context, eg *errgroup.Group, initBlockHeight uint64) error {
	hcs, err := NewHeaderCacheState(eg, initBlockHeight, nil)
	if err != nil {
		return err
	}

	eg.Go(func() error {
		return li.pollEventsAndHeaders(cxt, initBlockHeight, hcs)
	})

	return nil
}

func (li *Listener) pollEventsAndHeaders(ctx context.Context, initBlockHeight uint64, hcs *HeaderCacheState) error {
	events := make(chan gethTypes.Log)
	var eventsSubscriptionErr <-chan error
	headers := make(chan *gethTypes.Header, 5)
	var headersSubscriptionErr <-chan error

	if li.messages == nil {
		li.log.Info("Not polling events since channel is nil")
	} else {
		li.log.Info("Polling events started")

		query := makeFilterQuery(li.contracts)

		subscription, err := li.conn.client.SubscribeFilterLogs(ctx, query, events)
		if err != nil {
			li.log.WithError(err).Error("Failed to subscribe to application events")
			return err
		}
		eventsSubscriptionErr = subscription.Err()

		for _, contract := range li.contracts {
			li.log.WithFields(logrus.Fields{
				"addresses":    contract.Address.Hex(),
				"contractName": contract.Name,
			}).Debug("Subscribed to contract events")
		}
	}

	li.log.Info("Polling headers started")

	currentBlock := initBlockHeight
	newestBlock := uint64(0)
	subscription, err := li.conn.client.SubscribeNewHead(ctx, headers)
	if err != nil {
		li.log.WithError(err).Error("Failed to subscribe to new headers")
		return err
	}
	headersSubscriptionErr = subscription.Err()

	for {
		select {
		case <-ctx.Done():
			return ctx.Err()
		case err := <-eventsSubscriptionErr:
			li.log.WithError(err).Error("Events subscription terminated")
			return err
		case err := <-headersSubscriptionErr:
			li.log.WithError(err).Error("Headers subscription terminated")
			return err
		case event := <-events:
			li.log.WithFields(logrus.Fields{
				"address":     event.Address.Hex(),
				"txHash":      event.TxHash.Hex(),
				"blockNumber": event.BlockNumber,
			}).Info("Witnessed transaction for application")

			msg, err := MakeMessageFromEvent(event, li.log)
			if err != nil {
				li.log.WithFields(logrus.Fields{
					"address":     event.Address.Hex(),
					"txHash":      event.TxHash.Hex(),
					"blockNumber": event.BlockNumber,
				}).Error("Failed to generate message from ethereum event")
			} else {
				li.messages <- *msg
			}
		case gethheader := <-headers:
			blockNumber := gethheader.Number.Uint64()
			newestBlock = blockNumber
			li.log.WithFields(logrus.Fields{
				"blockNumber": blockNumber,
			}).Info("Witnessed new block header")

			if blockNumber <= currentBlock+1 {
				li.forwardHeader(hcs, gethheader)
				currentBlock = blockNumber
			}
		default:
			if currentBlock == newestBlock {
				continue
			}

			gethheader, err := li.conn.client.HeaderByNumber(ctx, new(big.Int).SetUint64(currentBlock))
			if err != nil {
				li.log.WithFields(logrus.Fields{
					"blockNumber": currentBlock,
				}).Error("Failed to retrieve old block header")
			} else {
				li.log.WithFields(logrus.Fields{
					"blockNumber": currentBlock,
				}).Info("Retrieved old block header")
				li.forwardHeader(hcs, gethheader)
				currentBlock = currentBlock + 1
			}
		}
	}
}

func (li *Listener) forwardHeader(hcs *HeaderCacheState, gethheader *gethTypes.Header) {
	cache, err := hcs.GetEthashproofCache(gethheader.Number.Uint64())
	if err != nil {
		li.log.WithFields(logrus.Fields{
			"blockHash":   gethheader.Hash(),
			"blockNumber": gethheader.Number,
		}).Error("Failed to get ethashproof cache for header")
	}

	header, err := MakeHeaderFromEthHeader(gethheader, cache, li.log)
	if err != nil {
		li.log.WithFields(logrus.Fields{
			"blockHash":   gethheader.Hash(),
			"blockNumber": gethheader.Number,
		}).Error("Failed to generate header from ethereum header")
	} else {
		li.headers <- *header
	}
}

func makeFilterQuery(contracts []Contract) geth.FilterQuery {
	var addresses []gethCommon.Address
	var topics []gethCommon.Hash

	for _, contract := range contracts {
		addresses = append(addresses, contract.Address)
		signature := contract.ABI.Events["AppTransfer"].Id().Hex()
		topics = append(topics, gethCommon.HexToHash(signature))
	}

	return geth.FilterQuery{
		Addresses: addresses,
		Topics:    [][]gethCommon.Hash{topics},
	}
}
