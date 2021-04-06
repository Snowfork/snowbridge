// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package ethereum

import (
	"context"
	"fmt"
	"math/big"

	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"

	gethTypes "github.com/ethereum/go-ethereum/core/types"
	"github.com/sirupsen/logrus"
	"golang.org/x/sync/errgroup"

	"github.com/snowfork/polkadot-ethereum/relayer/chain"
	"github.com/snowfork/polkadot-ethereum/relayer/chain/ethereum/syncer"
	"github.com/snowfork/polkadot-ethereum/relayer/contracts/outbound"
	"github.com/snowfork/polkadot-ethereum/relayer/contracts/polkadotrelaychainbridge"
	"github.com/snowfork/polkadot-ethereum/relayer/store"
)

const MaxMessagesPerSend = 10

// Listener streams the Ethereum blockchain for application events
type Listener struct {
	config                   *Config
	conn                     *Connection
	db                       *store.Database
	contracts                []*outbound.Contract
	polkadotRelayChainBridge *polkadotrelaychainbridge.Contract
	messages                 chan<- []chain.Message
	beefyMessages            chan<- store.DatabaseCmd
	headers                  chan<- chain.Header
	log                      *logrus.Entry
}

func NewListener(config *Config, conn *Connection, db *store.Database, messages chan<- []chain.Message, beefyMessages chan<- store.DatabaseCmd, headers chan<- chain.Header, contracts []*outbound.Contract, log *logrus.Entry) (*Listener, error) {
	return &Listener{
		config:        config,
		conn:          conn,
		db:            db,
		contracts:     contracts,
		messages:      messages,
		beefyMessages: beefyMessages,
		headers:       headers,
		log:           log,
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

	polkadotRelayChainBridgeContract, err := polkadotrelaychainbridge.NewContract(common.HexToAddress(li.config.PolkadotRelayChainBridge), li.conn.client)
	if err != nil {
		return err
	}
	li.polkadotRelayChainBridge = polkadotRelayChainBridgeContract

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
			var channelEvents []*outbound.ContractMessage
			for _, channelContract := range li.contracts {
				contractEvents, err := li.queryChannelEvents(ctx, channelContract, finalizedBlockNumber, &finalizedBlockNumber)
				if err != nil {
					li.log.WithError(err).Error("Failure fetching event logs")
				}

				channelEvents = append(channelEvents, contractEvents...)
			}
			li.forwardChannelEvents(ctx, hcs, channelEvents)

			// Query PolkadotRelayChainBridge contract's InitialVerificationSuccessful events
			blockNumber := gethheader.Number.Uint64()
			var relayChainBridgeEvents []*polkadotrelaychainbridge.ContractInitialVerificationSuccessful

			contractEvents, err := li.queryLightClientEvents(ctx, blockNumber, &blockNumber)
			if err != nil {
				li.log.WithError(err).Error("Failure fetching event logs")
			}
			relayChainBridgeEvents = append(relayChainBridgeEvents, contractEvents...)

			if len(relayChainBridgeEvents) > 0 {
				li.log.Info(fmt.Sprintf("Found %d PolkadotRelayChainBridge contract events on block %d", len(relayChainBridgeEvents), blockNumber))
			}
			li.processLightClientEvents(ctx, relayChainBridgeEvents)

			// Mark items ReadyToComplete if the current block number has passed their CompleteOnBlock number
			items := li.db.GetItemsByStatus(store.InitialVerificationTxConfirmed)
			if len(items) > 0 {
				li.log.Info(fmt.Sprintf("Found %d item(s) in database awaiting completion block", len(items)))
			}
			for _, item := range items {
				if item.CompleteOnBlock <= blockNumber {
					li.log.Info("4: Updating item status from 'InitialVerificationTxConfirmed' to 'ReadyToComplete'")

					// Fetch intended completion block's hash
					blockHash := gethheader.Hash()
					if item.CompleteOnBlock < blockNumber {
						block, err := li.conn.client.BlockByNumber(ctx, big.NewInt(int64(item.CompleteOnBlock)))
						if err != nil {
							li.log.WithError(err).Error("Failure fetching inclusion block")
						}
						blockHash = block.Hash()
					}

					instructions := map[string]interface{}{
						"status":      store.ReadyToComplete,
						"random_seed": blockHash,
					}
					updateCmd := store.NewDatabaseCmd(item, store.Update, instructions)
					li.beefyMessages <- updateCmd
				}
			}
		}
	}
}

func (li *Listener) queryChannelEvents(ctx context.Context, contract *outbound.Contract, start uint64, end *uint64) ([]*outbound.ContractMessage, error) {
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

		events = append(events, iter.Event)
	}

	return events, nil
}

func (li *Listener) forwardChannelEvents(ctx context.Context, hcs *HeaderCacheState, events []*outbound.ContractMessage) {
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

// queryLightClientEvents queries ContractInitialVerificationSuccessful events from the PolkadotRelayChainBridge contract
func (li *Listener) queryLightClientEvents(ctx context.Context, start uint64,
	end *uint64) ([]*polkadotrelaychainbridge.ContractInitialVerificationSuccessful, error) {
	var events []*polkadotrelaychainbridge.ContractInitialVerificationSuccessful
	filterOps := bind.FilterOpts{Start: start, End: end, Context: ctx}

	iter, err := li.polkadotRelayChainBridge.FilterInitialVerificationSuccessful(&filterOps)
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
func (li *Listener) processLightClientEvents(ctx context.Context, events []*polkadotrelaychainbridge.ContractInitialVerificationSuccessful) {
	for _, event := range events {
		li.log.WithFields(logrus.Fields{
			"blockHash":   event.Raw.BlockHash.Hex(),
			"blockNumber": event.Raw.BlockNumber,
			"txHash":      event.Raw.TxHash.Hex(),
		}).Info("event information")

		item := li.db.GetItemByInitialVerificationTxHash(event.Raw.TxHash)

		if item.Status != store.InitialVerificationTxSent {
			continue
		}

		li.log.Info("3: Updating item status from 'InitialVerificationTxSent' to 'InitialVerificationTxConfirmed'")
		instructions := map[string]interface{}{
			"status":            store.InitialVerificationTxConfirmed,
			"complete_on_block": event.Raw.BlockNumber + li.config.BeefyBlockDelay,
		}
		updateCmd := store.NewDatabaseCmd(item, store.Update, instructions)
		li.beefyMessages <- updateCmd
	}
}
