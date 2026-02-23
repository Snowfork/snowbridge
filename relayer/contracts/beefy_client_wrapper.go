// Code generated - DO NOT EDIT.
// This file is a generated binding and any manual changes will be lost.

package contracts

import (
	"errors"
	"math/big"
	"strings"

	ethereum "github.com/ethereum/go-ethereum"
	"github.com/ethereum/go-ethereum/accounts/abi"
	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/event"
)

// Reference imports to suppress errors if they are not otherwise used.
var (
	_ = errors.New
	_ = big.NewInt
	_ = strings.NewReader
	_ = ethereum.NotFound
	_ = bind.Bind
	_ = common.Big1
	_ = types.BloomLookup
	_ = event.NewSubscription
	_ = abi.ConvertType
)

// BeefyClientWrapperMetaData contains all meta data concerning the BeefyClientWrapper contract.
var BeefyClientWrapperMetaData = &bind.MetaData{
	ABI: "[{\"type\":\"constructor\",\"inputs\":[{\"name\":\"_gateway\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"_maxGasPrice\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"_maxRefundAmount\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"_refundTarget\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"_ticketTimeout\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"nonpayable\"},{\"type\":\"receive\",\"stateMutability\":\"payable\"},{\"type\":\"function\",\"name\":\"commitPrevRandao\",\"inputs\":[{\"name\":\"commitmentHash\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"gateway\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"address\",\"internalType\":\"address\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"highestPendingBlock\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"highestPendingBlockTimestamp\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"maxGasPrice\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"maxRefundAmount\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"pendingTickets\",\"inputs\":[{\"name\":\"\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"}],\"outputs\":[{\"name\":\"owner\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"creditedCost\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"createdAt\",\"type\":\"uint64\",\"internalType\":\"uint64\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"refundTarget\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"ticketTimeout\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"event\",\"name\":\"CostCredited\",\"inputs\":[{\"name\":\"relayer\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"commitmentHash\",\"type\":\"bytes32\",\"indexed\":true,\"internalType\":\"bytes32\"},{\"name\":\"cost\",\"type\":\"uint256\",\"indexed\":false,\"internalType\":\"uint256\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"SubmissionRefunded\",\"inputs\":[{\"name\":\"relayer\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"progress\",\"type\":\"uint256\",\"indexed\":false,\"internalType\":\"uint256\"},{\"name\":\"refundAmount\",\"type\":\"uint256\",\"indexed\":false,\"internalType\":\"uint256\"}],\"anonymous\":false},{\"type\":\"error\",\"name\":\"InsufficientProgress\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"InvalidAddress\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"NotTicketOwner\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"TicketAlreadyOwned\",\"inputs\":[]}]",
}

// BeefyClientWrapperABI is the input ABI used to generate the binding from.
// Deprecated: Use BeefyClientWrapperMetaData.ABI instead.
var BeefyClientWrapperABI = BeefyClientWrapperMetaData.ABI

// BeefyClientWrapper is an auto generated Go binding around an Ethereum contract.
type BeefyClientWrapper struct {
	BeefyClientWrapperCaller     // Read-only binding to the contract
	BeefyClientWrapperTransactor // Write-only binding to the contract
	BeefyClientWrapperFilterer   // Log filterer for contract events
}

// BeefyClientWrapperCaller is an auto generated read-only Go binding around an Ethereum contract.
type BeefyClientWrapperCaller struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// BeefyClientWrapperTransactor is an auto generated write-only Go binding around an Ethereum contract.
type BeefyClientWrapperTransactor struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// BeefyClientWrapperFilterer is an auto generated log filtering Go binding around an Ethereum contract events.
type BeefyClientWrapperFilterer struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// BeefyClientWrapperSession is an auto generated Go binding around an Ethereum contract,
// with pre-set call and transact options.
type BeefyClientWrapperSession struct {
	Contract     *BeefyClientWrapper // Generic contract binding to set the session for
	CallOpts     bind.CallOpts       // Call options to use throughout this session
	TransactOpts bind.TransactOpts   // Transaction auth options to use throughout this session
}

// BeefyClientWrapperCallerSession is an auto generated read-only Go binding around an Ethereum contract,
// with pre-set call options.
type BeefyClientWrapperCallerSession struct {
	Contract *BeefyClientWrapperCaller // Generic contract caller binding to set the session for
	CallOpts bind.CallOpts             // Call options to use throughout this session
}

