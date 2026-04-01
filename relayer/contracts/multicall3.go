package contracts

import (
	"strings"

	"github.com/ethereum/go-ethereum/accounts/abi"
	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/types"
)

const multicall3ABI = `[{"inputs":[{"components":[{"internalType":"address","name":"target","type":"address"},{"internalType":"bool","name":"allowFailure","type":"bool"},{"internalType":"bytes","name":"callData","type":"bytes"}],"internalType":"struct Multicall3.Call3[]","name":"calls","type":"tuple[]"}],"name":"aggregate3","outputs":[{"components":[{"internalType":"bool","name":"success","type":"bool"},{"internalType":"bytes","name":"returnData","type":"bytes"}],"internalType":"struct Multicall3.Result[]","name":"returnData","type":"tuple[]"}],"stateMutability":"payable","type":"function"}]`

type Multicall3Call3 struct {
	Target       common.Address
	AllowFailure bool
	CallData     []byte
}

type Multicall3Result struct {
	Success    bool
	ReturnData []byte
}

type Multicall3 struct {
	address  common.Address
	contract *bind.BoundContract
	abi      abi.ABI
}

func NewMulticall3(address common.Address, backend bind.ContractBackend) (*Multicall3, error) {
	parsedABI, err := abi.JSON(strings.NewReader(multicall3ABI))
	if err != nil {
		return nil, err
	}

	return &Multicall3{
		address:  address,
		contract: bind.NewBoundContract(address, parsedABI, backend, backend, backend),
		abi:      parsedABI,
	}, nil
}

func (m *Multicall3) Address() common.Address {
	return m.address
}

func (m *Multicall3) ABI() abi.ABI {
	return m.abi
}

func (m *Multicall3) Aggregate3(opts *bind.TransactOpts, calls []Multicall3Call3) (*types.Transaction, error) {
	return m.contract.Transact(opts, "aggregate3", calls)
}
