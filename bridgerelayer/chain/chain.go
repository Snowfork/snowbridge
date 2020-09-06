package chain

type Message struct {
	Payload interface{}
}

type Chain interface {
	Name() string
	Start() error
	Stop()
}

type Writer interface {
	Write(msg Message)
}
