// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package substrate

import (
	"bytes"
	"context"
	"encoding/hex"
	"fmt"
	"time"

	"golang.org/x/sync/errgroup"

	"github.com/centrifuge/go-substrate-rpc-client/scale"
	types "github.com/centrifuge/go-substrate-rpc-client/types"
	"github.com/sirupsen/logrus"
	"github.com/snowfork/polkadot-ethereum/relayer/chain"
)

type Listener struct {
	eventDecoder *EventDecoder
	config       *Config
	conn         *Connection
	messages     chan<- chain.Message
	log          *logrus.Entry
}

func NewListener(config *Config, conn *Connection, messages chan<- chain.Message, log *logrus.Entry) *Listener {
	return &Listener{
		eventDecoder: NewEventDecoder(&conn.metadata),
		config:       config,
		conn:         conn,
		messages:     messages,
		log:          log,
	}
}

func (li *Listener) Start(ctx context.Context, eg *errgroup.Group) error {
	eg.Go(func() error {
		return li.pollBlocks(ctx)
	})

	return nil
}

func (li *Listener) onDone(ctx context.Context) error {
	li.log.Info("Shutting down listener...")
	close(li.messages)
	return ctx.Err()
}

func (li *Listener) pollBlocks(ctx context.Context) error {
	if li.messages == nil {
		li.log.Info("Not polling events since channel is nil")
		return nil
	}

	storageKey, err := types.CreateStorageKey(&li.conn.metadata, "System", "Events", nil, nil)
	if err != nil {
		return err
	}

	// Get current block
	block, err := li.conn.api.RPC.Chain.GetHeaderLatest()
	if err != nil {
		return err
	}
	currentBlock := uint64(block.Number)

	retryInterval := time.Duration(10) * time.Second
	for {
		select {
		case <-ctx.Done():
			return li.onDone(ctx)
		default:

			li.log.WithField("block", currentBlock).Debug("Processing block")

			// Get block hash
			finalizedHash, err := li.conn.api.RPC.Chain.GetFinalizedHead()
			if err != nil {
				li.log.WithError(err).Error("Failed to fetch finalized head")
				sleep(ctx, retryInterval)
				continue
			}

			// Get block header
			finalizedHeader, err := li.conn.api.RPC.Chain.GetHeader(finalizedHash)
			if err != nil {
				li.log.WithError(err).Error("Failed to fetch header for finalized head")
				sleep(ctx, retryInterval)
				continue
			}

			// Sleep if the block we want comes after the most recently finalized block
			if currentBlock > uint64(finalizedHeader.Number) {
				li.log.WithFields(logrus.Fields{
					"block":  currentBlock,
					"latest": finalizedHeader.Number,
				}).Trace("Block not yet finalized")
				sleep(ctx, retryInterval)
				continue
			}

			// Get hash for latest block, sleep and retry if not ready
			hash, err := li.conn.api.RPC.Chain.GetBlockHash(currentBlock)
			if err != nil {
				li.log.WithFields(logrus.Fields{
					"error": err,
					"block": currentBlock,
				}).Error("Failed to fetch block hash")
				sleep(ctx, retryInterval)
				continue
			}

			var records types.EventRecordsRaw
			_, err = li.conn.api.RPC.State.GetStorage(storageKey, &records, hash)
			if err != nil {
				li.log.WithError(err).Error("Failed to fetch events for block")
				sleep(ctx, retryInterval)
				continue
			}

			li.log.WithField("record", hex.EncodeToString(records)).Trace("Fetched event record")

			events, err := li.eventDecoder.Decode(records)
			if err != nil {
				li.log.WithFields(logrus.Fields{
					"error": err,
					"block": currentBlock,
				}).Error("Failed to decode events for block")
				return err
			}

			li.handleEvents(currentBlock, events)

			currentBlock++
		}
	}
}

func sleep(ctx context.Context, delay time.Duration) {
	select {
	case <-ctx.Done():
	case <-time.After(delay):
	}
}

// Process transfer events in the block
func (li *Listener) handleEvents(blockNumber uint64, events []Event) {

	for i, event := range events {

		li.log.WithFields(logrus.Fields{
			"blockNumber": blockNumber,
			"name":        fmt.Sprintf("%s.%s", event.Name[0], event.Name[1]),
		}).Debug("Witnessed event")

		switch fields := event.Fields.(type) {
		case ETHTransfer:
			buf := bytes.NewBuffer(nil)
			encoder := scale.NewEncoder(buf)
			encoder.Encode(fields.AccountID)
			encoder.Encode(fields.Recipient)
			encoder.Encode(fields.Amount)
			encoder.Encode(uint64(blockNumber))
			encoder.Encode(uint64(i))

			targetAppID := li.config.Targets["eth"]

			li.messages <- chain.Message{AppID: targetAppID, Payload: buf.Bytes()}
		case ERC20Transfer:
			buf := bytes.NewBuffer(nil)
			encoder := scale.NewEncoder(buf)
			encoder.Encode(fields.AccountID)
			encoder.Encode(fields.Recipient)
			encoder.Encode(fields.TokenID)
			encoder.Encode(fields.Amount)
			encoder.Encode(uint64(blockNumber))
			encoder.Encode(uint64(i))

			targetAppID := li.config.Targets["erc20"]

			li.messages <- chain.Message{AppID: targetAppID, Payload: buf.Bytes()}
		}
	}
}
