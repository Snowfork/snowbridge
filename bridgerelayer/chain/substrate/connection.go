package substrate

import (
	"context"

	"github.com/sirupsen/logrus"

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
	log         *logrus.Entry
}

// NewConnection ...
func NewConnection(endpoint string, kp *signature.KeyringPair, log *logrus.Entry) *Connection {
	return &Connection{
		endpoint: endpoint,
		kp:       kp,
		log:      log,
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

	co.log.WithFields(logrus.Fields{
		"endpoint":    co.endpoint,
		"metaVersion": meta.Version,
	}).Info("Connected to chain")

	return nil
}

func (co *Connection) Close() {
	// TODO: Fix design issue in GSRPC preventing on-demand closing of connections
}