// BeefyClientWrapperTransactorSession is an auto generated write-only Go binding around an Ethereum contract,
// with pre-set transact options.
type BeefyClientWrapperTransactorSession struct {
	Contract     *BeefyClientWrapperTransactor // Generic contract transactor binding to set the session for
	TransactOpts bind.TransactOpts             // Transaction auth options to use throughout this session
}

// BeefyClientWrapperRaw is an auto generated low-level Go binding around an Ethereum contract.
type BeefyClientWrapperRaw struct {
	Contract *BeefyClientWrapper // Generic contract binding to access the raw methods on
}

// BeefyClientWrapperCallerRaw is an auto generated low-level read-only Go binding around an Ethereum contract.
type BeefyClientWrapperCallerRaw struct {
	Contract *BeefyClientWrapperCaller // Generic read-only contract binding to access the raw methods on
}

// BeefyClientWrapperTransactorRaw is an auto generated low-level write-only Go binding around an Ethereum contract.
type BeefyClientWrapperTransactorRaw struct {
	Contract *BeefyClientWrapperTransactor // Generic write-only contract binding to access the raw methods on
}

// NewBeefyClientWrapper creates a new instance of BeefyClientWrapper, bound to a specific deployed contract.
func NewBeefyClientWrapper(address common.Address, backend bind.ContractBackend) (*BeefyClientWrapper, error) {
	contract, err := bindBeefyClientWrapper(address, backend, backend, backend)
	if err != nil {
		return nil, err
	}
	return &BeefyClientWrapper{BeefyClientWrapperCaller: BeefyClientWrapperCaller{contract: contract}, BeefyClientWrapperTransactor: BeefyClientWrapperTransactor{contract: contract}, BeefyClientWrapperFilterer: BeefyClientWrapperFilterer{contract: contract}}, nil
}

// NewBeefyClientWrapperCaller creates a new read-only instance of BeefyClientWrapper, bound to a specific deployed contract.
func NewBeefyClientWrapperCaller(address common.Address, caller bind.ContractCaller) (*BeefyClientWrapperCaller, error) {
	contract, err := bindBeefyClientWrapper(address, caller, nil, nil)
	if err != nil {
		return nil, err
	}
	return &BeefyClientWrapperCaller{contract: contract}, nil
}

// NewBeefyClientWrapperTransactor creates a new write-only instance of BeefyClientWrapper, bound to a specific deployed contract.
func NewBeefyClientWrapperTransactor(address common.Address, transactor bind.ContractTransactor) (*BeefyClientWrapperTransactor, error) {
	contract, err := bindBeefyClientWrapper(address, nil, transactor, nil)
	if err != nil {
		return nil, err
	}
	return &BeefyClientWrapperTransactor{contract: contract}, nil
}

// NewBeefyClientWrapperFilterer creates a new log filterer instance of BeefyClientWrapper, bound to a specific deployed contract.
func NewBeefyClientWrapperFilterer(address common.Address, filterer bind.ContractFilterer) (*BeefyClientWrapperFilterer, error) {
	contract, err := bindBeefyClientWrapper(address, nil, nil, filterer)
	if err != nil {
		return nil, err
	}
	return &BeefyClientWrapperFilterer{contract: contract}, nil
}

// bindBeefyClientWrapper binds a generic wrapper to an already deployed contract.
func bindBeefyClientWrapper(address common.Address, caller bind.ContractCaller, transactor bind.ContractTransactor, filterer bind.ContractFilterer) (*bind.BoundContract, error) {
	parsed, err := BeefyClientWrapperMetaData.GetAbi()
	if err != nil {
		return nil, err
	}
	return bind.NewBoundContract(address, *parsed, caller, transactor, filterer), nil
}

// Call invokes the (constant) contract method with params as input values and
// sets the output to result. The result type might be a single field for simple
// returns, a slice of interfaces for anonymous returns and a struct for named
// returns.
func (_BeefyClientWrapper *BeefyClientWrapperRaw) Call(opts *bind.CallOpts, result *[]interface{}, method string, params ...interface{}) error {
	return _BeefyClientWrapper.Contract.BeefyClientWrapperCaller.contract.Call(opts, result, method, params...)
}

