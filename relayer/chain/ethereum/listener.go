// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package ethereum

import (
	"context"

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

func (li *Listener) Start(cxt context.Context, eg *errgroup.Group) error {
	eg.Go(func() error {
		return li.pollEventsAndHeaders(cxt)
	})

	return nil
}

func (li *Listener) pollEventsAndHeaders(ctx context.Context) error {
	events := make(chan gethTypes.Log)
	subscriptionEvents := *new(geth.Subscription)
	if li.messages == nil {
		li.log.Info("Not polling events since channel is nil")
	} else {
		li.log.Info("Polling events started")

		query := makeFilterQuery(li.contracts)

		subscription, err := li.conn.client.SubscribeFilterLogs(ctx, query, events)
		subscriptionEvents = subscription
		if err != nil {
			li.log.WithError(err).Error("Failed to subscribe to application events")
			return err
		}

		for _, contract := range li.contracts {
			li.log.WithFields(logrus.Fields{
				"addresses":    contract.Address.Hex(),
				"contractName": contract.Name,
			}).Debug("Subscribed to contract events")
		}
	}

	headers := make(chan *gethTypes.Header)
	subscriptionHeaders, err := li.conn.client.SubscribeNewHead(ctx, headers)
	if err != nil {
		li.log.WithError(err).Error("Failed to subscribe to new headers")
		return err
	}

	li.log.Info("Polling headers started")

	for {
		select {
		case <-ctx.Done():
			return ctx.Err()
		case err := <-subscriptionEvents.Err():
			li.log.WithError(err).Error("Events subscription terminated")
			return err
		case err := <-subscriptionHeaders.Err():
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
			li.log.WithFields(logrus.Fields{
				"blockNumber": gethheader.Number,
			}).Info("Witnessed block header")

			header, err := MakeHeaderFromEthHeader(gethheader, li.log)
			if err != nil {
				li.log.WithFields(logrus.Fields{
					"blockHash":   gethheader.Hash(),
					"blockNumber": gethheader.Number,
				}).Error("Failed to generate header from ethereum header")
			} else {
				li.headers <- *header
			}
		}
	}
}

func makeFilterQuery(contracts []Contract) geth.FilterQuery {
	var addresses []gethCommon.Address
	var topics []gethCommon.Hash

	for _, contract := range contracts {
		addresses = append(addresses, contract.Address)
		signature := contract.ABI.Events["AppTransfer"].ID.Hex()
		topics = append(topics, gethCommon.HexToHash(signature))
	}

	return geth.FilterQuery{
		Addresses: addresses,
		Topics:    [][]gethCommon.Hash{topics},
	}
}
