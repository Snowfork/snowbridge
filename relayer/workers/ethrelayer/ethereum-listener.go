// Copyright 2021 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package ethrelayer

import (
	"context"

	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"
	etypes "github.com/ethereum/go-ethereum/core/types"

	gethTypes "github.com/ethereum/go-ethereum/core/types"
	"github.com/sirupsen/logrus"
	"golang.org/x/sync/errgroup"

	"github.com/snowfork/polkadot-ethereum/relayer/chain"
	"github.com/snowfork/polkadot-ethereum/relayer/chain/ethereum"
	"github.com/snowfork/polkadot-ethereum/relayer/chain/ethereum/syncer"
	"github.com/snowfork/polkadot-ethereum/relayer/contracts/outbound"
)

const MaxMessagesPerSend = 10

// EthereumListener streams the Ethereum blockchain for application events
type EthereumListener struct {
	config                      *ethereum.Config
	conn                        *ethereum.Connection
	basicOutboundChannel        *outbound.BasicOutboundChannel
	incentivizedOutboundChannel *outbound.IncentivizedOutboundChannel
	mapping                     map[common.Address]string
	messages                    chan<- []chain.Message
	headers                     chan<- chain.Header
	log                         *logrus.Entry
}

func NewEthereumListener(
	config *ethereum.Config,
	conn *ethereum.Connection,
	messages chan<- []chain.Message,
	headers chan<- chain.Header,
	log *logrus.Entry,
) *EthereumListener {
	return &EthereumListener{
		config:                      config,
		conn:                        conn,
		basicOutboundChannel:        nil,
		incentivizedOutboundChannel: nil,
		mapping:                     make(map[common.Address]string),
		messages:                    messages,
		headers:                     headers,
		log:                         log,
	}
}

func (li *EthereumListener) Start(cxt context.Context, eg *errgroup.Group, initBlockHeight uint64, descendantsUntilFinal uint64) error {
	hcs, err := ethereum.NewHeaderCacheState(
		eg,
		initBlockHeight,
		&ethereum.DefaultBlockLoader{Conn: li.conn},
		nil,
	)
	if err != nil {
		return err
	}

	basicOutboundChannel, err := outbound.NewBasicOutboundChannel(common.HexToAddress(li.config.Channels.Basic.Outbound), li.conn.GetClient())
	if err != nil {
		return err
	}
	li.basicOutboundChannel = basicOutboundChannel

	incentivizedOutboundChannel, err := outbound.NewIncentivizedOutboundChannel(common.HexToAddress(li.config.Channels.Incentivized.Outbound), li.conn.GetClient())
	if err != nil {
		return err
	}
	li.incentivizedOutboundChannel = incentivizedOutboundChannel

	li.mapping[common.HexToAddress(li.config.Channels.Basic.Outbound)] = "BasicInboundChannel.submit"
	li.mapping[common.HexToAddress(li.config.Channels.Incentivized.Outbound)] = "IncentivizedInboundChannel.submit"

	eg.Go(func() error {
		err := li.pollEventsAndHeaders(cxt, initBlockHeight, descendantsUntilFinal, hcs)
		if li.messages != nil {
			close(li.messages)
		}
		close(li.headers)
		return err
	})

	return nil
}

