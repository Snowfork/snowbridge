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

// BasicInboundChannelV2Message is an auto generated low-level Go binding around an user-defined struct.
type BasicInboundChannelV2Message struct {
	Id      uint64
	Target  common.Address
	Payload []byte
}

// BasicInboundChannelV2MessageBundle is an auto generated low-level Go binding around an user-defined struct.
type BasicInboundChannelV2MessageBundle struct {
	SourceChannelID uint8
	Account         [32]byte
	Nonce           uint64
	Messages        []BasicInboundChannelV2Message
}

// BasicInboundChannelV2MetaData contains all meta data concerning the BasicInboundChannelV2 contract.
var BasicInboundChannelV2MetaData = &bind.MetaData{
	ABI: "[{\"inputs\":[{\"internalType\":\"uint8\",\"name\":\"_sourceChannelID\",\"type\":\"uint8\"},{\"internalType\":\"contractParachainClient\",\"name\":\"_parachainClient\",\"type\":\"address\"}],\"stateMutability\":\"nonpayable\",\"type\":\"constructor\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":false,\"internalType\":\"uint64\",\"name\":\"id\",\"type\":\"uint64\"},{\"indexed\":false,\"internalType\":\"bool\",\"name\":\"result\",\"type\":\"bool\"}],\"name\":\"MessageDispatched\",\"type\":\"event\"},{\"inputs\":[],\"name\":\"GAS_BUFFER\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"MAX_GAS_PER_MESSAGE\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\"}],\"name\":\"nonces\",\"outputs\":[{\"internalType\":\"uint64\",\"name\":\"\",\"type\":\"uint64\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"parachainClient\",\"outputs\":[{\"internalType\":\"contractParachainClient\",\"name\":\"\",\"type\":\"address\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"sourceChannelID\",\"outputs\":[{\"internalType\":\"uint8\",\"name\":\"\",\"type\":\"uint8\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"components\":[{\"internalType\":\"uint8\",\"name\":\"sourceChannelID\",\"type\":\"uint8\"},{\"internalType\":\"bytes32\",\"name\":\"account\",\"type\":\"bytes32\"},{\"internalType\":\"uint64\",\"name\":\"nonce\",\"type\":\"uint64\"},{\"components\":[{\"internalType\":\"uint64\",\"name\":\"id\",\"type\":\"uint64\"},{\"internalType\":\"address\",\"name\":\"target\",\"type\":\"address\"},{\"internalType\":\"bytes\",\"name\":\"payload\",\"type\":\"bytes\"}],\"internalType\":\"structBasicInboundChannelV2.Message[]\",\"name\":\"messages\",\"type\":\"tuple[]\"}],\"internalType\":\"structBasicInboundChannelV2.MessageBundle\",\"name\":\"bundle\",\"type\":\"tuple\"},{\"internalType\":\"bytes32[]\",\"name\":\"leafProof\",\"type\":\"bytes32[]\"},{\"internalType\":\"bool[]\",\"name\":\"hashSides\",\"type\":\"bool[]\"},{\"internalType\":\"bytes\",\"name\":\"proof\",\"type\":\"bytes\"}],\"name\":\"submit\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"}]",
}

// BasicInboundChannelV2ABI is the input ABI used to generate the binding from.
// Deprecated: Use BasicInboundChannelV2MetaData.ABI instead.
var BasicInboundChannelV2ABI = BasicInboundChannelV2MetaData.ABI

// BasicInboundChannelV2 is an auto generated Go binding around an Ethereum contract.
type BasicInboundChannelV2 struct {
	BasicInboundChannelV2Caller     // Read-only binding to the contract
	BasicInboundChannelV2Transactor // Write-only binding to the contract
	BasicInboundChannelV2Filterer   // Log filterer for contract events
}

// BasicInboundChannelV2Caller is an auto generated read-only Go binding around an Ethereum contract.
type BasicInboundChannelV2Caller struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// BasicInboundChannelV2Transactor is an auto generated write-only Go binding around an Ethereum contract.
type BasicInboundChannelV2Transactor struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// BasicInboundChannelV2Filterer is an auto generated log filtering Go binding around an Ethereum contract events.
type BasicInboundChannelV2Filterer struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// BasicInboundChannelV2Session is an auto generated Go binding around an Ethereum contract,
// with pre-set call and transact options.
type BasicInboundChannelV2Session struct {
	Contract     *BasicInboundChannelV2 // Generic contract binding to set the session for
	CallOpts     bind.CallOpts          // Call options to use throughout this session
	TransactOpts bind.TransactOpts      // Transaction auth options to use throughout this session
}

