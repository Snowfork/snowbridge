// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package substrate

import (
	"context"
	"time"

	"golang.org/x/sync/errgroup"

	"github.com/sirupsen/logrus"
	gsrpcOffchain "github.com/snowfork/go-substrate-rpc-client/v2/rpc/offchain"
	"github.com/snowfork/go-substrate-rpc-client/v2/types"
	"github.com/snowfork/polkadot-ethereum/relayer/chain"
	"github.com/snowfork/polkadot-ethereum/relayer/chain/substrate/digest"
	"github.com/snowfork/polkadot-ethereum/relayer/chain/substrate/offchain"
)

type Listener struct {
	config   *Config
	conn     *Connection
	messages chan<- chain.Message
	log      *logrus.Entry
}

func NewListener(config *Config, conn *Connection, messages chan<- chain.Message, log *logrus.Entry) *Listener {
	return &Listener{
		config:   config,
		conn:     conn,
		messages: messages,
		log:      log,
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

	// Get current block
	block, err := li.conn.api.RPC.Chain.GetHeaderLatest()
	if err != nil {
		return err
	}
	currentBlock := uint32(block.Number)

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
			if currentBlock > uint32(finalizedHeader.Number) {
				li.log.WithFields(logrus.Fields{
					"block":  currentBlock,
					"latest": finalizedHeader.Number,
				}).Trace("Block not yet finalized")
				sleep(ctx, retryInterval)
				continue
			}

			if uint32(finalizedHeader.Number)%li.config.CommitInterval != 0 {
				currentBlock++
				continue
			}

			auxiliaryDigestItem, err := getAuxiliaryDigestItem(finalizedHeader)
			if err != nil {
				sleep(ctx, retryInterval)
				continue
			}

			if auxiliaryDigestItem.IsCommitmentHash {
				storageKey, err := offchain.MakeStorageKey(auxiliaryDigestItem.AsCommitmentHash)
				if err != nil {
					li.log.WithError(err).Error("Failed to create storage key")
					return err
				}
				li.conn.api.RPC.Offchain.LocalStorageGet(gsrpcOffchain.Persistent, storageKey)
			}

			currentBlock++
		}
	}
}

func getAuxiliaryDigestItem(header *types.Header) (*digest.AuxiliaryDigestItem, error) {
	for _, digestItem := range header.Digest {
		if digestItem.IsOther {
			auxDigestItem, err := digest.DecodeFromBytes(digestItem.AsOther)
			if err != nil {
				return nil, err
			}

			return &auxDigestItem, nil
		}
	}
	return nil, nil
}

func sleep(ctx context.Context, delay time.Duration) {
	select {
	case <-ctx.Done():
	case <-time.After(delay):
	}
}