// Transfer initiates a plain transaction to move funds to the contract, calling
// its default method if one is available.
func (_BeefyClientWrapper *BeefyClientWrapperRaw) Transfer(opts *bind.TransactOpts) (*types.Transaction, error) {
	return _BeefyClientWrapper.Contract.BeefyClientWrapperTransactor.contract.Transfer(opts)
}

// Transact invokes the (paid) contract method with params as input values.
func (_BeefyClientWrapper *BeefyClientWrapperRaw) Transact(opts *bind.TransactOpts, method string, params ...interface{}) (*types.Transaction, error) {
	return _BeefyClientWrapper.Contract.BeefyClientWrapperTransactor.contract.Transact(opts, method, params...)
}

// Call invokes the (constant) contract method with params as input values and
// sets the output to result. The result type might be a single field for simple
// returns, a slice of interfaces for anonymous returns and a struct for named
// returns.
func (_BeefyClientWrapper *BeefyClientWrapperCallerRaw) Call(opts *bind.CallOpts, result *[]interface{}, method string, params ...interface{}) error {
	return _BeefyClientWrapper.Contract.contract.Call(opts, result, method, params...)
}

// Transfer initiates a plain transaction to move funds to the contract, calling
// its default method if one is available.
func (_BeefyClientWrapper *BeefyClientWrapperTransactorRaw) Transfer(opts *bind.TransactOpts) (*types.Transaction, error) {
	return _BeefyClientWrapper.Contract.contract.Transfer(opts)
}

// Transact invokes the (paid) contract method with params as input values.
func (_BeefyClientWrapper *BeefyClientWrapperTransactorRaw) Transact(opts *bind.TransactOpts, method string, params ...interface{}) (*types.Transaction, error) {
	return _BeefyClientWrapper.Contract.contract.Transact(opts, method, params...)
}

// Gateway is a free data retrieval call binding the contract method 0x116191b6.
//
// Solidity: function gateway() view returns(address)
func (_BeefyClientWrapper *BeefyClientWrapperCaller) Gateway(opts *bind.CallOpts) (common.Address, error) {
	var out []interface{}
	err := _BeefyClientWrapper.contract.Call(opts, &out, "gateway")

	if err != nil {
		return *new(common.Address), err
	}

	out0 := *abi.ConvertType(out[0], new(common.Address)).(*common.Address)

	return out0, err

}

// Gateway is a free data retrieval call binding the contract method 0x116191b6.
//
// Solidity: function gateway() view returns(address)
func (_BeefyClientWrapper *BeefyClientWrapperSession) Gateway() (common.Address, error) {
	return _BeefyClientWrapper.Contract.Gateway(&_BeefyClientWrapper.CallOpts)
}

// Gateway is a free data retrieval call binding the contract method 0x116191b6.
//
// Solidity: function gateway() view returns(address)
func (_BeefyClientWrapper *BeefyClientWrapperCallerSession) Gateway() (common.Address, error) {
	return _BeefyClientWrapper.Contract.Gateway(&_BeefyClientWrapper.CallOpts)
}