// BasicInboundChannelV2CallerSession is an auto generated read-only Go binding around an Ethereum contract,
// with pre-set call options.
type BasicInboundChannelV2CallerSession struct {
	Contract *BasicInboundChannelV2Caller // Generic contract caller binding to set the session for
	CallOpts bind.CallOpts                // Call options to use throughout this session
}

// BasicInboundChannelV2TransactorSession is an auto generated write-only Go binding around an Ethereum contract,
// with pre-set transact options.
type BasicInboundChannelV2TransactorSession struct {
	Contract     *BasicInboundChannelV2Transactor // Generic contract transactor binding to set the session for
	TransactOpts bind.TransactOpts                // Transaction auth options to use throughout this session
}

// BasicInboundChannelV2Raw is an auto generated low-level Go binding around an Ethereum contract.
type BasicInboundChannelV2Raw struct {
	Contract *BasicInboundChannelV2 // Generic contract binding to access the raw methods on
}

// BasicInboundChannelV2CallerRaw is an auto generated low-level read-only Go binding around an Ethereum contract.
type BasicInboundChannelV2CallerRaw struct {
	Contract *BasicInboundChannelV2Caller // Generic read-only contract binding to access the raw methods on
}

// BasicInboundChannelV2TransactorRaw is an auto generated low-level write-only Go binding around an Ethereum contract.
type BasicInboundChannelV2TransactorRaw struct {
	Contract *BasicInboundChannelV2Transactor // Generic write-only contract binding to access the raw methods on
}

// NewBasicInboundChannelV2 creates a new instance of BasicInboundChannelV2, bound to a specific deployed contract.
func NewBasicInboundChannelV2(address common.Address, backend bind.ContractBackend) (*BasicInboundChannelV2, error) {
	contract, err := bindBasicInboundChannelV2(address, backend, backend, backend)
	if err != nil {
		return nil, err
	}
	return &BasicInboundChannelV2{BasicInboundChannelV2Caller: BasicInboundChannelV2Caller{contract: contract}, BasicInboundChannelV2Transactor: BasicInboundChannelV2Transactor{contract: contract}, BasicInboundChannelV2Filterer: BasicInboundChannelV2Filterer{contract: contract}}, nil
}

// NewBasicInboundChannelV2Caller creates a new read-only instance of BasicInboundChannelV2, bound to a specific deployed contract.
func NewBasicInboundChannelV2Caller(address common.Address, caller bind.ContractCaller) (*BasicInboundChannelV2Caller, error) {
	contract, err := bindBasicInboundChannelV2(address, caller, nil, nil)
	if err != nil {
		return nil, err
	}
	return &BasicInboundChannelV2Caller{contract: contract}, nil
}

// NewBasicInboundChannelV2Transactor creates a new write-only instance of BasicInboundChannelV2, bound to a specific deployed contract.
func NewBasicInboundChannelV2Transactor(address common.Address, transactor bind.ContractTransactor) (*BasicInboundChannelV2Transactor, error) {
	contract, err := bindBasicInboundChannelV2(address, nil, transactor, nil)
	if err != nil {
		return nil, err
	}
	return &BasicInboundChannelV2Transactor{contract: contract}, nil
}

// NewBasicInboundChannelV2Filterer creates a new log filterer instance of BasicInboundChannelV2, bound to a specific deployed contract.
func NewBasicInboundChannelV2Filterer(address common.Address, filterer bind.ContractFilterer) (*BasicInboundChannelV2Filterer, error) {
	contract, err := bindBasicInboundChannelV2(address, nil, nil, filterer)
	if err != nil {
		return nil, err
	}
	return &BasicInboundChannelV2Filterer{contract: contract}, nil
}

// bindBasicInboundChannelV2 binds a generic wrapper to an already deployed contract.
func bindBasicInboundChannelV2(address common.Address, caller bind.ContractCaller, transactor bind.ContractTransactor, filterer bind.ContractFilterer) (*bind.BoundContract, error) {
	parsed, err := abi.JSON(strings.NewReader(BasicInboundChannelV2ABI))
	if err != nil {
		return nil, err
	}
	return bind.NewBoundContract(address, parsed, caller, transactor, filterer), nil
}

// Call invokes the (constant) contract method with params as input values and
// sets the output to result. The result type might be a single field for simple
// returns, a slice of interfaces for anonymous returns and a struct for named
// returns.
func (_BasicInboundChannelV2 *BasicInboundChannelV2Raw) Call(opts *bind.CallOpts, result *[]interface{}, method string, params ...interface{}) error {
	return _BasicInboundChannelV2.Contract.BasicInboundChannelV2Caller.contract.Call(opts, result, method, params...)
}

