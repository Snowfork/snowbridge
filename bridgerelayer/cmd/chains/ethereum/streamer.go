package ethereum

import (
	"context"
	"log"

	ethTypes "github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/ethclient"

	"github.com/snowfork/polkadot-ethereum/bridgerelayer/cmd/types"
)

// EthStreamer ...
type EthStreamer struct {
	types.Streamer
	WebsocketURL string
	stop         <-chan int
	sysErr       chan<- error
}

// NewEthStreamer returns a new ethereum transaction streamer
func NewEthStreamer(websocketURL string, stop <-chan int, sysErr chan<- error) EthStreamer {
	return EthStreamer{
		WebsocketURL: websocketURL,
		stop:         stop,
		sysErr:       sysErr,
	}
}

// Start ...
func (es *EthStreamer) Start() error {

	client, err := ethclient.Dial(el.WebsocketURL)
	if err != nil {
		log.Fatal(err)
		return err
	}

	err = SubscribeBlocks()
	if err != nil {
		return err
	}

	return nil
}

// SubscribeBlocks ...
func (es *EthStreamer) SubscribeBlocks() error {
	headers := make(chan *ethTypes.Header)
	sub, err := client.SubscribeNewHead(context.Background(), headers)
	if err != nil {
		log.Fatal(err)
	}

	for {
		select {
		case err := <-sub.Err():
			log.Fatal(err)
		case header := <-headers:

			block, err := client.BlockByHash(context.Background(), header.Hash())
			if err != nil {
				log.Fatal(err)
			}

			for _, tx := range block.Transactions() {
				packet, err := BuildPacket(tx, block)
				if err != nil {
					log.Fatal(err)
				}

				// TODO: send packet to router
			}
		}
	}
	return nil
}
