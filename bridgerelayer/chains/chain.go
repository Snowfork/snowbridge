package chains

import (
	"github.com/snowfork/polkadot-ethereum/bridgerelayer/types"
)

// Chain is a connection to a blockchain network
type Chain struct {
	Config   ChainConfig // The config of this chain
	Streamer Streamer    // The streamer of this chain
	Router   Router      // The router of the chain
	// stop     chan<- int
}

// Streamer streams transactions from a blockchain and passes them to the router
type Streamer interface {
	Start() error
}

// Router packages transaction data as packets and relays them to the bridge
type Router interface {
	Route(types.EventData) (error, types.Packet)
}
