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
	log       *logrus.Entry
}

func NewListener(conn *Connection, messages chan<- chain.Message, contracts []Contract, log *logrus.Entry) (*Listener, error) {
	return &Listener{
		conn:      conn,
		contracts: contracts,
		messages:  messages,
		log:       log,
	}, nil
}

func (li *Listener) Start(cxt context.Context, eg *errgroup.Group) error {
	eg.Go(func() error {
		return li.pollEvents(cxt)
	})

	return nil
}

func (li *Listener) pollEvents(ctx context.Context) error {
	li.log.Info("Polling started")

	query := makeFilterQuery(li.contracts)

	events := make(chan gethTypes.Log)

	subscription, err := li.conn.client.SubscribeFilterLogs(ctx, query, events)
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

	for {
		select {
		case <-ctx.Done():
			return ctx.Err()
		case err := <-subscription.Err():
			li.log.WithError(err).Error("Subscription terminated")
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
