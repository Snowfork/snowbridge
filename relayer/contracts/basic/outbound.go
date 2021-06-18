// Code generated - DO NOT EDIT.
// This file is a generated binding and any manual changes will be lost.

package basic

import (
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
	_ = big.NewInt
	_ = strings.NewReader
	_ = ethereum.NotFound
	_ = bind.Bind
	_ = common.Big1
	_ = types.BloomLookup
	_ = event.NewSubscription
)

// BasicOutboundChannelABI is the input ABI used to generate the binding from.
const BasicOutboundChannelABI = "[{\"anonymous\":false,\"inputs\":[{\"indexed\":false,\"internalType\":\"address\",\"name\":\"source\",\"type\":\"address\"},{\"indexed\":false,\"internalType\":\"uint64\",\"name\":\"nonce\",\"type\":\"uint64\"},{\"indexed\":false,\"internalType\":\"bytes\",\"name\":\"payload\",\"type\":\"bytes\"}],\"name\":\"Message\",\"type\":\"event\"},{\"inputs\":[],\"name\":\"nonce\",\"outputs\":[{\"internalType\":\"uint64\",\"name\":\"\",\"type\":\"uint64\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\"},{\"internalType\":\"bytes\",\"name\":\"payload\",\"type\":\"bytes\"}],\"name\":\"submit\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"}]"

// BasicOutboundChannel is an auto generated Go binding around an Ethereum contract.
type BasicOutboundChannel struct {
	BasicOutboundChannelCaller     // Read-only binding to the contract
	BasicOutboundChannelTransactor // Write-only binding to the contract
	BasicOutboundChannelFilterer   // Log filterer for contract events
}

// BasicOutboundChannelCaller is an auto generated read-only Go binding around an Ethereum contract.
type BasicOutboundChannelCaller struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// BasicOutboundChannelTransactor is an auto generated write-only Go binding around an Ethereum contract.
type BasicOutboundChannelTransactor struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// BasicOutboundChannelFilterer is an auto generated log filtering Go binding around an Ethereum contract events.
type BasicOutboundChannelFilterer struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// BasicOutboundChannelSession is an auto generated Go binding around an Ethereum contract,
// with pre-set call and transact options.
type BasicOutboundChannelSession struct {
	Contract     *BasicOutboundChannel // Generic contract binding to set the session for
	CallOpts     bind.CallOpts         // Call options to use throughout this session
	TransactOpts bind.TransactOpts     // Transaction auth options to use throughout this session
}

// BasicOutboundChannelCallerSession is an auto generated read-only Go binding around an Ethereum contract,
// with pre-set call options.
type BasicOutboundChannelCallerSession struct {
	Contract *BasicOutboundChannelCaller // Generic contract caller binding to set the session for
	CallOpts bind.CallOpts               // Call options to use throughout this session
}

// BasicOutboundChannelTransactorSession is an auto generated write-only Go binding around an Ethereum contract,
// with pre-set transact options.
type BasicOutboundChannelTransactorSession struct {
	Contract     *BasicOutboundChannelTransactor // Generic contract transactor binding to set the session for
	TransactOpts bind.TransactOpts               // Transaction auth options to use throughout this session
}

// BasicOutboundChannelRaw is an auto generated low-level Go binding around an Ethereum contract.
type BasicOutboundChannelRaw struct {
	Contract *BasicOutboundChannel // Generic contract binding to access the raw methods on
}

// BasicOutboundChannelCallerRaw is an auto generated low-level read-only Go binding around an Ethereum contract.
type BasicOutboundChannelCallerRaw struct {
	Contract *BasicOutboundChannelCaller // Generic read-only contract binding to access the raw methods on
}

// BasicOutboundChannelTransactorRaw is an auto generated low-level write-only Go binding around an Ethereum contract.
type BasicOutboundChannelTransactorRaw struct {
	Contract *BasicOutboundChannelTransactor // Generic write-only contract binding to access the raw methods on
}

// NewBasicOutboundChannel creates a new instance of BasicOutboundChannel, bound to a specific deployed contract.
func NewBasicOutboundChannel(address common.Address, backend bind.ContractBackend) (*BasicOutboundChannel, error) {
	contract, err := bindBasicOutboundChannel(address, backend, backend, backend)
	if err != nil {
		return nil, err
	}
	return &BasicOutboundChannel{BasicOutboundChannelCaller: BasicOutboundChannelCaller{contract: contract}, BasicOutboundChannelTransactor: BasicOutboundChannelTransactor{contract: contract}, BasicOutboundChannelFilterer: BasicOutboundChannelFilterer{contract: contract}}, nil
}

