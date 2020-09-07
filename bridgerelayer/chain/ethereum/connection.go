package ethereum

import (
	"context"

	"github.com/ethereum/go-ethereum/ethclient"
	log "github.com/sirupsen/logrus"

	"github.com/snowfork/polkadot-ethereum/bridgerelayer/crypto/secp256k1"
)

// Connection ...
type Connection struct {
	endpoint string
	kp       *secp256k1.Keypair
	client   *ethclient.Client
}

// NewConnection ...
func NewConnection(endpoint string, kp *secp256k1.Keypair) *Connection {
	return &Connection{
		endpoint: endpoint,
		kp:       kp,
	}
}

func (conn *Connection) Connect(ctx context.Context) error {

	client, err := ethclient.Dial(conn.endpoint)
	if err != nil {
		return err
	}

	chainID, err := client.NetworkID(ctx)
	if err != nil {
		return err
	}

	log.WithFields(log.Fields{
		"endpoint": conn.endpoint,
		"chainID":  chainID,
	}).Info("Connected to Ethereum chain")

	conn.client = client

	return nil
}

// Close terminates the client connection
func (conn *Connection) Close() {
	if conn.client != nil {
		conn.client.Close()
	}
}
