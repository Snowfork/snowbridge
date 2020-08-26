package substrate

import (
	"fmt"
	"time"
	"bytes"

	log "github.com/sirupsen/logrus"
	types "github.com/snowfork/go-substrate-rpc-client/types"
	"github.com/snowfork/go-substrate-rpc-client/scale"
	"github.com/snowfork/polkadot-ethereum/bridgerelayer/chains/ethereum"

)

// Streamer streams Substrate events
type Streamer struct {
	Core         *Core
	EthRouter    *ethereum.Router
	WebsocketURL string
	BlockRetryLimit uint
	BlockRetryInterval time.Duration
}

// NewStreamer returns a new substrate transaction streamer
func NewStreamer(core *Core, er *ethereum.Router, websocketURL string, blockRetryLimit uint, blockRetryInterval uint) *Streamer {
	return &Streamer{
		Core:         core,
		EthRouter:    er,
		WebsocketURL: websocketURL,
		BlockRetryLimit: blockRetryLimit,
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
			log.Info("DecodeEvents")
			if err != nil {
				log.Error("Error decoding event", err)
				return err
			}

			ss.handleEvents(&events)

			currentBlock++
			retry = int(ss.BlockRetryLimit)
		}
	}
}

type Message struct {
	Sender		types.AccountID
	Recipient 	types.H160
	TokenAddr   types.H160
	Amount		types.U256
}


func (ss *Streamer) handleEvents(events *Events) {
	for _, evt := range events.ERC20_Transfer {
		log.Debug("Handling ERC20 transfer event")

		msg := Message{
			Sender: evt.AccountID,
			Recipient: evt.Recipient,
			TokenAddr: evt.TokenID,
			Amount: evt.Amount,
		}

		buf := bytes.NewBuffer(nil)
		encoder := scale.NewEncoder(buf)
		encoder.Encode(msg)

		ss.EthRouter.Submit("erc20", buf.Bytes())
	}

	for _, evt := range events.ETH_Transfer {
		log.Debug("Handling ETH transfer event")

		msg := Message{
			Sender: evt.AccountID,
			Recipient: evt.Recipient,
			TokenAddr: [20]byte{0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0},
			Amount: evt.Amount,
		}

		buf := bytes.NewBuffer(nil)
		encoder := scale.NewEncoder(buf)
		encoder.Encode(msg)

		ss.EthRouter.Submit("eth", buf.Bytes())
	}
}
