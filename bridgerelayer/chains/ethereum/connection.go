package ethereum

import (
	"github.com/ethereum/go-ethereum/ethclient"
)

type Connection struct {
	client   *ethclient.Client
}

func NewConnection() (*Connection, err) {

	client, err := ethclient.Dial(viper.GetString("ethereum.endpoint"))
	if err != nil {
		return nil, err
	}

	return &Connection { client }, nil
}

