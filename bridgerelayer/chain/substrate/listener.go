package substrate

import (
	"bytes"
	"context"
	"fmt"
	"time"

	"golang.org/x/sync/errgroup"

	log "github.com/sirupsen/logrus"
	"github.com/snowfork/go-substrate-rpc-client/scale"
	types "github.com/snowfork/go-substrate-rpc-client/types"
	"github.com/snowfork/polkadot-ethereum/bridgerelayer/chain"
	"github.com/snowfork/polkadot-ethereum/bridgerelayer/core"
)

// Listener streams Substrate events
type Listener struct {
	conn               *Connection
	blockRetryLimit    uint
	blockRetryInterval time.Duration
	messages           chan<- core.Message
}

// NewListener returns a new substrate transaction streamer
func NewListener(conn *Connection, messages chan<- core.Message, blockRetryLimit uint, blockRetryInterval uint) *Listener {
	return &Listener{
		conn:               conn,
		blockRetryLimit:    blockRetryLimit,
		blockRetryInterval: time.Duration(blockRetryInterval) * time.Second,
		messages:           messages,
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
	retry := int(li.blockRetryLimit)

	for {
		select {
		case <-ctx.Done():
			return ctx.Err()
		default:
			if retry == 0 {
				return fmt.Errorf("No more retries for polling Substrate")
			}

			// Get block hash
			finalizedHash, err := li.conn.api.RPC.Chain.GetFinalizedHead()
			if err != nil {
				log.WithFields(log.Fields{
					"error": err,
				}).Error("Failed to fetch head hash")
				retry--
				sleep(ctx, li.blockRetryInterval)
				continue
			}

			// Get block header
			finalizedHeader, err := li.conn.api.RPC.Chain.GetHeader(finalizedHash)
			if err != nil {
				log.WithFields(log.Fields{
					"error": err,
				}).Error("Failed to fetch finalized header")
				retry--
				sleep(ctx, li.blockRetryInterval)
				continue
			}

			// Sleep if the block we want comes after the most recently finalized block
			if currentBlock > uint64(finalizedHeader.Number) {
				log.WithFields(log.Fields{
					"target": currentBlock,
					"latest": finalizedHeader.Number,
				}).Debug("Block not yet finalized")
				sleep(ctx, li.blockRetryInterval)
				continue
			}

			// Get hash for latest block, sleep and retry if not ready
			hash, err := li.conn.api.RPC.Chain.GetBlockHash(currentBlock)
			if err != nil && err.Error() == "required result to be 32 bytes, but got 0" {
				sleep(ctx, li.blockRetryInterval)
				continue
			} else if err != nil {
				log.WithFields(log.Fields{
					"error": err,
					"block": currentBlock,
				}).Error("Failed to query latest block")
				retry--
				sleep(ctx, li.blockRetryInterval)
				continue
			}

			log.WithFields(log.Fields{
				"block": currentBlock,
				"hash":  hash.Hex(),
			}).Info("Fetching events for block")

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

			if err == nil {
				li.handleEvents(&events)
			}

			currentBlock++
			retry = int(li.blockRetryLimit)
		}
	}
}

func sleep(ctx context.Context, delay time.Duration) {
	select {
	case <-ctx.Done():
	case <-time.After(delay):
	}
}

// These are the data packages we submit to the Ethereum contracts
type Erc20Message struct {
	Sender    types.AccountID
	Recipient types.H160
	TokenAddr types.H160
	Amount    types.U256
}

type EthMessage struct {
	Sender    types.AccountID
	Recipient types.H160
	Amount    types.U256
}

func (li *Listener) handleEvents(events *Events) {
	for _, evt := range events.ERC20_Transfer {
		log.Debug("Handling ERC20 transfer event")

		msg := Erc20Message{
			Sender:    evt.AccountID,
			Recipient: evt.Recipient,
			TokenAddr: evt.TokenID,
			Amount:    evt.Amount,
		}

		buf := bytes.NewBuffer(nil)
		encoder := scale.NewEncoder(buf)
		encoder.Encode(msg)

		li.messages <- core.Message{AppID: chain.Erc20AppID, Payload: buf.Bytes()}
	}

	for _, evt := range events.ETH_Transfer {
		log.Debug("Handling ETH transfer event")

		msg := EthMessage{
			Sender:    evt.AccountID,
			Recipient: evt.Recipient,
			Amount:    evt.Amount,
		}

		buf := bytes.NewBuffer(nil)
		encoder := scale.NewEncoder(buf)
		encoder.Encode(msg)

		li.messages <- core.Message{AppID: chain.EthAppID, Payload: buf.Bytes()}
	}
}
