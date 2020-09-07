package substrate

import (
	"context"

	log "github.com/sirupsen/logrus"

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
}

// NewConnection ...
func NewConnection(endpoint string, kp *signature.KeyringPair) *Connection {
	return &Connection{
		endpoint: endpoint,
		kp:       kp,
	}
}

func (co *Connection) Connect(_ context.Context) error {

	// Initialize API
	api, err := gsrpc.NewSubstrateAPI(co.endpoint)
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

func (co *Connection) Close() {
	// TODO: Fix design issue in GSRPC preventing on-demand closing of connections
}