func (li *EthereumListener) pollEventsAndHeaders(
	ctx context.Context,
	initBlockHeight uint64,
	descendantsUntilFinal uint64,
	hcs *ethereum.HeaderCacheState,
) error {
	headers := make(chan *gethTypes.Header, 5)
	headerEg, headerCtx := errgroup.WithContext(ctx)

	headerSyncer := syncer.NewSyncer(descendantsUntilFinal, syncer.NewHeaderLoader(li.conn.GetClient()), headers, li.log)

	li.log.Info("Syncing headers starting...")
	err := headerSyncer.StartSync(headerCtx, headerEg, initBlockHeight-1)
	if err != nil {
		li.log.WithError(err).Error("Failed to start header sync")
		return err
	}

	for {
		select {
		case <-ctx.Done():
			li.log.Info("Shutting down listener...")
			return ctx.Err()
		case <-headerCtx.Done():
			li.log.Info("Shutting down listener...")
			return ctx.Err()
		case gethheader := <-headers:
			err := li.forwardHeader(hcs, gethheader)
			if err != nil {
				return err
			}

			if li.messages == nil {
				li.log.Info("Not polling events since channel is nil")
			}

			// Don't attempt to forward events prior to genesis block
			if descendantsUntilFinal > gethheader.Number.Uint64() {
				continue
			}

			finalizedBlockNumber := gethheader.Number.Uint64() - descendantsUntilFinal
			var events []*etypes.Log

			filterOptions := bind.FilterOpts{Start: finalizedBlockNumber, End: &finalizedBlockNumber, Context: ctx}

			basicEvents, err := li.queryBasicEvents(li.basicOutboundChannel, &filterOptions)
			if err != nil {
				li.log.WithError(err).Error("Failure fetching event logs")
				return err
			}
			events = append(events, basicEvents...)

			incentivizedEvents, err := li.queryIncentivizedEvents(li.incentivizedOutboundChannel, &filterOptions)
			if err != nil {
				li.log.WithError(err).Error("Failure fetching event logs")
				return err
			}
			events = append(events, incentivizedEvents...)

			li.forwardEvents(ctx, hcs, events)
		}
	}
}

func (li *EthereumListener) queryBasicEvents(contract *outbound.BasicOutboundChannel, options *bind.FilterOpts) ([]*etypes.Log, error) {
	var events []*etypes.Log

	iter, err := contract.FilterMessage(options)
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
		events = append(events, &iter.Event.Raw)
	}
	return events, nil
}

func (li *EthereumListener) queryIncentivizedEvents(contract *outbound.IncentivizedOutboundChannel, options *bind.FilterOpts) ([]*etypes.Log, error) {
	var events []*etypes.Log

	iter, err := contract.FilterMessage(options)
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
		events = append(events, &iter.Event.Raw)
	}
	return events, nil
}

func (li *EthereumListener) forwardEvents(ctx context.Context, hcs *ethereum.HeaderCacheState, events []*etypes.Log) error {
	messages := make([]chain.Message, len(events))

	for i, event := range events {
		receiptTrie, err := hcs.GetReceiptTrie(ctx, event.BlockHash)
		if err != nil {
			li.log.WithFields(logrus.Fields{
				"blockHash":   event.BlockHash.Hex(),
				"blockNumber": event.BlockNumber,
				"txHash":      event.TxHash.Hex(),
			}).WithError(err).Error("Failed to get receipt trie for event")
			return err
		}

		msg, err := ethereum.MakeMessageFromEvent(li.mapping, event, receiptTrie, li.log)
		if err != nil {
			li.log.WithFields(logrus.Fields{
				"address":     event.Address.Hex(),
				"blockHash":   event.BlockHash.Hex(),
				"blockNumber": event.BlockNumber,
				"txHash":      event.TxHash.Hex(),
			}).WithError(err).Error("Failed to generate message from ethereum event")
			return err
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

	return nil
}

func (li *EthereumListener) forwardHeader(hcs *ethereum.HeaderCacheState, gethheader *gethTypes.Header) error {
	cache, err := hcs.GetEthashproofCache(gethheader.Number.Uint64())
	if err != nil {
		li.log.WithFields(logrus.Fields{
			"blockHash":   gethheader.Hash().Hex(),
			"blockNumber": gethheader.Number,
		}).WithError(err).Error("Failed to get ethashproof cache for header")
		return err
	}

	header, err := ethereum.MakeHeaderFromEthHeader(gethheader, cache, li.log)
	if err != nil {
		li.log.WithFields(logrus.Fields{
			"blockHash":   gethheader.Hash().Hex(),
			"blockNumber": gethheader.Number,
		}).WithError(err).Error("Failed to generate header from ethereum header")
		return err
	} else {
		li.headers <- *header
	}

	return nil
}
