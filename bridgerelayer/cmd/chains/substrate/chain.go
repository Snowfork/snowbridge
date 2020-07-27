package substrate

// import (
// 	"log"

// 	gsrpc "github.com/centrifuge/go-substrate-rpc-client"
// 	gsrpcTypes "github.com/centrifuge/go-substrate-rpc-client/types"

// 	"github.com/snowfork/polkadot-ethereum/bridgerelayer/cmd/chains"
// 	subKeyPair "github.com/snowfork/polkadot-ethereum/bridgerelayer/cmd/keybase/substrate"
// )

// var _ types.Chain = &SubstrateChain{}

// // SubstrateChain ...
// type SubstrateChain struct {
// 	Config   *chains.ChainConfig // The config of the chain
// 	Core     *chains.Core        // The chains connection
// 	Streamer *chains.Streamer    // The streamer of this chain
// 	Router   *chains.Router      // The router of this chain
// 	Stop     chan<- int
// }

// // SubstrateCore holds core SubstrateChain information including credentials
// type SubstrateCore struct {
// 	chains.Core
// 	KeyPair     subKeyPair.KeyPair
// 	API         *gsrpc.SubstrateAPI
// 	MetaData    gsrpcTypes.Metadata
// 	GenesisHash gsrpcTypes.Hash
// 	Logger      *log.Logger
// }

// func Initialize(cfg *types.Config, chainCfg *chains.ChainConfig, sysErr chan<- error) (*SubstrateChain, error) {
// 	cfg, err := parseChainConfig(chainCfg)
// 	if err != nil {
// 		return nil, err
// 	}

// 	core := *SubstrateCore

// 	// Load key pair
// 	kp, err := keystore.KeypairFromAddress(cfg.From, keystore.SubChain, cfg.KeystorePath, cfg.Insecure)
// 	if err != nil {
// 		return nil, err
// 	}

// 	krp := kp.(*subKeyPair.Keypair).AsKeyringPair()
// 	core.KeyPair = krp

// 	// Initialize API
// 	api, err := gsrpc.NewSubstrateAPI(cfg.Url)
// 	if err != nil {
// 		return err
// 	}
// 	core.API = api

// 	// Fetch metadata
// 	meta, err := api.RPC.State.GetMetadataLatest()
// 	if err != nil {
// 		return err
// 	}
// 	core.MetaData = *meta

// 	// Fetch genesis hash
// 	genesisHash, err := c.api.RPC.Chain.GetBlockHash(0)
// 	if err != nil {
// 		return err
// 	}
// 	core.GenesisHash = genesisHash

// 	// Incorporate a more robust logger...
// 	logger := log.Logger
// 	core.Logger = logger

// 	// Fetch header
// 	currBlock, err := api.RPC.Chain.GetHeaderLatest()
// 	if err != nil {
// 		return nil, err
// 	}
// 	startBlock := uint64(currBlock.Number)

// 	// Streamer and Router
// 	stop := make(chan int)
// 	streamer := NewSubstrateStreamer(core, cfg.ChainID, startBlock, bs, stop, sysErr)
// 	router := NewSubstrateRouter(core, stop, sysErr)

// 	return &SubstrateChain{
// 		Config:   cfg,
// 		Core:     core,
// 		Streamer: streamer,
// 		Router:   router,
// 		Stop:     stop,
// 	}, nil
// }

// // Start ...
// func (sc *SubstrateChain) Start() error {
// 	err := sc.Streamer.start()
// 	if err != nil {
// 		return err
// 	}

// 	err = sc.Router.start()
// 	if err != nil {
// 		return err
// 	}

// 	sc.Core.Logger.Debug("Successfully started chain")
// 	return nil
// }
