// Code generated - DO NOT EDIT.
// This file is a generated binding and any manual changes will be lost.

package basic

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
)

// BasicInboundChannelMessage is an auto generated low-level Go binding around an user-defined struct.
type BasicInboundChannelMessage struct {
	SourceId [32]byte
	Nonce    uint64
	Payload  []byte
}

// BasicInboundChannelMetaData contains all meta data concerning the BasicInboundChannel contract.
var BasicInboundChannelMetaData = &bind.MetaData{
	ABI: "[{\"inputs\":[{\"internalType\":\"contractParachainClient\",\"name\":\"_parachainClient\",\"type\":\"address\"}],\"stateMutability\":\"nonpayable\",\"type\":\"constructor\"},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\"}],\"name\":\"nonce\",\"outputs\":[{\"internalType\":\"uint64\",\"name\":\"\",\"type\":\"uint64\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"parachainClient\",\"outputs\":[{\"internalType\":\"contractParachainClient\",\"name\":\"\",\"type\":\"address\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"components\":[{\"internalType\":\"bytes32\",\"name\":\"sourceId\",\"type\":\"bytes32\"},{\"internalType\":\"uint64\",\"name\":\"nonce\",\"type\":\"uint64\"},{\"internalType\":\"bytes\",\"name\":\"payload\",\"type\":\"bytes\"}],\"internalType\":\"structBasicInboundChannel.Message\",\"name\":\"message\",\"type\":\"tuple\"},{\"internalType\":\"bytes32[]\",\"name\":\"leafProof\",\"type\":\"bytes32[]\"},{\"internalType\":\"bool[]\",\"name\":\"hashSides\",\"type\":\"bool[]\"},{\"internalType\":\"bytes\",\"name\":\"parachainHeaderProof\",\"type\":\"bytes\"}],\"name\":\"submit\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"}]",
}

// BasicInboundChannelABI is the input ABI used to generate the binding from.
// Deprecated: Use BasicInboundChannelMetaData.ABI instead.
var BasicInboundChannelABI = BasicInboundChannelMetaData.ABI

// BasicInboundChannel is an auto generated Go binding around an Ethereum contract.
type BasicInboundChannel struct {
	BasicInboundChannelCaller     // Read-only binding to the contract
	BasicInboundChannelTransactor // Write-only binding to the contract
	BasicInboundChannelFilterer   // Log filterer for contract events
}

// BasicInboundChannelCaller is an auto generated read-only Go binding around an Ethereum contract.
type BasicInboundChannelCaller struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// BasicInboundChannelTransactor is an auto generated write-only Go binding around an Ethereum contract.
type BasicInboundChannelTransactor struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// BasicInboundChannelFilterer is an auto generated log filtering Go binding around an Ethereum contract events.
type BasicInboundChannelFilterer struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// BasicInboundChannelSession is an auto generated Go binding around an Ethereum contract,
// with pre-set call and transact options.
type BasicInboundChannelSession struct {
	Contract     *BasicInboundChannel // Generic contract binding to set the session for
	CallOpts     bind.CallOpts        // Call options to use throughout this session
	TransactOpts bind.TransactOpts    // Transaction auth options to use throughout this session
}

// BasicInboundChannelCallerSession is an auto generated read-only Go binding around an Ethereum contract,
// with pre-set call options.
type BasicInboundChannelCallerSession struct {
	Contract *BasicInboundChannelCaller // Generic contract caller binding to set the session for
	CallOpts bind.CallOpts              // Call options to use throughout this session
}

// BasicInboundChannelTransactorSession is an auto generated write-only Go binding around an Ethereum contract,
// with pre-set transact options.
type BasicInboundChannelTransactorSession struct {
	Contract     *BasicInboundChannelTransactor // Generic contract transactor binding to set the session for
	TransactOpts bind.TransactOpts              // Transaction auth options to use throughout this session
}

// BasicInboundChannelRaw is an auto generated low-level Go binding around an Ethereum contract.
type BasicInboundChannelRaw struct {
	Contract *BasicInboundChannel // Generic contract binding to access the raw methods on
}

// BasicInboundChannelCallerRaw is an auto generated low-level read-only Go binding around an Ethereum contract.
type BasicInboundChannelCallerRaw struct {
	Contract *BasicInboundChannelCaller // Generic read-only contract binding to access the raw methods on
}

// BasicInboundChannelTransactorRaw is an auto generated low-level write-only Go binding around an Ethereum contract.
type BasicInboundChannelTransactorRaw struct {
	Contract *BasicInboundChannelTransactor // Generic write-only contract binding to access the raw methods on
}

