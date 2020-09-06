package substrate

import (
	"bytes"
	"time"

	log "github.com/sirupsen/logrus"
	"github.com/snowfork/go-substrate-rpc-client/scale"
	types "github.com/snowfork/go-substrate-rpc-client/types"
	"github.com/snowfork/polkadot-ethereum/bridgerelayer/chain"

)

// Listener streams Substrate events
type Listener struct {
	conn               *Connection
	channel            chain.Channel
	blockRetryLimit    uint
	blockRetryInterval time.Duration
	stop               <-chan int
}

// NewListener returns a new substrate transaction streamer
func NewListener(conn *Connection, blockRetryLimit uint, blockRetryInterval uint, stop <-chan int) *Listener {
	return &Listener{
		conn:               conn,
		blockRetryLimit:    blockRetryLimit,
		blockRetryInterval: time.Duration(blockRetryInterval) * time.Second,
		stop:               stop,
	}
}

// Start the listener
func (li *Listener) Start() error {

	go func() {
		err := li.pollBlocks()
		if err != nil {
			log.WithFields(log.Fields{
				"error": err,
			}).Error("Error while polling substrate blocks")
		}
	}()

	return nil
}

func (li *Listener) pollBlocks() error {

	// Get current block
	block, err := li.conn.api.RPC.Chain.GetHeaderLatest()
	if err != nil {
		return err
	}

	currentBlock := uint64(block.Number)
	retry := int(li.blockRetryLimit)

	for {
		select {
		case <-li.stop:
			log.Info("Polling stopped")
			return nil
		default:
			// No more retries, go to next block
			if retry == 0 {
				log.Error("No more retries")
				return nil
			}

			// Get block hash
			finalizedHash, err := li.conn.api.RPC.Chain.GetFinalizedHead()
			if err != nil {
				log.Error("Failed to fetch head hash", err)
				retry--
				time.Sleep(li.blockRetryInterval)
				continue
			}

			// Get block header
			finalizedHeader, err := li.conn.api.RPC.Chain.GetHeader(finalizedHash)
			if err != nil {
				log.Error("Failed to fetch finalized header", err)
				retry--
				time.Sleep(li.blockRetryInterval)
				continue
			}

			// Sleep if the block we want comes after the most recently finalized block
			if currentBlock > uint64(finalizedHeader.Number) {
				log.WithFields(log.Fields{
					"target": currentBlock,
					"latest": finalizedHeader.Number,
				}).Info("Block not yet finalized")
				time.Sleep(li.blockRetryInterval)
				continue
			}

			// Get hash for latest block, sleep and retry if not ready
			hash, err := li.conn.api.RPC.Chain.GetBlockHash(currentBlock)
			if err != nil && err.Error() == "required result to be 32 bytes, but got 0" {
				time.Sleep(li.blockRetryInterval)
				continue
			} else if err != nil {
				log.Error("Failed to query latest block", "block", currentBlock, "err", err)
				retry--
				time.Sleep(li.blockRetryInterval)
				continue
			}

			log.Info("Fetching block for events", "hash", hash.Hex())
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

		// err := ss.EthRouter.Submit("erc20", buf.Bytes())
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

		// err := ss.EthRouter.Submit("eth", buf.Bytes())
	}
}

func (li *Listener) setChannel(ch chain.Channel) {
	li.channel = ch
}
