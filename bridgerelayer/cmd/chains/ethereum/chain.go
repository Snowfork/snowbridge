package ethereum

import (
	"log"

	"github.com/snowfork/polkadot-ethereum/bridgerelayer/cmd/chains"
	ethKey "github.com/snowfork/polkadot-ethereum/bridgerelayer/cmd/keybase/ethereum"
	"github.com/snowfork/polkadot-ethereum/bridgerelayer/cmd/types"
)

var _ types.Chain = &EthChain{}

// EthChain ...
type EthChain struct {
	Config   *chains.ChainConfig // The config of the chain
	Core     *chains.Core        // The chains connection
	Streamer *chains.Streamer    // The streamer of this chain
	Router   *chains.Router      // The router of this chain
	Stop     chan<- int
}

// EthCore holds core EthChain information including credentials
type EthCore struct {
	chains.Core
	KeyPair ethKey.KeyPair
	Logger  *log.Logger // Should be passed in from bridgerelayer as a universal logger...
}

// Initialize ...
func Initialize(cfg *types.Config, chainCfg *types.ChainConfig, sysErr chan<- error) (*EthChain, error) {
	cfg, err := parseChainConfig(chainCfg)
	if err != nil {
		return nil, err
	}

	core := *EthCore

	kpI, err := keystore.KeypairFromAddress(cfg.from, keystore.EthChain, cfg.keystorePath, chainCfg.Insecure)
	if err != nil {
		return nil, err
	}
	kp, _ := kpI.(*ethKey.Keypair)
	core.KeyPair = kp

	// Incorporate a more robust logger...
	logger := log.Logger
	core.Logger = logger

	// Streamer and Router
	stop := make(chan int)
	streamer := NewEthStreamer(core, cfg, logger, stop, sysErr)
	router := NewEthRouter(core, cfg, logger, stop, sysErr)

	return &EthChain{
		Config:   cfg,
		Core:     core,
		Streamer: streamer,
		Router:   router,
		Stop:     stop,
	}, nil
}

// Start ...
func (ec *EthChain) Start() error {
	err := ec.Streamer.start()
	if err != nil {
		return err
	}

	err = ec.Router.start()
	if err != nil {
		return err
	}

	ec.Core.Logger.Debug("Successfully started chain")
	return nil
}