// NewBasicInboundChannel creates a new instance of BasicInboundChannel, bound to a specific deployed contract.
func NewBasicInboundChannel(address common.Address, backend bind.ContractBackend) (*BasicInboundChannel, error) {
	contract, err := bindBasicInboundChannel(address, backend, backend, backend)
	if err != nil {
		return nil, err
	}
	return &BasicInboundChannel{BasicInboundChannelCaller: BasicInboundChannelCaller{contract: contract}, BasicInboundChannelTransactor: BasicInboundChannelTransactor{contract: contract}, BasicInboundChannelFilterer: BasicInboundChannelFilterer{contract: contract}}, nil
}

// NewBasicInboundChannelCaller creates a new read-only instance of BasicInboundChannel, bound to a specific deployed contract.
func NewBasicInboundChannelCaller(address common.Address, caller bind.ContractCaller) (*BasicInboundChannelCaller, error) {
	contract, err := bindBasicInboundChannel(address, caller, nil, nil)
	if err != nil {
		return nil, err
	}
	return &BasicInboundChannelCaller{contract: contract}, nil
}

// NewBasicInboundChannelTransactor creates a new write-only instance of BasicInboundChannel, bound to a specific deployed contract.
func NewBasicInboundChannelTransactor(address common.Address, transactor bind.ContractTransactor) (*BasicInboundChannelTransactor, error) {
	contract, err := bindBasicInboundChannel(address, nil, transactor, nil)
	if err != nil {
		return nil, err
	}
	return &BasicInboundChannelTransactor{contract: contract}, nil
}

// NewBasicInboundChannelFilterer creates a new log filterer instance of BasicInboundChannel, bound to a specific deployed contract.
func NewBasicInboundChannelFilterer(address common.Address, filterer bind.ContractFilterer) (*BasicInboundChannelFilterer, error) {
	contract, err := bindBasicInboundChannel(address, nil, nil, filterer)
	if err != nil {
		return nil, err
	}
	return &BasicInboundChannelFilterer{contract: contract}, nil
}

// bindBasicInboundChannel binds a generic wrapper to an already deployed contract.
func bindBasicInboundChannel(address common.Address, caller bind.ContractCaller, transactor bind.ContractTransactor, filterer bind.ContractFilterer) (*bind.BoundContract, error) {
	parsed, err := abi.JSON(strings.NewReader(BasicInboundChannelABI))
	if err != nil {
		return nil, err
	}
	return bind.NewBoundContract(address, parsed, caller, transactor, filterer), nil
}

// Call invokes the (constant) contract method with params as input values and
// sets the output to result. The result type might be a single field for simple
// returns, a slice of interfaces for anonymous returns and a struct for named
// returns.
func (_BasicInboundChannel *BasicInboundChannelRaw) Call(opts *bind.CallOpts, result *[]interface{}, method string, params ...interface{}) error {
	return _BasicInboundChannel.Contract.BasicInboundChannelCaller.contract.Call(opts, result, method, params...)
}

// Transfer initiates a plain transaction to move funds to the contract, calling
// its default method if one is available.
func (_BasicInboundChannel *BasicInboundChannelRaw) Transfer(opts *bind.TransactOpts) (*types.Transaction, error) {
	return _BasicInboundChannel.Contract.BasicInboundChannelTransactor.contract.Transfer(opts)
}

// Transact invokes the (paid) contract method with params as input values.
func (_BasicInboundChannel *BasicInboundChannelRaw) Transact(opts *bind.TransactOpts, method string, params ...interface{}) (*types.Transaction, error) {
	return _BasicInboundChannel.Contract.BasicInboundChannelTransactor.contract.Transact(opts, method, params...)
}

// Call invokes the (constant) contract method with params as input values and
// sets the output to result. The result type might be a single field for simple
// returns, a slice of interfaces for anonymous returns and a struct for named
// returns.
func (_BasicInboundChannel *BasicInboundChannelCallerRaw) Call(opts *bind.CallOpts, result *[]interface{}, method string, params ...interface{}) error {
	return _BasicInboundChannel.Contract.contract.Call(opts, result, method, params...)
}

// Transfer initiates a plain transaction to move funds to the contract, calling
// its default method if one is available.
func (_BasicInboundChannel *BasicInboundChannelTransactorRaw) Transfer(opts *bind.TransactOpts) (*types.Transaction, error) {
	return _BasicInboundChannel.Contract.contract.Transfer(opts)
}

// Transact invokes the (paid) contract method with params as input values.
func (_BasicInboundChannel *BasicInboundChannelTransactorRaw) Transact(opts *bind.TransactOpts, method string, params ...interface{}) (*types.Transaction, error) {
	return _BasicInboundChannel.Contract.contract.Transact(opts, method, params...)
}

