package substrate

import (
	"bytes"
	"context"
	"time"

	"golang.org/x/sync/errgroup"

	"github.com/sirupsen/logrus"
	"github.com/snowfork/go-substrate-rpc-client/scale"
	types "github.com/snowfork/go-substrate-rpc-client/types"
	"github.com/snowfork/polkadot-ethereum/bridgerelayer/chain"
)

// Listener streams Substrate events
type Listener struct {
	conn               *Connection
	blockRetryLimit    uint
	blockRetryInterval time.Duration
	messages           chan<- chain.Message
	log                *logrus.Entry
}

// NewListener returns a new substrate transaction streamer
func NewListener(conn *Connection, messages chan<- chain.Message, blockRetryLimit uint, blockRetryInterval uint, log *logrus.Entry) *Listener {
	return &Listener{
		conn:               conn,
		blockRetryLimit:    blockRetryLimit,
		blockRetryInterval: time.Duration(blockRetryInterval) * time.Second,
		messages:           messages,
		log:                log,
	}
}

// Start the listener
func (li *Listener) Start(ctx context.Context, eg *errgroup.Group) error {

	eg.Go(func() error {
		return li.pollBlocks(ctx)
	})

	return nil
}

func (li *Listener) pollBlocks(ctx context.Context) error {

	// Get current block
	block, err := li.conn.api.RPC.Chain.GetHeaderLatest()
	if err != nil {
		return err
	}

	currentBlock := uint64(block.Number)

	for {
		select {
		case <-ctx.Done():
			return ctx.Err()
		default:

			// Get block hash
			finalizedHash, err := li.conn.api.RPC.Chain.GetFinalizedHead()
			if err != nil {
				li.log.WithError(err).Error("Failed to fetch head hash")
				sleep(ctx, li.blockRetryInterval)
				continue
			}

			// Get block header
			finalizedHeader, err := li.conn.api.RPC.Chain.GetHeader(finalizedHash)
			if err != nil {
				li.log.WithError(err).Error("Failed to fetch finalized header")
				sleep(ctx, li.blockRetryInterval)
				continue
			}

			// Sleep if the block we want comes after the most recently finalized block
			if currentBlock > uint64(finalizedHeader.Number) {
				li.log.WithFields(logrus.Fields{
					"target": currentBlock,
					"latest": finalizedHeader.Number,
				}).Debug("Block not yet finalized")
				sleep(ctx, li.blockRetryInterval)
				continue
			}

			// Get hash for latest block, sleep and retry if not ready
			hash, err := li.conn.api.RPC.Chain.GetBlockHash(currentBlock)
			if err != nil {
				li.log.WithFields(logrus.Fields{
					"error": err,
					"block": currentBlock,
				}).Error("Failed to query latest block hash")
				sleep(ctx, li.blockRetryInterval)
				continue
			}

			li.log.WithFields(logrus.Fields{
				"block": currentBlock,
			}).Debug("Fetching events for block")

			key, err := types.CreateStorageKey(&li.conn.metadata, "System", "Events", nil, nil)
			if err != nil {
				return err
			}

			var records types.EventRecordsRaw
			_, err = li.conn.api.RPC.State.GetStorage(key, &records, hash)
			if err != nil {
				return err
			}

			events := Events{}
			err = records.DecodeEventRecords(&li.conn.metadata, &events)
			if err != nil {
				li.log.WithFields(logrus.Fields{
					"error": err,
					"block": currentBlock,
				}).Error("Failed to decode events for block")
			} else {
				li.handleEvents(&events)
			}

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

func (li *Listener) handleEvents(events *Events) {
	for _, evt := range events.ERC20_Transfer {
		li.log.Info("Handling Transfer event")

		buf := bytes.NewBuffer(nil)
		encoder := scale.NewEncoder(buf)
		encoder.Encode(evt.AccountID)
		encoder.Encode(evt.Recipient)
		encoder.Encode(evt.TokenID)
		encoder.Encode(evt.Amount)

		li.messages <- chain.Message{AppID: chain.Erc20AppID, Payload: buf.Bytes()}
	}

	for _, evt := range events.ETH_Transfer {
		li.log.Info("Handling Transfer event")

		buf := bytes.NewBuffer(nil)
		encoder := scale.NewEncoder(buf)
		encoder.Encode(evt.AccountID)
		encoder.Encode(evt.Recipient)
		encoder.Encode(evt.Amount)

		li.messages <- chain.Message{AppID: chain.EthAppID, Payload: buf.Bytes()}
	}
}
