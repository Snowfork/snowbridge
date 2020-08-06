package substrate

// import (
// 	"errors"
// 	"fmt"
// 	"time"

// 	"github.com/snowfork/polkadot-ethereum/bridgerelayer/chains"
// )

// type SubstrateStreamer struct {
// 	chains.Streamer
// 	WebsocketURL string
// 	Core         chains.Core
// 	stop         <-chan int
// 	sysErr       chan<- error
// }

// // NewSubstrateStreamer returns a new substrate transaction streamer
// func NewSubstrateStreamer(core chains.Core, websocketURL string, stop <-chan int, sysErr chan<- error) SubstrateStreamer {
// 	return SubstrateStreamer{
// 		Core:         core,
// 		WebsocketURL: websocketURL,
// 		stop:         stop,
// 		sysErr:       sysErr,
// 	}
// }

// func (ss *SubstrateStreamer) Start() error {
// 	// Check whether latest is less than starting block
// 	header, err := ss.Core.API.RPC.Chain.GetHeaderLatest()
// 	if err != nil {
// 		return err
// 	}
// 	if uint64(header.Number) < ss.Core.startBlock {
// 		return fmt.Errorf("starting block (%d) is greater than latest known block (%d)", ss.Core.StartBlock, header.Number)
// 	}

// 	err = ss.SubscribeBlocks(BlockRetryLimit)
// 	if err != nil {
// 		return err
// 	}

// 	return nil
// }

// // SubscribeBlocks ...
// func (ss *SubstrateStreamer) SubscribeBlocks(blockRetryLimit int) error {
// 	var currentBlock = ss.Core.startBlock
// 	var retry = blockRetryLimit
// 	for {
// 		select {
// 		case <-ss.stop:
// 			return errors.New("closed")
// 		default:
// 			// No more retries, go to next block
// 			if retry == 0 {
// 				ss.sysErr <- fmt.Errorf("block retries exceeded", ss.Core.ChainID, ss.Core.Name)
// 				return nil
// 			}

// 			// Get block header
// 			finalizedHeader, err := ss.Core.API.RPC.Chain.GetHeader(finalizedHash)
// 			if err != nil {
// 				l.log.Error("Failed to fetch finalized header", "err", err)
// 				retry--
// 				time.Sleep(BlockRetryInterval)
// 				continue
// 			}

// 			// Get block hash
// 			finalizedHash, err := ss.Core.API.RPC.Chain.GetFinalizedHead()
// 			if err != nil {
// 				l.log.Error("Failed to fetch head hash", "err", err)
// 				retry--
// 				time.Sleep(BlockRetryInterval)
// 				continue
// 			}

// 			// Sleep if the block we want comes after the most recently finalized block
// 			if currentBlock > uint64(finalizedHeader.Number) {
// 				ss.Core.Logger.Trace("Block not yet finalized", "target", currentBlock, "latest", finalizedHeader.Number)
// 				time.Sleep(BlockRetryInterval)
// 				continue
// 			}

// 			// Get hash for latest block, sleep and retry if not ready
// 			hash, err := ss.Core.API.RPC.Chain.GetBlockHash(currentBlock)
// 			if err != nil && err.Error() == ErrBlockNotReady.Error() {
// 				time.Sleep(BlockRetryInterval)
// 				continue
// 			} else if err != nil {
// 				ss.Core.Logger.Error("Failed to query latest block", "block", currentBlock, "err", err)
// 				retry--
// 				time.Sleep(BlockRetryInterval)
// 				continue
// 			}

// 			// 1. Get transactions from block...

// 			// 2. Send transaction and block to Router for packaging and relay...

// 			currentBlock++
// 			retry = BlockRetryLimit
// 		}
// 	}
// }
