package substrate

import (
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
func NewChain() (*Chain, error) {

	endpoint := viper.GetString("substrate.endpoint")
	blockRetryLimit := viper.GetUint("substrate.block-retry-limit")
	blockRetryInterval := viper.GetUint("substrate.block-retry-interval")

	kp, err := sr25519.NewKeypairFromSeed("//Alice", "")
	if err != nil {
		return nil, err
	}

	stop := make(chan int, 0)

	conn := NewConnection(endpoint, kp.AsKeyringPair(), stop)

	listener := NewListener(
		conn,
		blockRetryLimit,
		blockRetryInterval,
		stop,
	)

	writer, err := NewWriter(conn, stop)
	if err != nil {
		return nil, err
	}

	return &Chain{
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
