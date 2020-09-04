package substrate

import (
	"bytes"
	"fmt"
	"time"

	log "github.com/sirupsen/logrus"
	"github.com/snowfork/go-substrate-rpc-client/scale"
	types "github.com/snowfork/go-substrate-rpc-client/types"
	"github.com/snowfork/polkadot-ethereum/bridgerelayer/chains/ethereum"
)

// Streamer streams Substrate events
type Streamer struct {
	Core               *Core
	EthRouter          *ethereum.Router
	WebsocketURL       string
	BlockRetryLimit    uint
	BlockRetryInterval time.Duration
}

// NewStreamer returns a new substrate transaction streamer
func NewStreamer(core *Core, er *ethereum.Router, websocketURL string, blockRetryLimit uint, blockRetryInterval uint) *Streamer {
	return &Streamer{
		Core:               core,
		EthRouter:          er,
		WebsocketURL:       websocketURL,
		BlockRetryLimit:    blockRetryLimit,
		BlockRetryInterval: time.Duration(blockRetryInterval) * time.Second,
	}
}

// Start the streamer
func (ss *Streamer) Start() error {
	// Check whether latest is less than starting block
	header, err := ss.Core.API.RPC.Chain.GetHeaderLatest()
	if err != nil {
		return err
	}
	if uint64(header.Number) < ss.Core.StartBlock {
		return fmt.Errorf("starting block (%d) is greater than latest known block (%d)", ss.Core.StartBlock, header.Number)
	}

	err = ss.SubscribeBlocks(int(ss.BlockRetryLimit))
	if err != nil {
		return err
	}

	return nil
}

// SubscribeBlocks ...
func (ss *Streamer) SubscribeBlocks(blockRetryLimit int) error {
	var currentBlock = ss.Core.StartBlock
	var retry = blockRetryLimit
	for {
		select {
		default:
			// No more retries, go to next block
			if retry == 0 {
				log.Error("No more retries")
				return nil
			}

			// Get block hash
			finalizedHash, err := ss.Core.API.RPC.Chain.GetFinalizedHead()
			if err != nil {
				log.Error("Failed to fetch head hash", err)
				retry--
				time.Sleep(ss.BlockRetryInterval)
				continue
			}

			// Get block header
			finalizedHeader, err := ss.Core.API.RPC.Chain.GetHeader(finalizedHash)
			if err != nil {
				log.Error("Failed to fetch finalized header", err)
				retry--
				time.Sleep(ss.BlockRetryInterval)
				continue
			}

			// Sleep if the block we want comes after the most recently finalized block
			if currentBlock > uint64(finalizedHeader.Number) {
				log.WithFields(log.Fields{
					"target": currentBlock,
					"latest": finalizedHeader.Number,
				}).Info("Block not yet finalized")
				time.Sleep(ss.BlockRetryInterval)
				continue
			}

			// Get hash for latest block, sleep and retry if not ready
			hash, err := ss.Core.API.RPC.Chain.GetBlockHash(currentBlock)
			if err != nil && err.Error() == "required result to be 32 bytes, but got 0" {
				time.Sleep(ss.BlockRetryInterval)
				continue
			} else if err != nil {
				log.Error("Failed to query latest block", "block", currentBlock, "err", err)
				retry--
				time.Sleep(ss.BlockRetryInterval)
				continue
			}

			log.Info("Fetching block for events", "hash", hash.Hex())
			key, err := types.CreateStorageKey(&ss.Core.MetaData, "System", "Events", nil, nil)
			if err != nil {
				return err
			}

			var records types.EventRecordsRaw
			_, err = ss.Core.API.RPC.State.GetStorage(key, &records, hash)
			if err != nil {
				return err
			}

			events := Events{}
			err = records.DecodeEventRecords(&ss.Core.MetaData, &events)

			// TODO: DecodeEventRecords will fail if the chain emits events from pallet_scheduler.
			// We'll need to decode those events properly by adding the necessary support in ./events.go.
			// For now we just ignore the error since pallet_scheduler events aren't important for us.
			if err == nil {
				ss.handleEvents(&events)
			}

			currentBlock++
			retry = int(ss.BlockRetryLimit)
		}
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

// TODO: Refactor this code!
func (ss *Streamer) handleEvents(events *Events) {
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

		err := ss.EthRouter.Submit("erc20", buf.Bytes())
		if err != nil {
			log.Error("Error submitting Tx: ", err)
		}
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

		err := ss.EthRouter.Submit("eth", buf.Bytes())
		if err != nil {
			log.Error("Error submitting Tx: ", err)
		}
	}
}
