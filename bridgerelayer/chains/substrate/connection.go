package substrate

import (
	log "github.com/sirupsen/logrus"
	"github.com/spf13/viper"

	gsrpc "github.com/snowfork/go-substrate-rpc-client"
	"github.com/snowfork/go-substrate-rpc-client/signature"
	"github.com/snowfork/go-substrate-rpc-client/types"
)

// Connection ...
type Connection struct {
	endpoint    string
	kp          *signature.KeyringPair
	api         *gsrpc.SubstrateAPI
	metadata    types.Metadata
	genesisHash types.Hash
	stop        <-chan int
}

// NewConnection ...
func NewConnection(endpoint string, kp *signature.KeyringPair, stop <-chan int) *Connection {
	return &Connection{
		endpoint: endpoint,
		kp:       kp,
		stop:     stop,
	}
}

func (co *Connection) Connect() error {

	// Initialize API
	api, err := gsrpc.NewSubstrateAPI(viper.GetString("substrate.endpoint"))
	if err != nil {
		return err
	}
	co.api = api

	// Fetch metadata
	meta, err := api.RPC.State.GetMetadataLatest()
	if err != nil {
		return err
	}
	co.metadata = *meta

	// Fetch genesis hash
	genesisHash, err := api.RPC.Chain.GetBlockHash(0)
	if err != nil {
		return err
	}
	co.genesisHash = genesisHash

	log.WithFields(log.Fields{
		"endpoint": co.endpoint,
	}).Info("Connected to Substrate chain")

	return nil
}

// Close terminates the client connection and stops any running routines
func (co *Connection) Close() {
	// TODO: Fix design issue in GSRPC preventing on-demand closing of connections
}
