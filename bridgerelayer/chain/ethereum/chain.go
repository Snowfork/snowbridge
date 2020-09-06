package ethereum

import (
	"github.com/snowfork/polkadot-ethereum/bridgerelayer/chain"
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

// NewEthChain initializes a new instance of EthChain
func NewChain(ethMessages chan chain.Message, subMessages chan chain.Message) (*Chain, error) {

	kp, err := secp256k1.NewKeypairFromString(viper.GetString("ethereum.private_key"))
	if err != nil {
		return nil, err
	}

	conn := NewConnection(viper.GetString("ethereum.endpoint"), kp)

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