// Nonce is a free data retrieval call binding the contract method 0x905da30f.
//
// Solidity: function nonce(bytes32 ) view returns(uint64)
func (_BasicInboundChannel *BasicInboundChannelCaller) Nonce(opts *bind.CallOpts, arg0 [32]byte) (uint64, error) {
	var out []interface{}
	err := _BasicInboundChannel.contract.Call(opts, &out, "nonce", arg0)

	if err != nil {
		return *new(uint64), err
	}

	out0 := *abi.ConvertType(out[0], new(uint64)).(*uint64)

	return out0, err

}

// Nonce is a free data retrieval call binding the contract method 0x905da30f.
//
// Solidity: function nonce(bytes32 ) view returns(uint64)
func (_BasicInboundChannel *BasicInboundChannelSession) Nonce(arg0 [32]byte) (uint64, error) {
	return _BasicInboundChannel.Contract.Nonce(&_BasicInboundChannel.CallOpts, arg0)
}

// Nonce is a free data retrieval call binding the contract method 0x905da30f.
//
// Solidity: function nonce(bytes32 ) view returns(uint64)
func (_BasicInboundChannel *BasicInboundChannelCallerSession) Nonce(arg0 [32]byte) (uint64, error) {
	return _BasicInboundChannel.Contract.Nonce(&_BasicInboundChannel.CallOpts, arg0)
}

// ParachainClient is a free data retrieval call binding the contract method 0x1674f9b7.
//
// Solidity: function parachainClient() view returns(address)
func (_BasicInboundChannel *BasicInboundChannelCaller) ParachainClient(opts *bind.CallOpts) (common.Address, error) {
	var out []interface{}
	err := _BasicInboundChannel.contract.Call(opts, &out, "parachainClient")

	if err != nil {
		return *new(common.Address), err
	}

	out0 := *abi.ConvertType(out[0], new(common.Address)).(*common.Address)

	return out0, err

}

// ParachainClient is a free data retrieval call binding the contract method 0x1674f9b7.
//
// Solidity: function parachainClient() view returns(address)
func (_BasicInboundChannel *BasicInboundChannelSession) ParachainClient() (common.Address, error) {
	return _BasicInboundChannel.Contract.ParachainClient(&_BasicInboundChannel.CallOpts)
}

// ParachainClient is a free data retrieval call binding the contract method 0x1674f9b7.
//
// Solidity: function parachainClient() view returns(address)
func (_BasicInboundChannel *BasicInboundChannelCallerSession) ParachainClient() (common.Address, error) {
	return _BasicInboundChannel.Contract.ParachainClient(&_BasicInboundChannel.CallOpts)
}

// Submit is a paid mutator transaction binding the contract method 0xb690a07e.
//
// Solidity: function submit((bytes32,uint64,bytes) message, bytes32[] leafProof, bool[] hashSides, bytes parachainHeaderProof) returns()
func (_BasicInboundChannel *BasicInboundChannelTransactor) Submit(opts *bind.TransactOpts, message BasicInboundChannelMessage, leafProof [][32]byte, hashSides []bool, parachainHeaderProof []byte) (*types.Transaction, error) {
	return _BasicInboundChannel.contract.Transact(opts, "submit", message, leafProof, hashSides, parachainHeaderProof)
}

// Submit is a paid mutator transaction binding the contract method 0xb690a07e.
//
// Solidity: function submit((bytes32,uint64,bytes) message, bytes32[] leafProof, bool[] hashSides, bytes parachainHeaderProof) returns()
func (_BasicInboundChannel *BasicInboundChannelSession) Submit(message BasicInboundChannelMessage, leafProof [][32]byte, hashSides []bool, parachainHeaderProof []byte) (*types.Transaction, error) {
	return _BasicInboundChannel.Contract.Submit(&_BasicInboundChannel.TransactOpts, message, leafProof, hashSides, parachainHeaderProof)
}

// Submit is a paid mutator transaction binding the contract method 0xb690a07e.
//
// Solidity: function submit((bytes32,uint64,bytes) message, bytes32[] leafProof, bool[] hashSides, bytes parachainHeaderProof) returns()
func (_BasicInboundChannel *BasicInboundChannelTransactorSession) Submit(message BasicInboundChannelMessage, leafProof [][32]byte, hashSides []bool, parachainHeaderProof []byte) (*types.Transaction, error) {
	return _BasicInboundChannel.Contract.Submit(&_BasicInboundChannel.TransactOpts, message, leafProof, hashSides, parachainHeaderProof)
}
