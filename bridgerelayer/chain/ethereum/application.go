package ethereum

import "github.com/ethereum/go-ethereum/accounts/abi"

// Application contains information about a bridge app deployed to an Ethereum network
type Application struct {
	ID  string
	ABI abi.ABI
}

// NewApplication instantiates a new instance of Application
func NewApplication(id string, abi abi.ABI) Application {
	return Application{
		ID:  id,
		ABI: abi,
	}
}
