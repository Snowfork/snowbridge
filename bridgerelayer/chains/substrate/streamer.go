package substrate

import (
	"fmt"
	"time"

	log "github.com/sirupsen/logrus"
	types "github.com/snowfork/go-substrate-rpc-client/types"

)

// Streamer streams Substrate events
type Streamer struct {
	Core         *Core
	WebsocketURL string
	BlockRetryLimit uint
	BlockRetryInterval time.Duration
}

// NewStreamer returns a new substrate transaction streamer
func NewStreamer(core *Core, websocketURL string, blockRetryLimit uint, blockRetryInterval uint) *Streamer {
	return &Streamer{
		Core:         core,
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
				log.Info("Block not yet finalized", "target", currentBlock, "latest", finalizedHeader.Number)
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

func (ss *Streamer) handleEvents(events *Events) {
	fmt.Println(events)
}