// Transfer initiates a plain transaction to move funds to the contract, calling
// its default method if one is available.
func (_BasicInboundChannelV2 *BasicInboundChannelV2Raw) Transfer(opts *bind.TransactOpts) (*types.Transaction, error) {
	return _BasicInboundChannelV2.Contract.BasicInboundChannelV2Transactor.contract.Transfer(opts)
}

// Transact invokes the (paid) contract method with params as input values.
func (_BasicInboundChannelV2 *BasicInboundChannelV2Raw) Transact(opts *bind.TransactOpts, method string, params ...interface{}) (*types.Transaction, error) {
	return _BasicInboundChannelV2.Contract.BasicInboundChannelV2Transactor.contract.Transact(opts, method, params...)
}

// Call invokes the (constant) contract method with params as input values and
// sets the output to result. The result type might be a single field for simple
// returns, a slice of interfaces for anonymous returns and a struct for named
// returns.
func (_BasicInboundChannelV2 *BasicInboundChannelV2CallerRaw) Call(opts *bind.CallOpts, result *[]interface{}, method string, params ...interface{}) error {
	return _BasicInboundChannelV2.Contract.contract.Call(opts, result, method, params...)
}

// Transfer initiates a plain transaction to move funds to the contract, calling
// its default method if one is available.
func (_BasicInboundChannelV2 *BasicInboundChannelV2TransactorRaw) Transfer(opts *bind.TransactOpts) (*types.Transaction, error) {
	return _BasicInboundChannelV2.Contract.contract.Transfer(opts)
}

// Transact invokes the (paid) contract method with params as input values.
func (_BasicInboundChannelV2 *BasicInboundChannelV2TransactorRaw) Transact(opts *bind.TransactOpts, method string, params ...interface{}) (*types.Transaction, error) {
	return _BasicInboundChannelV2.Contract.contract.Transact(opts, method, params...)
}

