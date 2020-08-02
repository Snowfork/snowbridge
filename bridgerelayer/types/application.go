package types

import "github.com/ethereum/go-ethereum/accounts/abi"

// Application ...
type Application struct {
	ID  string
	ABI abi.ABI
}

// NewApplication ...
func NewApplication(id string, abi abi.ABI) Application {
	return Application{
		ID:  id,
		ABI: abi,
	}
}
