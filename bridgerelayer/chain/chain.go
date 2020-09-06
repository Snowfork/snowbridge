package chain

type Chain interface {
	Name() string
	Start() error
	Stop()
}
