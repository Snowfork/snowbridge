package substrate

import (
	"fmt"

	"github.com/snowfork/polkadot-ethereum/bridgerelayer/core"
	"github.com/snowfork/polkadot-ethereum/bridgerelayer/crypto/sr25519"
	"github.com/spf13/viper"
)

// Chain ...
type Chain struct {
	listener *Listener
	writer   *Writer
	conn     *Connection
	stop     chan<- int
}

const Name = "Substrate"

// NewChain ...
func NewChain(ethMessages chan core.Message, subMessages chan core.Message) (*Chain, error) {

	// Validate and load configuration
	keys := []string{
		"substrate.endpoint",
		"substrate.private-key",
		"substrate.block-retry-limit",
		"substrate.block-retry-interval",
	}
	for _, key := range keys {
		if !viper.IsSet(key) {
			return nil, fmt.Errorf("Config key %q not set", key)
		}
	}
	endpoint := viper.GetString("substrate.endpoint")
	secret := viper.GetString("substrate.private-key")
	blockRetryLimit := viper.GetUint("substrate.block-retry-limit")
	blockRetryInterval := viper.GetUint("substrate.block-retry-interval")

	// Generate keypair from secret
	kp, err := sr25519.NewKeypairFromSeed(secret, "")
	if err != nil {
		return nil, err
	}

	stop := make(chan int, 0)

	conn := NewConnection(endpoint, kp.AsKeyringPair(), stop)

	listener := NewListener(
		conn,
		subMessages,
		blockRetryLimit,
		blockRetryInterval,
		stop,
	)

	writer, err := NewWriter(conn, ethMessages, stop)
	if err != nil {
		return nil, err
	}

	return &Chain{
		conn:     conn,
		listener: listener,
		writer:   writer,
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
