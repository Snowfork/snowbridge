package ethereum

import (
	log "github.com/sirupsen/logrus"
	"github.com/snowfork/polkadot-ethereum/bridgerelayer/chains"

	"github.com/snowfork/polkadot-ethereum/bridgerelayer/types"
)

// var _ chains.Chain = &EthChain{}

// EthChain streams the Ethereum blockchain and routes tx data packets
type EthChain struct {
	Config   chains.ChainConfig // The config of this chain
	Streamer Streamer           // The streamer of this chain
	Router   Router             // The router of this chain
}

// NewEthChain initializes a new instance of EthChain
func NewEthChain(config chains.ChainConfig, streamer Streamer, router Router) EthChain {
	return EthChain{
		Config:   config,
		Streamer: streamer,
		Router:   router,
	}
}

// Start starts the chain's Streamer and Router
func (ec EthChain) Start() error {
	errors := make(chan error, 0)
	events := make(chan types.EventData, 0)

	go ec.Streamer.Start(events, errors)

	for {
		select {
		case err := <-errors:
			log.Error(err)
		case event := <-events:
			err := ec.Router.Route(event)
			if err != nil {
				log.Error(err)
			}
		}
	}
}
