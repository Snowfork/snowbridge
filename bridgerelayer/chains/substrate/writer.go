package substrate

import "github.com/ethereum/go-ethereum/log"

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

// Submit sends a SCALE-encoded message to an application deployed on the Ethereum network
func (wr *Writer) Write(_ string, _ []byte) error {
	return nil
}
