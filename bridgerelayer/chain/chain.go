package chain

type Message struct {
	Payload interface{}
}

type Channel interface {
	Send(msg *Message)
}

type Chain interface {
	Name() string
	Start() error
	Stop()
}
