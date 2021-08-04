// Copyright 2021 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package ethereum

import (
	"context"
	"errors"
	"os"
	"path/filepath"

	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"
	etypes "github.com/ethereum/go-ethereum/core/types"

	gethTypes "github.com/ethereum/go-ethereum/core/types"
	"github.com/sirupsen/logrus"
	"golang.org/x/sync/errgroup"

	"github.com/snowfork/snowbridge/relayer/chain"
	"github.com/snowfork/snowbridge/relayer/chain/ethereum"
	"github.com/snowfork/snowbridge/relayer/chain/ethereum/syncer"
	"github.com/snowfork/snowbridge/relayer/contracts/basic"
	"github.com/snowfork/snowbridge/relayer/contracts/incentivized"

	log "github.com/sirupsen/logrus"
)

// EthereumListener streams the Ethereum blockchain for application events
type EthereumListener struct {
	ethashDataDir               string
	ethashCacheDir              string
	config                      *SourceConfig
	conn                        *ethereum.Connection
	basicOutboundChannel        *basic.BasicOutboundChannel
	incentivizedOutboundChannel *incentivized.IncentivizedOutboundChannel
	mapping                     map[common.Address]string
	payloads                    chan<- ParachainPayload
	headerSyncer                *syncer.Syncer
}

func NewEthereumListener(
	config *SourceConfig,
	conn *ethereum.Connection,
	payloads chan<- ParachainPayload,
) *EthereumListener {
	return &EthereumListener{
		ethashDataDir:               filepath.Join(config.DataDir, "ethash-data"),
		ethashCacheDir:              filepath.Join(config.DataDir, "ethash-cache"),
		config:                      config,
		conn:                        conn,
		basicOutboundChannel:        nil,
		incentivizedOutboundChannel: nil,
		mapping:                     make(map[common.Address]string),
		payloads:                    payloads,
		headerSyncer:                nil,
	}
}

func (li *EthereumListener) Start(cxt context.Context, eg *errgroup.Group, initBlockHeight uint64, descendantsUntilFinal uint64) error {
	closeWithError := func(err error) error {
		log.Info("Shutting down listener...")
		close(li.payloads)
		return err
	}

	var err error

	err = os.Mkdir(li.ethashDataDir, 0755)
	if err != nil && !errors.Is(err, os.ErrExist) {
		log.WithError(err).Error("Could not create data dir")
		return err
	}

	err = os.Mkdir(li.ethashCacheDir, 0755)
	if err != nil && !errors.Is(err, os.ErrExist) {
		log.WithError(err).Error("Could not create cache dir")
		return err
	}

	hcs, err := ethereum.NewHeaderCacheState(
		li.ethashDataDir,
		li.ethashCacheDir,
		eg,
		initBlockHeight,
		&ethereum.DefaultBlockLoader{Conn: li.conn},
		nil,
	)
	if err != nil {
		return closeWithError(err)
	}

	var address common.Address

	address = common.HexToAddress(li.config.Contracts.BasicOutboundChannel)
	basicOutboundChannel, err := basic.NewBasicOutboundChannel(address, li.conn.GetClient())
	if err != nil {
		return closeWithError(err)
	}
	li.basicOutboundChannel = basicOutboundChannel
	li.mapping[address] = "BasicInboundChannel.submit"

	address = common.HexToAddress(li.config.Contracts.IncentivizedOutboundChannel)
	incentivizedOutboundChannel, err := incentivized.NewIncentivizedOutboundChannel(address, li.conn.GetClient())
	if err != nil {
		return closeWithError(err)
	}
	li.incentivizedOutboundChannel = incentivizedOutboundChannel
	li.mapping[address] = "IncentivizedInboundChannel.submit"

	headersIn := make(chan *gethTypes.Header, 5)
	li.headerSyncer = syncer.NewSyncer(
		descendantsUntilFinal,
		syncer.NewHeaderLoader(li.conn.GetClient()),
		headersIn,
	)

	eg.Go(func() error {
		err := li.processEventsAndHeaders(cxt, initBlockHeight, descendantsUntilFinal, headersIn, hcs)

		// Ensures the context is canceled so that the channel below is
		// closed by the syncer
		eg.Go(func() error { return err })

		// Avoid deadlock if the syncer is still trying to send a header
		for range headersIn {
			log.Debug("Discarded header")
		}

		return closeWithError(err)
	})

	return nil
}

