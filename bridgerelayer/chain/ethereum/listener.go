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

	"github.com/snowfork/polkadot-ethereum/bridgerelayer/chain"
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

	events := make(chan gethTypes.Log)
	for _, contract := range li.contracts {
		query := makeQuery(contract)

		_, err := li.conn.client.SubscribeFilterLogs(ctx, query, events)
		if err != nil {
			li.log.WithFields(logrus.Fields{
				"address": contract.Address.Hex(),
			}).Error("Failed to subscribe to application events")
			continue
		}

		li.log.WithFields(logrus.Fields{
			"contractAddress": contract.Address.Hex(),
			"contractName":    contract.Name,
		}).Info("Subscribed to contract events")
	}

	for {
		select {
		case <-ctx.Done():
			return ctx.Err()
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

func makeQuery(contract Contract) geth.FilterQuery {
	signature := contract.ABI.Events["AppTransfer"].ID.Hex()
	topic := gethCommon.HexToHash(signature)

	return geth.FilterQuery{
		Addresses: []gethCommon.Address{contract.Address},
		Topics:    [][]gethCommon.Hash{{topic}},
	}
}