// GASBUFFER is a free data retrieval call binding the contract method 0xc7e67360.
//
// Solidity: function GAS_BUFFER() view returns(uint256)
func (_BasicInboundChannelV2 *BasicInboundChannelV2Caller) GASBUFFER(opts *bind.CallOpts) (*big.Int, error) {
	var out []interface{}
	err := _BasicInboundChannelV2.contract.Call(opts, &out, "GAS_BUFFER")

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// GASBUFFER is a free data retrieval call binding the contract method 0xc7e67360.
//
// Solidity: function GAS_BUFFER() view returns(uint256)
func (_BasicInboundChannelV2 *BasicInboundChannelV2Session) GASBUFFER() (*big.Int, error) {
	return _BasicInboundChannelV2.Contract.GASBUFFER(&_BasicInboundChannelV2.CallOpts)
}

// GASBUFFER is a free data retrieval call binding the contract method 0xc7e67360.
//
// Solidity: function GAS_BUFFER() view returns(uint256)
func (_BasicInboundChannelV2 *BasicInboundChannelV2CallerSession) GASBUFFER() (*big.Int, error) {
	return _BasicInboundChannelV2.Contract.GASBUFFER(&_BasicInboundChannelV2.CallOpts)
}

// MAXGASPERMESSAGE is a free data retrieval call binding the contract method 0x49bee574.
//
// Solidity: function MAX_GAS_PER_MESSAGE() view returns(uint256)
func (_BasicInboundChannelV2 *BasicInboundChannelV2Caller) MAXGASPERMESSAGE(opts *bind.CallOpts) (*big.Int, error) {
	var out []interface{}
	err := _BasicInboundChannelV2.contract.Call(opts, &out, "MAX_GAS_PER_MESSAGE")

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// MAXGASPERMESSAGE is a free data retrieval call binding the contract method 0x49bee574.
//
// Solidity: function MAX_GAS_PER_MESSAGE() view returns(uint256)
func (_BasicInboundChannelV2 *BasicInboundChannelV2Session) MAXGASPERMESSAGE() (*big.Int, error) {
	return _BasicInboundChannelV2.Contract.MAXGASPERMESSAGE(&_BasicInboundChannelV2.CallOpts)
}

// MAXGASPERMESSAGE is a free data retrieval call binding the contract method 0x49bee574.
//
// Solidity: function MAX_GAS_PER_MESSAGE() view returns(uint256)
func (_BasicInboundChannelV2 *BasicInboundChannelV2CallerSession) MAXGASPERMESSAGE() (*big.Int, error) {
	return _BasicInboundChannelV2.Contract.MAXGASPERMESSAGE(&_BasicInboundChannelV2.CallOpts)
}

// Nonces is a free data retrieval call binding the contract method 0x9e317f12.
//
// Solidity: function nonces(bytes32 ) view returns(uint64)
func (_BasicInboundChannelV2 *BasicInboundChannelV2Caller) Nonces(opts *bind.CallOpts, arg0 [32]byte) (uint64, error) {
	var out []interface{}
	err := _BasicInboundChannelV2.contract.Call(opts, &out, "nonces", arg0)

	if err != nil {
		return *new(uint64), err
	}

	out0 := *abi.ConvertType(out[0], new(uint64)).(*uint64)

	return out0, err

}

// Nonces is a free data retrieval call binding the contract method 0x9e317f12.
//
// Solidity: function nonces(bytes32 ) view returns(uint64)
func (_BasicInboundChannelV2 *BasicInboundChannelV2Session) Nonces(arg0 [32]byte) (uint64, error) {
	return _BasicInboundChannelV2.Contract.Nonces(&_BasicInboundChannelV2.CallOpts, arg0)
}

// Nonces is a free data retrieval call binding the contract method 0x9e317f12.
//
// Solidity: function nonces(bytes32 ) view returns(uint64)
func (_BasicInboundChannelV2 *BasicInboundChannelV2CallerSession) Nonces(arg0 [32]byte) (uint64, error) {
	return _BasicInboundChannelV2.Contract.Nonces(&_BasicInboundChannelV2.CallOpts, arg0)
}

// ParachainClient is a free data retrieval call binding the contract method 0x1674f9b7.
//
// Solidity: function parachainClient() view returns(address)
func (_BasicInboundChannelV2 *BasicInboundChannelV2Caller) ParachainClient(opts *bind.CallOpts) (common.Address, error) {
	var out []interface{}
	err := _BasicInboundChannelV2.contract.Call(opts, &out, "parachainClient")

	if err != nil {
		return *new(common.Address), err
	}

	out0 := *abi.ConvertType(out[0], new(common.Address)).(*common.Address)

	return out0, err

}

// ParachainClient is a free data retrieval call binding the contract method 0x1674f9b7.
//
// Solidity: function parachainClient() view returns(address)
func (_BasicInboundChannelV2 *BasicInboundChannelV2Session) ParachainClient() (common.Address, error) {
	return _BasicInboundChannelV2.Contract.ParachainClient(&_BasicInboundChannelV2.CallOpts)
}

// ParachainClient is a free data retrieval call binding the contract method 0x1674f9b7.
//
// Solidity: function parachainClient() view returns(address)
func (_BasicInboundChannelV2 *BasicInboundChannelV2CallerSession) ParachainClient() (common.Address, error) {
	return _BasicInboundChannelV2.Contract.ParachainClient(&_BasicInboundChannelV2.CallOpts)
}

// SourceChannelID is a free data retrieval call binding the contract method 0x157fb143.
//
// Solidity: function sourceChannelID() view returns(uint8)
func (_BasicInboundChannelV2 *BasicInboundChannelV2Caller) SourceChannelID(opts *bind.CallOpts) (uint8, error) {
	var out []interface{}
	err := _BasicInboundChannelV2.contract.Call(opts, &out, "sourceChannelID")

	if err != nil {
		return *new(uint8), err
	}

	out0 := *abi.ConvertType(out[0], new(uint8)).(*uint8)

	return out0, err

}

// SourceChannelID is a free data retrieval call binding the contract method 0x157fb143.
//
// Solidity: function sourceChannelID() view returns(uint8)
func (_BasicInboundChannelV2 *BasicInboundChannelV2Session) SourceChannelID() (uint8, error) {
	return _BasicInboundChannelV2.Contract.SourceChannelID(&_BasicInboundChannelV2.CallOpts)
}

// SourceChannelID is a free data retrieval call binding the contract method 0x157fb143.
//
// Solidity: function sourceChannelID() view returns(uint8)
func (_BasicInboundChannelV2 *BasicInboundChannelV2CallerSession) SourceChannelID() (uint8, error) {
	return _BasicInboundChannelV2.Contract.SourceChannelID(&_BasicInboundChannelV2.CallOpts)
}

// Submit is a paid mutator transaction binding the contract method 0xecc18dba.
//
// Solidity: function submit((uint8,bytes32,uint64,(uint64,address,bytes)[]) bundle, bytes32[] leafProof, bool[] hashSides, bytes proof) returns()
func (_BasicInboundChannelV2 *BasicInboundChannelV2Transactor) Submit(opts *bind.TransactOpts, bundle BasicInboundChannelV2MessageBundle, leafProof [][32]byte, hashSides []bool, proof []byte) (*types.Transaction, error) {
	return _BasicInboundChannelV2.contract.Transact(opts, "submit", bundle, leafProof, hashSides, proof)
}

// Submit is a paid mutator transaction binding the contract method 0xecc18dba.
//
// Solidity: function submit((uint8,bytes32,uint64,(uint64,address,bytes)[]) bundle, bytes32[] leafProof, bool[] hashSides, bytes proof) returns()
func (_BasicInboundChannelV2 *BasicInboundChannelV2Session) Submit(bundle BasicInboundChannelV2MessageBundle, leafProof [][32]byte, hashSides []bool, proof []byte) (*types.Transaction, error) {
	return _BasicInboundChannelV2.Contract.Submit(&_BasicInboundChannelV2.TransactOpts, bundle, leafProof, hashSides, proof)
}

// Submit is a paid mutator transaction binding the contract method 0xecc18dba.
//
// Solidity: function submit((uint8,bytes32,uint64,(uint64,address,bytes)[]) bundle, bytes32[] leafProof, bool[] hashSides, bytes proof) returns()
func (_BasicInboundChannelV2 *BasicInboundChannelV2TransactorSession) Submit(bundle BasicInboundChannelV2MessageBundle, leafProof [][32]byte, hashSides []bool, proof []byte) (*types.Transaction, error) {
	return _BasicInboundChannelV2.Contract.Submit(&_BasicInboundChannelV2.TransactOpts, bundle, leafProof, hashSides, proof)
}

// BasicInboundChannelV2MessageDispatchedIterator is returned from FilterMessageDispatched and is used to iterate over the raw logs and unpacked data for MessageDispatched events raised by the BasicInboundChannelV2 contract.
type BasicInboundChannelV2MessageDispatchedIterator struct {
	Event *BasicInboundChannelV2MessageDispatched // Event containing the contract specifics and raw log

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
func (it *BasicInboundChannelV2MessageDispatchedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(BasicInboundChannelV2MessageDispatched)
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
		it.Event = new(BasicInboundChannelV2MessageDispatched)
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
func (it *BasicInboundChannelV2MessageDispatchedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *BasicInboundChannelV2MessageDispatchedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// BasicInboundChannelV2MessageDispatched represents a MessageDispatched event raised by the BasicInboundChannelV2 contract.
type BasicInboundChannelV2MessageDispatched struct {
	Id     uint64
	Result bool
	Raw    types.Log // Blockchain specific contextual infos
}

// FilterMessageDispatched is a free log retrieval operation binding the contract event 0x504b093d860dc827c72a879d052fd8ac6b4c2af80c5f3a634654f172690bf10a.
//
// Solidity: event MessageDispatched(uint64 id, bool result)
func (_BasicInboundChannelV2 *BasicInboundChannelV2Filterer) FilterMessageDispatched(opts *bind.FilterOpts) (*BasicInboundChannelV2MessageDispatchedIterator, error) {

	logs, sub, err := _BasicInboundChannelV2.contract.FilterLogs(opts, "MessageDispatched")
	if err != nil {
		return nil, err
	}
	return &BasicInboundChannelV2MessageDispatchedIterator{contract: _BasicInboundChannelV2.contract, event: "MessageDispatched", logs: logs, sub: sub}, nil
}

// WatchMessageDispatched is a free log subscription operation binding the contract event 0x504b093d860dc827c72a879d052fd8ac6b4c2af80c5f3a634654f172690bf10a.
//
// Solidity: event MessageDispatched(uint64 id, bool result)
func (_BasicInboundChannelV2 *BasicInboundChannelV2Filterer) WatchMessageDispatched(opts *bind.WatchOpts, sink chan<- *BasicInboundChannelV2MessageDispatched) (event.Subscription, error) {

	logs, sub, err := _BasicInboundChannelV2.contract.WatchLogs(opts, "MessageDispatched")
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(BasicInboundChannelV2MessageDispatched)
				if err := _BasicInboundChannelV2.contract.UnpackLog(event, "MessageDispatched", log); err != nil {
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

// ParseMessageDispatched is a log parse operation binding the contract event 0x504b093d860dc827c72a879d052fd8ac6b4c2af80c5f3a634654f172690bf10a.
//
// Solidity: event MessageDispatched(uint64 id, bool result)
func (_BasicInboundChannelV2 *BasicInboundChannelV2Filterer) ParseMessageDispatched(log types.Log) (*BasicInboundChannelV2MessageDispatched, error) {
	event := new(BasicInboundChannelV2MessageDispatched)
	if err := _BasicInboundChannelV2.contract.UnpackLog(event, "MessageDispatched", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}
