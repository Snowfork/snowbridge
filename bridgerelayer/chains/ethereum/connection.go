package ethereum

import (
	"context"

	"github.com/ethereum/go-ethereum/ethclient"
	log "github.com/sirupsen/logrus"
	"github.com/spf13/viper"
)

// Connection ...
type Connection struct {
	client *ethclient.Client
}

// NewConnection ...
func NewConnection() *Connection {
	return &Connection{}
}

func (conn *Connection) Connect() error {

	endpoint := viper.GetString("ethereum.endpoint")

	client, err := ethclient.Dial(endpoint)
	if err != nil {
		return err
	}

	chainID, err := conn.client.NetworkID(context.Background())
	if err != nil {
		return err
	}

	log.WithFields(
		log.Fields{
			"endpoint": endpoint,
			"chainID":  chainID,
		},
	).Info("Connected to Ethereum chain")

	conn.client = client
}