// NewBasicOutboundChannelCaller creates a new read-only instance of BasicOutboundChannel, bound to a specific deployed contract.
func NewBasicOutboundChannelCaller(address common.Address, caller bind.ContractCaller) (*BasicOutboundChannelCaller, error) {
	contract, err := bindBasicOutboundChannel(address, caller, nil, nil)
	if err != nil {
		return nil, err
	}
	return &BasicOutboundChannelCaller{contract: contract}, nil
}

// NewBasicOutboundChannelTransactor creates a new write-only instance of BasicOutboundChannel, bound to a specific deployed contract.
func NewBasicOutboundChannelTransactor(address common.Address, transactor bind.ContractTransactor) (*BasicOutboundChannelTransactor, error) {
	contract, err := bindBasicOutboundChannel(address, nil, transactor, nil)
	if err != nil {
		return nil, err
	}
	return &BasicOutboundChannelTransactor{contract: contract}, nil
}

// NewBasicOutboundChannelFilterer creates a new log filterer instance of BasicOutboundChannel, bound to a specific deployed contract.
func NewBasicOutboundChannelFilterer(address common.Address, filterer bind.ContractFilterer) (*BasicOutboundChannelFilterer, error) {
	contract, err := bindBasicOutboundChannel(address, nil, nil, filterer)
	if err != nil {
		return nil, err
	}
	return &BasicOutboundChannelFilterer{contract: contract}, nil
}

// bindBasicOutboundChannel binds a generic wrapper to an already deployed contract.
func bindBasicOutboundChannel(address common.Address, caller bind.ContractCaller, transactor bind.ContractTransactor, filterer bind.ContractFilterer) (*bind.BoundContract, error) {
	parsed, err := abi.JSON(strings.NewReader(BasicOutboundChannelABI))
	if err != nil {
		return nil, err
	}
	return bind.NewBoundContract(address, parsed, caller, transactor, filterer), nil
}

// Call invokes the (constant) contract method with params as input values and
// sets the output to result. The result type might be a single field for simple
// returns, a slice of interfaces for anonymous returns and a struct for named
// returns.
func (_BasicOutboundChannel *BasicOutboundChannelRaw) Call(opts *bind.CallOpts, result *[]interface{}, method string, params ...interface{}) error {
	return _BasicOutboundChannel.Contract.BasicOutboundChannelCaller.contract.Call(opts, result, method, params...)
}

// Transfer initiates a plain transaction to move funds to the contract, calling
// its default method if one is available.
func (_BasicOutboundChannel *BasicOutboundChannelRaw) Transfer(opts *bind.TransactOpts) (*types.Transaction, error) {
	return _BasicOutboundChannel.Contract.BasicOutboundChannelTransactor.contract.Transfer(opts)
}

// Transact invokes the (paid) contract method with params as input values.
func (_BasicOutboundChannel *BasicOutboundChannelRaw) Transact(opts *bind.TransactOpts, method string, params ...interface{}) (*types.Transaction, error) {
	return _BasicOutboundChannel.Contract.BasicOutboundChannelTransactor.contract.Transact(opts, method, params...)
}

// Call invokes the (constant) contract method with params as input values and
// sets the output to result. The result type might be a single field for simple
// returns, a slice of interfaces for anonymous returns and a struct for named
// returns.
func (_BasicOutboundChannel *BasicOutboundChannelCallerRaw) Call(opts *bind.CallOpts, result *[]interface{}, method string, params ...interface{}) error {
	return _BasicOutboundChannel.Contract.contract.Call(opts, result, method, params...)
}

// Transfer initiates a plain transaction to move funds to the contract, calling
// its default method if one is available.
func (_BasicOutboundChannel *BasicOutboundChannelTransactorRaw) Transfer(opts *bind.TransactOpts) (*types.Transaction, error) {
	return _BasicOutboundChannel.Contract.contract.Transfer(opts)
}

// Transact invokes the (paid) contract method with params as input values.
func (_BasicOutboundChannel *BasicOutboundChannelTransactorRaw) Transact(opts *bind.TransactOpts, method string, params ...interface{}) (*types.Transaction, error) {
	return _BasicOutboundChannel.Contract.contract.Transact(opts, method, params...)
}

// Nonce is a free data retrieval call binding the contract method 0xaffed0e0.
//
// Solidity: function nonce() view returns(uint64)
func (_BasicOutboundChannel *BasicOutboundChannelCaller) Nonce(opts *bind.CallOpts) (uint64, error) {
	var out []interface{}
	err := _BasicOutboundChannel.contract.Call(opts, &out, "nonce")

	if err != nil {
		return *new(uint64), err
	}

	out0 := *abi.ConvertType(out[0], new(uint64)).(*uint64)

	return out0, err

}

