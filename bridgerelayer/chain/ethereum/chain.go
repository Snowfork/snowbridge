package ethereum

import (
	"fmt"

	"github.com/snowfork/polkadot-ethereum/bridgerelayer/core"

	"github.com/snowfork/polkadot-ethereum/bridgerelayer/crypto/secp256k1"
	"github.com/spf13/viper"
)

// EthChain streams the Ethereum blockchain and routes tx data packets
type Chain struct {
	listener *Listener
	writer   *Writer
	conn     *Connection
	stop     chan<- int
}

const Name = "Ethereum"

// NewChain initializes a new instance of EthChain
func NewChain(ethMessages chan core.Message, subMessages chan core.Message) (*Chain, error) {

	// Validate and load configuration
	keys := []string{
		"ethereum.endpoint",
		"ethereum.private-key",
	}
	for _, key := range keys {
		if !viper.IsSet(key) {
			return nil, fmt.Errorf("Config key %q not set", key)
		}
	}
	endpoint := viper.GetString("ethereum.endpoint")
	privateKey := viper.GetString("ethereum.private-key")

	kp, err := secp256k1.NewKeypairFromString(privateKey)
	if err != nil {
		return nil, err
	}

	conn := NewConnection(endpoint, kp)

	stop := make(chan int, 0)

	listener, err := NewListener(conn, ethMessages, stop)
	if err != nil {
		return nil, err
	}

	writer, err := NewWriter(conn, subMessages, stop)
	if err != nil {
		return nil, err
	}

	return &Chain{
		listener: listener,
		writer:   writer,
		conn:     conn,
		stop:     stop,
	}, nil
}

func (ch *Chain) Start() error {

	err := ch.conn.Connect()
	if err != nil {
		return err
	}

	err = ch.listener.Start()
	if err != nil {
		return err
	}

	err = ch.writer.Start()
	if err != nil {
		return err
	}

	return nil
}

// Stop signals to any running routines to exit
func (ch *Chain) Stop() {
	close(ch.stop)
	if ch.conn != nil {
		ch.conn.Close()
	}
}

func (ch *Chain) Name() string {
	return Name
}