func (li *EthereumListener) processEventsAndHeaders(
	ctx context.Context,
	initBlockHeight uint64,
	descendantsUntilFinal uint64,
	headers <-chan *gethTypes.Header,
	hcs *ethereum.HeaderCacheState,
) error {
	headerEg, headerCtx := errgroup.WithContext(ctx)

	log.Info("Syncing headers starting...")
	err := li.headerSyncer.StartSync(headerCtx, headerEg, initBlockHeight-1)
	if err != nil {
		log.WithError(err).Error("Failed to start header sync")
		return err
	}

	for {
		select {
		case <-ctx.Done():
			log.WithField("reason", ctx.Err()).Info("Shutting down ethereum listener")
			return nil
		case <-headerCtx.Done():
			return headerCtx.Err()
		case gethheader, ok := <-headers:
			if !ok {
				return nil
			}

			header, err := li.makeOutgoingHeader(hcs, gethheader)
			if err != nil {
				return err
			}

			// Don't attempt to forward events prior to genesis block
			if descendantsUntilFinal > gethheader.Number.Uint64() {
				li.payloads <- ParachainPayload{Header: header}
				continue
			}

			finalizedBlockNumber := gethheader.Number.Uint64() - descendantsUntilFinal
			var events []*etypes.Log

			filterOptions := bind.FilterOpts{Start: finalizedBlockNumber, End: &finalizedBlockNumber, Context: ctx}

			basicEvents, err := li.queryBasicEvents(li.basicOutboundChannel, &filterOptions)
			if err != nil {
				log.WithError(err).Error("Failure fetching event logs")
				return err
			}
			events = append(events, basicEvents...)

			incentivizedEvents, err := li.queryIncentivizedEvents(li.incentivizedOutboundChannel, &filterOptions)
			if err != nil {
				log.WithError(err).Error("Failure fetching event logs")
				return err
			}
			events = append(events, incentivizedEvents...)

			messages, err := li.makeOutgoingMessages(ctx, hcs, events)
			if err != nil {
				return err
			}

			li.payloads <- ParachainPayload{Header: header, Messages: messages}
		}
	}
}

func (li *EthereumListener) queryBasicEvents(contract *basic.BasicOutboundChannel, options *bind.FilterOpts) ([]*etypes.Log, error) {
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

func (li *EthereumListener) queryIncentivizedEvents(contract *incentivized.IncentivizedOutboundChannel, options *bind.FilterOpts) ([]*etypes.Log, error) {
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

func (li *EthereumListener) makeOutgoingMessages(
	ctx context.Context,
	hcs *ethereum.HeaderCacheState,
	events []*etypes.Log,
) ([]*chain.EthereumOutboundMessage, error) {
	messages := make([]*chain.EthereumOutboundMessage, len(events))

	for i, event := range events {
		receiptTrie, err := hcs.GetReceiptTrie(ctx, event.BlockHash)
		if err != nil {
			log.WithFields(logrus.Fields{
				"blockHash":   event.BlockHash.Hex(),
				"blockNumber": event.BlockNumber,
				"txHash":      event.TxHash.Hex(),
			}).WithError(err).Error("Failed to get receipt trie for event")
			return nil, err
		}

		msg, err := ethereum.MakeMessageFromEvent(li.mapping, event, receiptTrie)
		if err != nil {
			log.WithFields(logrus.Fields{
				"address":     event.Address.Hex(),
				"blockHash":   event.BlockHash.Hex(),
				"blockNumber": event.BlockNumber,
				"txHash":      event.TxHash.Hex(),
			}).WithError(err).Error("Failed to generate message from ethereum event")
			return nil, err
		}

		messages[i] = msg
	}

	return messages, nil
}

func (li *EthereumListener) makeOutgoingHeader(
	hcs *ethereum.HeaderCacheState,
	gethheader *gethTypes.Header,
) (*chain.Header, error) {
	cache, err := hcs.GetEthashproofCache(gethheader.Number.Uint64())
	if err != nil {
		log.WithFields(logrus.Fields{
			"blockHash":   gethheader.Hash().Hex(),
			"blockNumber": gethheader.Number,
		}).WithError(err).Error("Failed to get ethashproof cache for header")
		return nil, err
	}

	header, err := ethereum.MakeHeaderFromEthHeader(gethheader, cache, li.ethashDataDir)
	if err != nil {
		log.WithFields(logrus.Fields{
			"blockHash":   gethheader.Hash().Hex(),
			"blockNumber": gethheader.Number,
		}).WithError(err).Error("Failed to generate header from ethereum header")
		return nil, err
	}
	return header, nil
}
