package substrate

import (
	"github.com/ethereum/go-ethereum/log"
	"github.com/snowfork/polkadot-ethereum/bridgerelayer/chain"
)

type Writer struct {
	conn *Connection
	stop <-chan int
}

func NewWriter(conn *Connection, stop <-chan int) (*Writer, error) {
	return &Writer{
		conn: conn,
		stop: stop,
	}, nil
}

func (wr *Writer) Start() error {
	log.Debug("Starting writer")
	return nil
}

func (wr *Writer) Resolve(_ *chain.Message) {

}

// Submit sends a SCALE-encoded message to an application deployed on the Ethereum network
func (wr *Writer) write(_ string, _ []byte) error {
	return nil
}