// Nonce is a free data retrieval call binding the contract method 0xaffed0e0.
//
// Solidity: function nonce() view returns(uint64)
func (_BasicOutboundChannel *BasicOutboundChannelSession) Nonce() (uint64, error) {
	return _BasicOutboundChannel.Contract.Nonce(&_BasicOutboundChannel.CallOpts)
}

// Nonce is a free data retrieval call binding the contract method 0xaffed0e0.
//
// Solidity: function nonce() view returns(uint64)
func (_BasicOutboundChannel *BasicOutboundChannelCallerSession) Nonce() (uint64, error) {
	return _BasicOutboundChannel.Contract.Nonce(&_BasicOutboundChannel.CallOpts)
}

// Submit is a paid mutator transaction binding the contract method 0x76846edd.
//
// Solidity: function submit(address , bytes payload) returns()
func (_BasicOutboundChannel *BasicOutboundChannelTransactor) Submit(opts *bind.TransactOpts, arg0 common.Address, payload []byte) (*types.Transaction, error) {
	return _BasicOutboundChannel.contract.Transact(opts, "submit", arg0, payload)
}

// Submit is a paid mutator transaction binding the contract method 0x76846edd.
//
// Solidity: function submit(address , bytes payload) returns()
func (_BasicOutboundChannel *BasicOutboundChannelSession) Submit(arg0 common.Address, payload []byte) (*types.Transaction, error) {
	return _BasicOutboundChannel.Contract.Submit(&_BasicOutboundChannel.TransactOpts, arg0, payload)
}

// Submit is a paid mutator transaction binding the contract method 0x76846edd.
//
// Solidity: function submit(address , bytes payload) returns()
func (_BasicOutboundChannel *BasicOutboundChannelTransactorSession) Submit(arg0 common.Address, payload []byte) (*types.Transaction, error) {
	return _BasicOutboundChannel.Contract.Submit(&_BasicOutboundChannel.TransactOpts, arg0, payload)
}

// BasicOutboundChannelMessageIterator is returned from FilterMessage and is used to iterate over the raw logs and unpacked data for Message events raised by the BasicOutboundChannel contract.
type BasicOutboundChannelMessageIterator struct {
	Event *BasicOutboundChannelMessage // Event containing the contract specifics and raw log

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
func (it *BasicOutboundChannelMessageIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(BasicOutboundChannelMessage)
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
		it.Event = new(BasicOutboundChannelMessage)
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
func (it *BasicOutboundChannelMessageIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *BasicOutboundChannelMessageIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// BasicOutboundChannelMessage represents a Message event raised by the BasicOutboundChannel contract.
type BasicOutboundChannelMessage struct {
	Source  common.Address
	Nonce   uint64
	Payload []byte
	Raw     types.Log // Blockchain specific contextual infos
}

// FilterMessage is a free log retrieval operation binding the contract event 0x779b38144a38cfc4351816442048b17fe24ba2b0e0c63446b576e8281160b15b.
//
// Solidity: event Message(address source, uint64 nonce, bytes payload)
func (_BasicOutboundChannel *BasicOutboundChannelFilterer) FilterMessage(opts *bind.FilterOpts) (*BasicOutboundChannelMessageIterator, error) {

	logs, sub, err := _BasicOutboundChannel.contract.FilterLogs(opts, "Message")
	if err != nil {
		return nil, err
	}
	return &BasicOutboundChannelMessageIterator{contract: _BasicOutboundChannel.contract, event: "Message", logs: logs, sub: sub}, nil
}

// WatchMessage is a free log subscription operation binding the contract event 0x779b38144a38cfc4351816442048b17fe24ba2b0e0c63446b576e8281160b15b.
//
// Solidity: event Message(address source, uint64 nonce, bytes payload)
func (_BasicOutboundChannel *BasicOutboundChannelFilterer) WatchMessage(opts *bind.WatchOpts, sink chan<- *BasicOutboundChannelMessage) (event.Subscription, error) {

	logs, sub, err := _BasicOutboundChannel.contract.WatchLogs(opts, "Message")
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(BasicOutboundChannelMessage)
				if err := _BasicOutboundChannel.contract.UnpackLog(event, "Message", log); err != nil {
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

// ParseMessage is a log parse operation binding the contract event 0x779b38144a38cfc4351816442048b17fe24ba2b0e0c63446b576e8281160b15b.
//
// Solidity: event Message(address source, uint64 nonce, bytes payload)
func (_BasicOutboundChannel *BasicOutboundChannelFilterer) ParseMessage(log types.Log) (*BasicOutboundChannelMessage, error) {
	event := new(BasicOutboundChannelMessage)
	if err := _BasicOutboundChannel.contract.UnpackLog(event, "Message", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}
