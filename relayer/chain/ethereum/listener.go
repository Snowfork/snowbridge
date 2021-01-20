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
	"github.com/snowfork/polkadot-ethereum/relayer/chain/ethereum/syncer"
)

const MaxMessagesPerSend = 10

// Listener streams the Ethereum blockchain for application events
type Listener struct {
	conn      *Connection
	contracts []Contract
	messages  chan<- []chain.Message
	headers   chan<- chain.Header
	log       *logrus.Entry
}

func NewListener(conn *Connection, messages chan<- []chain.Message, headers chan<- chain.Header, contracts []Contract, log *logrus.Entry) (*Listener, error) {
	return &Listener{
		conn:      conn,
		contracts: contracts,
		messages:  messages,
		headers:   headers,
		log:       log,
	}, nil
}

func (li *Listener) Start(cxt context.Context, eg *errgroup.Group, initBlockHeight uint64, descendantsUntilFinal uint64) error {
	hcs, err := NewHeaderCacheState(
		eg,
		initBlockHeight,
		&DefaultBlockLoader{conn: li.conn},
		nil,
	)
	if err != nil {
		return err
	}

	eg.Go(func() error {
		return li.pollEventsAndHeaders(cxt, initBlockHeight, descendantsUntilFinal, hcs)
	})

	return nil
}

func (li *Listener) onDone(ctx context.Context) error {
	li.log.Info("Shutting down listener...")
	if li.messages != nil {
		close(li.messages)
	}
	close(li.headers)
	return ctx.Err()
}

func (li *Listener) pollEventsAndHeaders(
	ctx context.Context,
	initBlockHeight uint64,
	descendantsUntilFinal uint64,
	hcs *HeaderCacheState,
) error {
	headers := make(chan *gethTypes.Header, 5)
	headerEg, headerCtx := errgroup.WithContext(ctx)
	var eventQuery *geth.FilterQuery

	if li.messages == nil {
		li.log.Info("Not polling events since channel is nil")
	} else {
		eventQuery = makeFilterQuery(li.contracts)

		for _, contract := range li.contracts {
			li.log.WithFields(logrus.Fields{
				"addresses":    contract.Address.Hex(),
				"contractName": contract.Name,
			}).Debug("Polling contract events")
		}
	}

	headerSyncer := syncer.NewSyncer(descendantsUntilFinal, syncer.NewHeaderLoader(li.conn.client), headers, li.log)

	li.log.Info("Syncing headers starting...")
	err := headerSyncer.StartSync(headerCtx, headerEg, initBlockHeight-1)
	if err != nil {
		li.log.WithError(err).Error("Failed to start header sync")
		return err
	}

	for {
		select {
		case <-ctx.Done():
			return li.onDone(ctx)
		case <-headerCtx.Done():
			return li.onDone(ctx)
		case gethheader := <-headers:
			li.forwardHeader(hcs, gethheader)
			if eventQuery == nil {
				continue
			}

			finalizedHeader := gethheader.Number.Uint64() - descendantsUntilFinal
			eventQuery.FromBlock = new(big.Int).SetUint64(finalizedHeader)
			eventQuery.ToBlock = new(big.Int).SetUint64(finalizedHeader)
			events, err := li.conn.client.FilterLogs(ctx, *eventQuery)
			if err != nil {
				li.log.WithFields(logrus.Fields{
					"blockNumber": finalizedHeader,
				}).WithError(err).Error("Failed to query logs for finalized block")
				continue
			}

			li.forwardEvents(ctx, hcs, events)
		}
	}
}

func (li *Listener) forwardEvents(ctx context.Context, hcs *HeaderCacheState, events []gethTypes.Log) {
	messages := make([]chain.Message, len(events))

	for i, event := range events {
		receiptTrie, err := hcs.GetReceiptTrie(ctx, event.BlockHash)
		if err != nil {
			li.log.WithFields(logrus.Fields{
				"blockHash":   event.BlockHash.Hex(),
				"blockNumber": event.BlockNumber,
				"txHash":      event.TxHash.Hex(),
			}).WithError(err).Error("Failed to get receipt trie for event")
			return
		}

		msg, err := MakeMessageFromEvent(&event, receiptTrie, li.log)
		if err != nil {
			li.log.WithFields(logrus.Fields{
				"address":     event.Address.Hex(),
				"blockHash":   event.BlockHash.Hex(),
				"blockNumber": event.BlockNumber,
				"txHash":      event.TxHash.Hex(),
			}).WithError(err).Error("Failed to generate message from ethereum event")
			return
		}

		messages[i] = *msg
		if (i+1)%MaxMessagesPerSend == 0 || i == len(events)-1 {
			start := i + 1 - MaxMessagesPerSend
			if i == len(events)-1 {
				start = i - (i % MaxMessagesPerSend)
			}
			li.messages <- messages[start : i+1]
		}
	}
}

func (li *Listener) forwardHeader(hcs *HeaderCacheState, gethheader *gethTypes.Header) {
	cache, err := hcs.GetEthashproofCache(gethheader.Number.Uint64())
	if err != nil {
		li.log.WithFields(logrus.Fields{
			"blockHash":   gethheader.Hash().Hex(),
			"blockNumber": gethheader.Number,
		}).WithError(err).Error("Failed to get ethashproof cache for header")
		return
	}

	header, err := MakeHeaderFromEthHeader(gethheader, cache, li.log)
	if err != nil {
		li.log.WithFields(logrus.Fields{
			"blockHash":   gethheader.Hash().Hex(),
			"blockNumber": gethheader.Number,
		}).WithError(err).Error("Failed to generate header from ethereum header")
	} else {
		li.headers <- *header
	}
}

func makeFilterQuery(contracts []Contract) *geth.FilterQuery {
	var addresses []gethCommon.Address
	var topics []gethCommon.Hash
	topicSet := make(map[gethCommon.Hash]bool)

	for _, contract := range contracts {
		addresses = append(addresses, contract.Address)
		for _, event := range contract.ABI.Events {
			signature := gethCommon.HexToHash(event.ID.Hex())
			_, exists := topicSet[signature]
			if !exists {
				topics = append(topics, signature)
				topicSet[signature] = true
			}
		}
	}

	return &geth.FilterQuery{
		Addresses: addresses,
		Topics:    [][]gethCommon.Hash{topics},
	}
}
