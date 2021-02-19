// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package ethereum

import (
	"context"

	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"

	gethTypes "github.com/ethereum/go-ethereum/core/types"
	"github.com/sirupsen/logrus"
	"golang.org/x/sync/errgroup"

	"github.com/snowfork/polkadot-ethereum/relayer/chain"
	"github.com/snowfork/polkadot-ethereum/relayer/chain/ethereum/syncer"
	"github.com/snowfork/polkadot-ethereum/relayer/contracts/outbound"
)

const MaxMessagesPerSend = 10

// Listener streams the Ethereum blockchain for application events
type Listener struct {
	config    *Config
	conn      *Connection
	contracts []*outbound.Contract
	messages  chan<- []chain.Message
	headers   chan<- chain.Header
	log       *logrus.Entry
}

func NewListener(config *Config, conn *Connection, messages chan<- []chain.Message, headers chan<- chain.Header, contracts []*outbound.Contract, log *logrus.Entry) (*Listener, error) {
	return &Listener{
		config:    config,
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
		&DefaultBlockLoader{Conn: li.conn},
		nil,
	)
	if err != nil {
		return err
	}

	contract, err := outbound.NewContract(common.HexToAddress(li.config.Channels.Basic.Outbound), li.conn.client)
	if err != nil {
		return err
	}
	li.contracts = append(li.contracts, contract)

	contract, err = outbound.NewContract(common.HexToAddress(li.config.Channels.Incentivized.Outbound), li.conn.client)
	if err != nil {
		return err
	}
	li.contracts = append(li.contracts, contract)

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

			if li.messages == nil {
				li.log.Info("Not polling events since channel is nil")
			}

			// Don't attempt to forward events prior to genesis block
			if descendantsUntilFinal > gethheader.Number.Uint64() {
				continue
			}

			finalizedBlockNumber := gethheader.Number.Uint64() - descendantsUntilFinal
			var events []*outbound.ContractMessage

			for _, channelContract := range li.contracts {
				channelEvents, err := li.queryEvents(ctx, channelContract, finalizedBlockNumber, &finalizedBlockNumber)
				if err != nil {
					li.log.WithError(err).Error("Failure fetching event logs")
				}

				events = append(events, channelEvents...)
			}

			li.forwardEvents(ctx, hcs, events)
		}
	}
}

func (li *Listener) queryEvents(ctx context.Context, contract *outbound.Contract, start uint64, end *uint64) ([]*outbound.ContractMessage, error) {
	var events []*outbound.ContractMessage
	filterOps := bind.FilterOpts{Start: start, End: end, Context: ctx}

	iter, err := contract.FilterMessage(&filterOps)
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

		if li.config.Channels.Basic.AccountWhitelistMap[iter.Event.Source] {
			events = append(events, iter.Event)
		}
	}

	return events, nil
}

func (li *Listener) forwardEvents(ctx context.Context, hcs *HeaderCacheState, events []*outbound.ContractMessage) {
	messages := make([]chain.Message, len(events))

	for i, event := range events {
		receiptTrie, err := hcs.GetReceiptTrie(ctx, event.Raw.BlockHash)
		if err != nil {
			li.log.WithFields(logrus.Fields{
				"blockHash":   event.Raw.BlockHash.Hex(),
				"blockNumber": event.Raw.BlockNumber,
				"txHash":      event.Raw.TxHash.Hex(),
			}).WithError(err).Error("Failed to get receipt trie for event")
			return
		}

		msg, err := MakeMessageFromEvent(event, receiptTrie, li.log)
		if err != nil {
			li.log.WithFields(logrus.Fields{
				"address":     event.Raw.Address.Hex(),
				"blockHash":   event.Raw.BlockHash.Hex(),
				"blockNumber": event.Raw.BlockNumber,
				"txHash":      event.Raw.TxHash.Hex(),
			}).WithError(err).Error("Failed to generate message from ethereum event")
			return
		}

		messages[i] = msg
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
