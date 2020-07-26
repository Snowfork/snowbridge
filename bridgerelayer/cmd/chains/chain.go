package chains

import "github.com/snowfork/polkadot-ethereum/bridgerelayer/cmd/types"

// Chain ...
type Chain struct {
	Streamer Streamer // The streamer of this chain
	Router   Router   // The router of the chain
	// Keybase Keybase
	// stop     chan<- int
}

// // Core contains important information for each chain, including credentials
// // type Core interface {
// // 	Keypair() *keybase.Keypair
// // }

// Streamer streams transactions from a blockchain and passes them to the router
type Streamer interface {
	Start() error
}

// Router packages transaction data as packets and relays them to the bridge
type Router interface {
	Route(types.EventData) (error, types.Packet)
}