// HighestPendingBlock is a free data retrieval call binding the contract method 0x0dc2be13.
//
// Solidity: function highestPendingBlock() view returns(uint256)
func (_BeefyClientWrapper *BeefyClientWrapperCaller) HighestPendingBlock(opts *bind.CallOpts) (*big.Int, error) {
	var out []interface{}
	err := _BeefyClientWrapper.contract.Call(opts, &out, "highestPendingBlock")

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// HighestPendingBlock is a free data retrieval call binding the contract method 0x0dc2be13.
//
// Solidity: function highestPendingBlock() view returns(uint256)
func (_BeefyClientWrapper *BeefyClientWrapperSession) HighestPendingBlock() (*big.Int, error) {
	return _BeefyClientWrapper.Contract.HighestPendingBlock(&_BeefyClientWrapper.CallOpts)
}

// HighestPendingBlock is a free data retrieval call binding the contract method 0x0dc2be13.
//
// Solidity: function highestPendingBlock() view returns(uint256)
func (_BeefyClientWrapper *BeefyClientWrapperCallerSession) HighestPendingBlock() (*big.Int, error) {
	return _BeefyClientWrapper.Contract.HighestPendingBlock(&_BeefyClientWrapper.CallOpts)
}

// HighestPendingBlockTimestamp is a free data retrieval call binding the contract method 0x33e2c682.
//
// Solidity: function highestPendingBlockTimestamp() view returns(uint256)
func (_BeefyClientWrapper *BeefyClientWrapperCaller) HighestPendingBlockTimestamp(opts *bind.CallOpts) (*big.Int, error) {
	var out []interface{}
	err := _BeefyClientWrapper.contract.Call(opts, &out, "highestPendingBlockTimestamp")

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// HighestPendingBlockTimestamp is a free data retrieval call binding the contract method 0x33e2c682.
//
// Solidity: function highestPendingBlockTimestamp() view returns(uint256)
func (_BeefyClientWrapper *BeefyClientWrapperSession) HighestPendingBlockTimestamp() (*big.Int, error) {
	return _BeefyClientWrapper.Contract.HighestPendingBlockTimestamp(&_BeefyClientWrapper.CallOpts)
}

// HighestPendingBlockTimestamp is a free data retrieval call binding the contract method 0x33e2c682.
//
// Solidity: function highestPendingBlockTimestamp() view returns(uint256)
func (_BeefyClientWrapper *BeefyClientWrapperCallerSession) HighestPendingBlockTimestamp() (*big.Int, error) {
	return _BeefyClientWrapper.Contract.HighestPendingBlockTimestamp(&_BeefyClientWrapper.CallOpts)
}

// MaxGasPrice is a free data retrieval call binding the contract method 0x3de39c11.
//
// Solidity: function maxGasPrice() view returns(uint256)
func (_BeefyClientWrapper *BeefyClientWrapperCaller) MaxGasPrice(opts *bind.CallOpts) (*big.Int, error) {
	var out []interface{}
	err := _BeefyClientWrapper.contract.Call(opts, &out, "maxGasPrice")

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// MaxGasPrice is a free data retrieval call binding the contract method 0x3de39c11.
//
// Solidity: function maxGasPrice() view returns(uint256)
func (_BeefyClientWrapper *BeefyClientWrapperSession) MaxGasPrice() (*big.Int, error) {
	return _BeefyClientWrapper.Contract.MaxGasPrice(&_BeefyClientWrapper.CallOpts)
}

// MaxGasPrice is a free data retrieval call binding the contract method 0x3de39c11.
//
// Solidity: function maxGasPrice() view returns(uint256)
func (_BeefyClientWrapper *BeefyClientWrapperCallerSession) MaxGasPrice() (*big.Int, error) {
	return _BeefyClientWrapper.Contract.MaxGasPrice(&_BeefyClientWrapper.CallOpts)
}

// MaxRefundAmount is a free data retrieval call binding the contract method 0x28bbc5c1.
//
// Solidity: function maxRefundAmount() view returns(uint256)
func (_BeefyClientWrapper *BeefyClientWrapperCaller) MaxRefundAmount(opts *bind.CallOpts) (*big.Int, error) {
	var out []interface{}
	err := _BeefyClientWrapper.contract.Call(opts, &out, "maxRefundAmount")

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// MaxRefundAmount is a free data retrieval call binding the contract method 0x28bbc5c1.
//
// Solidity: function maxRefundAmount() view returns(uint256)
func (_BeefyClientWrapper *BeefyClientWrapperSession) MaxRefundAmount() (*big.Int, error) {
	return _BeefyClientWrapper.Contract.MaxRefundAmount(&_BeefyClientWrapper.CallOpts)
}

// MaxRefundAmount is a free data retrieval call binding the contract method 0x28bbc5c1.
//
// Solidity: function maxRefundAmount() view returns(uint256)
func (_BeefyClientWrapper *BeefyClientWrapperCallerSession) MaxRefundAmount() (*big.Int, error) {
	return _BeefyClientWrapper.Contract.MaxRefundAmount(&_BeefyClientWrapper.CallOpts)
}

// PendingTickets is a free data retrieval call binding the contract method 0x6ff8c34f.
//
// Solidity: function pendingTickets(bytes32 ) view returns(address owner, uint256 creditedCost, uint64 createdAt)
func (_BeefyClientWrapper *BeefyClientWrapperCaller) PendingTickets(opts *bind.CallOpts, arg0 [32]byte) (struct {
	Owner        common.Address
	CreditedCost *big.Int
	CreatedAt    uint64
}, error) {
	var out []interface{}
	err := _BeefyClientWrapper.contract.Call(opts, &out, "pendingTickets", arg0)

	outstruct := new(struct {
		Owner        common.Address
		CreditedCost *big.Int
		CreatedAt    uint64
	})
	if err != nil {
		return *outstruct, err
	}

	outstruct.Owner = *abi.ConvertType(out[0], new(common.Address)).(*common.Address)
	outstruct.CreditedCost = *abi.ConvertType(out[1], new(*big.Int)).(**big.Int)
	outstruct.CreatedAt = *abi.ConvertType(out[2], new(uint64)).(*uint64)

	return *outstruct, err

}

// PendingTickets is a free data retrieval call binding the contract method 0x6ff8c34f.
//
// Solidity: function pendingTickets(bytes32 ) view returns(address owner, uint256 creditedCost, uint64 createdAt)
func (_BeefyClientWrapper *BeefyClientWrapperSession) PendingTickets(arg0 [32]byte) (struct {
	Owner        common.Address
	CreditedCost *big.Int
	CreatedAt    uint64
}, error) {
	return _BeefyClientWrapper.Contract.PendingTickets(&_BeefyClientWrapper.CallOpts, arg0)
}

// PendingTickets is a free data retrieval call binding the contract method 0x6ff8c34f.
//
// Solidity: function pendingTickets(bytes32 ) view returns(address owner, uint256 creditedCost, uint64 createdAt)
func (_BeefyClientWrapper *BeefyClientWrapperCallerSession) PendingTickets(arg0 [32]byte) (struct {
	Owner        common.Address
	CreditedCost *big.Int
	CreatedAt    uint64
}, error) {
	return _BeefyClientWrapper.Contract.PendingTickets(&_BeefyClientWrapper.CallOpts, arg0)
}

// RefundTarget is a free data retrieval call binding the contract method 0xd679e02a.
//
// Solidity: function refundTarget() view returns(uint256)
func (_BeefyClientWrapper *BeefyClientWrapperCaller) RefundTarget(opts *bind.CallOpts) (*big.Int, error) {
	var out []interface{}
	err := _BeefyClientWrapper.contract.Call(opts, &out, "refundTarget")

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// RefundTarget is a free data retrieval call binding the contract method 0xd679e02a.
//
// Solidity: function refundTarget() view returns(uint256)
func (_BeefyClientWrapper *BeefyClientWrapperSession) RefundTarget() (*big.Int, error) {
	return _BeefyClientWrapper.Contract.RefundTarget(&_BeefyClientWrapper.CallOpts)
}

// RefundTarget is a free data retrieval call binding the contract method 0xd679e02a.
//
// Solidity: function refundTarget() view returns(uint256)
func (_BeefyClientWrapper *BeefyClientWrapperCallerSession) RefundTarget() (*big.Int, error) {
	return _BeefyClientWrapper.Contract.RefundTarget(&_BeefyClientWrapper.CallOpts)
}

// TicketTimeout is a free data retrieval call binding the contract method 0x091207ac.
//
// Solidity: function ticketTimeout() view returns(uint256)
func (_BeefyClientWrapper *BeefyClientWrapperCaller) TicketTimeout(opts *bind.CallOpts) (*big.Int, error) {
	var out []interface{}
	err := _BeefyClientWrapper.contract.Call(opts, &out, "ticketTimeout")

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// TicketTimeout is a free data retrieval call binding the contract method 0x091207ac.
//
// Solidity: function ticketTimeout() view returns(uint256)
func (_BeefyClientWrapper *BeefyClientWrapperSession) TicketTimeout() (*big.Int, error) {
	return _BeefyClientWrapper.Contract.TicketTimeout(&_BeefyClientWrapper.CallOpts)
}

// TicketTimeout is a free data retrieval call binding the contract method 0x091207ac.
//
// Solidity: function ticketTimeout() view returns(uint256)
func (_BeefyClientWrapper *BeefyClientWrapperCallerSession) TicketTimeout() (*big.Int, error) {
	return _BeefyClientWrapper.Contract.TicketTimeout(&_BeefyClientWrapper.CallOpts)
}

// CommitPrevRandao is a paid mutator transaction binding the contract method 0xa77cf3d2.
//
// Solidity: function commitPrevRandao(bytes32 commitmentHash) returns()
func (_BeefyClientWrapper *BeefyClientWrapperTransactor) CommitPrevRandao(opts *bind.TransactOpts, commitmentHash [32]byte) (*types.Transaction, error) {
	return _BeefyClientWrapper.contract.Transact(opts, "commitPrevRandao", commitmentHash)
}

// CommitPrevRandao is a paid mutator transaction binding the contract method 0xa77cf3d2.
//
// Solidity: function commitPrevRandao(bytes32 commitmentHash) returns()
func (_BeefyClientWrapper *BeefyClientWrapperSession) CommitPrevRandao(commitmentHash [32]byte) (*types.Transaction, error) {
	return _BeefyClientWrapper.Contract.CommitPrevRandao(&_BeefyClientWrapper.TransactOpts, commitmentHash)
}

// CommitPrevRandao is a paid mutator transaction binding the contract method 0xa77cf3d2.
//
// Solidity: function commitPrevRandao(bytes32 commitmentHash) returns()
func (_BeefyClientWrapper *BeefyClientWrapperTransactorSession) CommitPrevRandao(commitmentHash [32]byte) (*types.Transaction, error) {
	return _BeefyClientWrapper.Contract.CommitPrevRandao(&_BeefyClientWrapper.TransactOpts, commitmentHash)
}

// Receive is a paid mutator transaction binding the contract receive function.
//
// Solidity: receive() payable returns()
func (_BeefyClientWrapper *BeefyClientWrapperTransactor) Receive(opts *bind.TransactOpts) (*types.Transaction, error) {
	return _BeefyClientWrapper.contract.RawTransact(opts, nil) // calldata is disallowed for receive function
}

// Receive is a paid mutator transaction binding the contract receive function.
//
// Solidity: receive() payable returns()
func (_BeefyClientWrapper *BeefyClientWrapperSession) Receive() (*types.Transaction, error) {
	return _BeefyClientWrapper.Contract.Receive(&_BeefyClientWrapper.TransactOpts)
}

// Receive is a paid mutator transaction binding the contract receive function.
//
// Solidity: receive() payable returns()
func (_BeefyClientWrapper *BeefyClientWrapperTransactorSession) Receive() (*types.Transaction, error) {
	return _BeefyClientWrapper.Contract.Receive(&_BeefyClientWrapper.TransactOpts)
}

// BeefyClientWrapperCostCreditedIterator is returned from FilterCostCredited and is used to iterate over the raw logs and unpacked data for CostCredited events raised by the BeefyClientWrapper contract.
type BeefyClientWrapperCostCreditedIterator struct {
	Event *BeefyClientWrapperCostCredited // Event containing the contract specifics and raw log

	contract *bind.BoundContract // Generic contract to use for unpacking event data
	event    string              // Event name to use for unpacking event data

	logs chan types.Log        // Log channel receiving the found contract events
	sub  ethereum.Subscription // Subscription for errors, completion and termination
	done bool                  // Whether the subscription completed delivering logs
	fail error                 // Occurred error to stop iteration
}

// Next advances the iterator to the subsequent event, returning whether there
// are any more events found. In case of a retrieval or parsing error, false is
// returned and Error() can be queried for the exact failure.
func (it *BeefyClientWrapperCostCreditedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(BeefyClientWrapperCostCredited)
			if err := it.contract.UnpackLog(it.Event, it.event, log); err != nil {
				it.fail = err
				return false
			}
			it.Event.Raw = log
			return true

		default:
			return false
		}
	}
	// Iterator still in progress, wait for either a data or an error event
	select {
	case log := <-it.logs:
		it.Event = new(BeefyClientWrapperCostCredited)
		if err := it.contract.UnpackLog(it.Event, it.event, log); err != nil {
			it.fail = err
			return false
		}
		it.Event.Raw = log
		return true

	case err := <-it.sub.Err():
		it.done = true
		it.fail = err
		return it.Next()
	}
}

// Error returns any retrieval or parsing error occurred during filtering.
func (it *BeefyClientWrapperCostCreditedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *BeefyClientWrapperCostCreditedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// BeefyClientWrapperCostCredited represents a CostCredited event raised by the BeefyClientWrapper contract.
type BeefyClientWrapperCostCredited struct {
	Relayer        common.Address
	CommitmentHash [32]byte
	Cost           *big.Int
	Raw            types.Log // Blockchain specific contextual infos
}

// FilterCostCredited is a free log retrieval operation binding the contract event 0x793a43b692b5c13204c15d826f125eb739e34b8a6d486f4f94de6b11a7f15cea.
//
// Solidity: event CostCredited(address indexed relayer, bytes32 indexed commitmentHash, uint256 cost)
func (_BeefyClientWrapper *BeefyClientWrapperFilterer) FilterCostCredited(opts *bind.FilterOpts, relayer []common.Address, commitmentHash [][32]byte) (*BeefyClientWrapperCostCreditedIterator, error) {

	var relayerRule []interface{}
	for _, relayerItem := range relayer {
		relayerRule = append(relayerRule, relayerItem)
	}
	var commitmentHashRule []interface{}
	for _, commitmentHashItem := range commitmentHash {
		commitmentHashRule = append(commitmentHashRule, commitmentHashItem)
	}

	logs, sub, err := _BeefyClientWrapper.contract.FilterLogs(opts, "CostCredited", relayerRule, commitmentHashRule)
	if err != nil {
		return nil, err
	}
	return &BeefyClientWrapperCostCreditedIterator{contract: _BeefyClientWrapper.contract, event: "CostCredited", logs: logs, sub: sub}, nil
}

// WatchCostCredited is a free log subscription operation binding the contract event 0x793a43b692b5c13204c15d826f125eb739e34b8a6d486f4f94de6b11a7f15cea.
//
// Solidity: event CostCredited(address indexed relayer, bytes32 indexed commitmentHash, uint256 cost)
func (_BeefyClientWrapper *BeefyClientWrapperFilterer) WatchCostCredited(opts *bind.WatchOpts, sink chan<- *BeefyClientWrapperCostCredited, relayer []common.Address, commitmentHash [][32]byte) (event.Subscription, error) {

	var relayerRule []interface{}
	for _, relayerItem := range relayer {
		relayerRule = append(relayerRule, relayerItem)
	}
	var commitmentHashRule []interface{}
	for _, commitmentHashItem := range commitmentHash {
		commitmentHashRule = append(commitmentHashRule, commitmentHashItem)
	}

	logs, sub, err := _BeefyClientWrapper.contract.WatchLogs(opts, "CostCredited", relayerRule, commitmentHashRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(BeefyClientWrapperCostCredited)
				if err := _BeefyClientWrapper.contract.UnpackLog(event, "CostCredited", log); err != nil {
					return err
				}
				event.Raw = log

				select {
				case sink <- event:
				case err := <-sub.Err():
					return err
				case <-quit:
					return nil
				}
			case err := <-sub.Err():
				return err
			case <-quit:
				return nil
			}
		}
	}), nil
}

// ParseCostCredited is a log parse operation binding the contract event 0x793a43b692b5c13204c15d826f125eb739e34b8a6d486f4f94de6b11a7f15cea.
//
// Solidity: event CostCredited(address indexed relayer, bytes32 indexed commitmentHash, uint256 cost)
func (_BeefyClientWrapper *BeefyClientWrapperFilterer) ParseCostCredited(log types.Log) (*BeefyClientWrapperCostCredited, error) {
	event := new(BeefyClientWrapperCostCredited)
	if err := _BeefyClientWrapper.contract.UnpackLog(event, "CostCredited", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// BeefyClientWrapperSubmissionRefundedIterator is returned from FilterSubmissionRefunded and is used to iterate over the raw logs and unpacked data for SubmissionRefunded events raised by the BeefyClientWrapper contract.
type BeefyClientWrapperSubmissionRefundedIterator struct {
	Event *BeefyClientWrapperSubmissionRefunded // Event containing the contract specifics and raw log

	contract *bind.BoundContract // Generic contract to use for unpacking event data
	event    string              // Event name to use for unpacking event data

	logs chan types.Log        // Log channel receiving the found contract events
	sub  ethereum.Subscription // Subscription for errors, completion and termination
	done bool                  // Whether the subscription completed delivering logs
	fail error                 // Occurred error to stop iteration
}

// Next advances the iterator to the subsequent event, returning whether there
// are any more events found. In case of a retrieval or parsing error, false is
// returned and Error() can be queried for the exact failure.
func (it *BeefyClientWrapperSubmissionRefundedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(BeefyClientWrapperSubmissionRefunded)
			if err := it.contract.UnpackLog(it.Event, it.event, log); err != nil {
				it.fail = err
				return false
			}
			it.Event.Raw = log
			return true

		default:
			return false
		}
	}
	// Iterator still in progress, wait for either a data or an error event
	select {
	case log := <-it.logs:
		it.Event = new(BeefyClientWrapperSubmissionRefunded)
		if err := it.contract.UnpackLog(it.Event, it.event, log); err != nil {
			it.fail = err
			return false
		}
		it.Event.Raw = log
		return true

	case err := <-it.sub.Err():
		it.done = true
		it.fail = err
		return it.Next()
	}
}

// Error returns any retrieval or parsing error occurred during filtering.
func (it *BeefyClientWrapperSubmissionRefundedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *BeefyClientWrapperSubmissionRefundedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// BeefyClientWrapperSubmissionRefunded represents a SubmissionRefunded event raised by the BeefyClientWrapper contract.
type BeefyClientWrapperSubmissionRefunded struct {
	Relayer      common.Address
	Progress     *big.Int
	RefundAmount *big.Int
	Raw          types.Log // Blockchain specific contextual infos
}

// FilterSubmissionRefunded is a free log retrieval operation binding the contract event 0x103cb711554967fbeea0c6394a6dfedc44d0e729191e719d0976438783912f52.
//
// Solidity: event SubmissionRefunded(address indexed relayer, uint256 progress, uint256 refundAmount)
func (_BeefyClientWrapper *BeefyClientWrapperFilterer) FilterSubmissionRefunded(opts *bind.FilterOpts, relayer []common.Address) (*BeefyClientWrapperSubmissionRefundedIterator, error) {

	var relayerRule []interface{}
	for _, relayerItem := range relayer {
		relayerRule = append(relayerRule, relayerItem)
	}

	logs, sub, err := _BeefyClientWrapper.contract.FilterLogs(opts, "SubmissionRefunded", relayerRule)
	if err != nil {
		return nil, err
	}
	return &BeefyClientWrapperSubmissionRefundedIterator{contract: _BeefyClientWrapper.contract, event: "SubmissionRefunded", logs: logs, sub: sub}, nil
}

// WatchSubmissionRefunded is a free log subscription operation binding the contract event 0x103cb711554967fbeea0c6394a6dfedc44d0e729191e719d0976438783912f52.
//
// Solidity: event SubmissionRefunded(address indexed relayer, uint256 progress, uint256 refundAmount)
func (_BeefyClientWrapper *BeefyClientWrapperFilterer) WatchSubmissionRefunded(opts *bind.WatchOpts, sink chan<- *BeefyClientWrapperSubmissionRefunded, relayer []common.Address) (event.Subscription, error) {

	var relayerRule []interface{}
	for _, relayerItem := range relayer {
		relayerRule = append(relayerRule, relayerItem)
	}

	logs, sub, err := _BeefyClientWrapper.contract.WatchLogs(opts, "SubmissionRefunded", relayerRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(BeefyClientWrapperSubmissionRefunded)
				if err := _BeefyClientWrapper.contract.UnpackLog(event, "SubmissionRefunded", log); err != nil {
					return err
				}
				event.Raw = log

				select {
				case sink <- event:
				case err := <-sub.Err():
					return err
				case <-quit:
					return nil
				}
			case err := <-sub.Err():
				return err
			case <-quit:
				return nil
			}
		}
	}), nil
}

// ParseSubmissionRefunded is a log parse operation binding the contract event 0x103cb711554967fbeea0c6394a6dfedc44d0e729191e719d0976438783912f52.
//
// Solidity: event SubmissionRefunded(address indexed relayer, uint256 progress, uint256 refundAmount)
func (_BeefyClientWrapper *BeefyClientWrapperFilterer) ParseSubmissionRefunded(log types.Log) (*BeefyClientWrapperSubmissionRefunded, error) {
	event := new(BeefyClientWrapperSubmissionRefunded)
	if err := _BeefyClientWrapper.contract.UnpackLog(event, "SubmissionRefunded", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}
