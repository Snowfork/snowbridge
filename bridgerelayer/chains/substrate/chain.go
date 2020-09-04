package substrate

import (
	"github.com/spf13/viper"

	"sync"

	gsrpc "github.com/snowfork/go-substrate-rpc-client"
	gsrpcTypes "github.com/snowfork/go-substrate-rpc-client/types"
	subKeyPair "github.com/snowfork/polkadot-ethereum/bridgerelayer/keybase/substrate"

	"github.com/snowfork/polkadot-ethereum/bridgerelayer/chains/ethereum"
)

// Core holds core SubstrateChain information including credentials
type Core struct {
	KeyPair     *subKeyPair.Keypair
	API         *gsrpc.SubstrateAPI
	MetaData    gsrpcTypes.Metadata
	GenesisHash gsrpcTypes.Hash
	StartBlock  uint64
}

// Chain ...
type Chain struct {
	Streamer *Streamer // The streamer of this chain
}

// NewChain ...
func NewChain(er *ethereum.Router) (*Chain, error) {

	core := Core{}

	krp, err := subKeyPair.NewKeypairFromSeed("//Alice")
	if err != nil {
		return nil, err
	}
	core.KeyPair = krp

	// Initialize API
	api, err := gsrpc.NewSubstrateAPI(viper.GetString("substrate.endpoint"))
	if err != nil {
		return nil, err
	}
	core.API = api

	// Fetch metadata
	meta, err := api.RPC.State.GetMetadataLatest()
	if err != nil {
		return nil, err
	}
	core.MetaData = *meta

	// Fetch genesis hash
	genesisHash, err := api.RPC.Chain.GetBlockHash(0)
	if err != nil {
		return nil, err
	}
	core.GenesisHash = genesisHash

	// Fetch header
	currBlock, err := api.RPC.Chain.GetHeaderLatest()
	if err != nil {
		return nil, err
	}
	core.StartBlock = uint64(currBlock.Number)

	streamer := NewStreamer(
		&core,
		er,
		viper.GetString("substrate.endpoint"),
		viper.GetUint("substrate.block-retry-limit"),
		viper.GetUint("substrate.block-retry-interval"),
	)
	router := Router{Core: &core}

	return &Chain{
		Streamer: streamer,
		Router:   &router,
	}, nil
}

// Start ...
func (sc *Chain) Start(wg *sync.WaitGroup) error {
	defer wg.Done()

	go sc.Streamer.Start()

	return nil
}
