// Code generated - DO NOT EDIT.
// This file is a generated binding and any manual changes will be lost.

package outbound

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

// IncentivizedOutboundChannelABI is the input ABI used to generate the binding from.
const IncentivizedOutboundChannelABI = "[{\"anonymous\":false,\"inputs\":[{\"indexed\":false,\"internalType\":\"address\",\"name\":\"source\",\"type\":\"address\"},{\"indexed\":false,\"internalType\":\"uint64\",\"name\":\"nonce\",\"type\":\"uint64\"},{\"indexed\":false,\"internalType\":\"bytes\",\"name\":\"payload\",\"type\":\"bytes\"}],\"name\":\"Message\",\"type\":\"event\"},{\"inputs\":[],\"name\":\"nonce\",\"outputs\":[{\"internalType\":\"uint64\",\"name\":\"\",\"type\":\"uint64\"}],\"stateMutability\":\"view\",\"type\":\"function\",\"constant\":true},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\"},{\"internalType\":\"bytes\",\"name\":\"payload\",\"type\":\"bytes\"}],\"name\":\"submit\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"}]"

// IncentivizedOutboundChannel is an auto generated Go binding around an Ethereum contract.
type IncentivizedOutboundChannel struct {
	IncentivizedOutboundChannelCaller     // Read-only binding to the contract
	IncentivizedOutboundChannelTransactor // Write-only binding to the contract
	IncentivizedOutboundChannelFilterer   // Log filterer for contract events
}

// IncentivizedOutboundChannelCaller is an auto generated read-only Go binding around an Ethereum contract.
type IncentivizedOutboundChannelCaller struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// IncentivizedOutboundChannelTransactor is an auto generated write-only Go binding around an Ethereum contract.
type IncentivizedOutboundChannelTransactor struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// IncentivizedOutboundChannelFilterer is an auto generated log filtering Go binding around an Ethereum contract events.
type IncentivizedOutboundChannelFilterer struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// IncentivizedOutboundChannelSession is an auto generated Go binding around an Ethereum contract,
// with pre-set call and transact options.
type IncentivizedOutboundChannelSession struct {
	Contract     *IncentivizedOutboundChannel // Generic contract binding to set the session for
	CallOpts     bind.CallOpts                // Call options to use throughout this session
	TransactOpts bind.TransactOpts            // Transaction auth options to use throughout this session
}

// IncentivizedOutboundChannelCallerSession is an auto generated read-only Go binding around an Ethereum contract,
// with pre-set call options.
type IncentivizedOutboundChannelCallerSession struct {
	Contract *IncentivizedOutboundChannelCaller // Generic contract caller binding to set the session for
	CallOpts bind.CallOpts                      // Call options to use throughout this session
}

// IncentivizedOutboundChannelTransactorSession is an auto generated write-only Go binding around an Ethereum contract,
// with pre-set transact options.
type IncentivizedOutboundChannelTransactorSession struct {
	Contract     *IncentivizedOutboundChannelTransactor // Generic contract transactor binding to set the session for
	TransactOpts bind.TransactOpts                      // Transaction auth options to use throughout this session
}

// IncentivizedOutboundChannelRaw is an auto generated low-level Go binding around an Ethereum contract.
type IncentivizedOutboundChannelRaw struct {
	Contract *IncentivizedOutboundChannel // Generic contract binding to access the raw methods on
}

// IncentivizedOutboundChannelCallerRaw is an auto generated low-level read-only Go binding around an Ethereum contract.
type IncentivizedOutboundChannelCallerRaw struct {
	Contract *IncentivizedOutboundChannelCaller // Generic read-only contract binding to access the raw methods on
}

// IncentivizedOutboundChannelTransactorRaw is an auto generated low-level write-only Go binding around an Ethereum contract.
type IncentivizedOutboundChannelTransactorRaw struct {
	Contract *IncentivizedOutboundChannelTransactor // Generic write-only contract binding to access the raw methods on
}

// NewIncentivizedOutboundChannel creates a new instance of IncentivizedOutboundChannel, bound to a specific deployed contract.
func NewIncentivizedOutboundChannel(address common.Address, backend bind.ContractBackend) (*IncentivizedOutboundChannel, error) {
	contract, err := bindIncentivizedOutboundChannel(address, backend, backend, backend)
	if err != nil {
		return nil, err
	}
	return &IncentivizedOutboundChannel{IncentivizedOutboundChannelCaller: IncentivizedOutboundChannelCaller{contract: contract}, IncentivizedOutboundChannelTransactor: IncentivizedOutboundChannelTransactor{contract: contract}, IncentivizedOutboundChannelFilterer: IncentivizedOutboundChannelFilterer{contract: contract}}, nil
}

// NewIncentivizedOutboundChannelCaller creates a new read-only instance of IncentivizedOutboundChannel, bound to a specific deployed contract.
func NewIncentivizedOutboundChannelCaller(address common.Address, caller bind.ContractCaller) (*IncentivizedOutboundChannelCaller, error) {
	contract, err := bindIncentivizedOutboundChannel(address, caller, nil, nil)
	if err != nil {
		return nil, err
	}
	return &IncentivizedOutboundChannelCaller{contract: contract}, nil
}

// NewIncentivizedOutboundChannelTransactor creates a new write-only instance of IncentivizedOutboundChannel, bound to a specific deployed contract.
func NewIncentivizedOutboundChannelTransactor(address common.Address, transactor bind.ContractTransactor) (*IncentivizedOutboundChannelTransactor, error) {
	contract, err := bindIncentivizedOutboundChannel(address, nil, transactor, nil)
	if err != nil {
		return nil, err
	}
	return &IncentivizedOutboundChannelTransactor{contract: contract}, nil
}

// NewIncentivizedOutboundChannelFilterer creates a new log filterer instance of IncentivizedOutboundChannel, bound to a specific deployed contract.
func NewIncentivizedOutboundChannelFilterer(address common.Address, filterer bind.ContractFilterer) (*IncentivizedOutboundChannelFilterer, error) {
	contract, err := bindIncentivizedOutboundChannel(address, nil, nil, filterer)
	if err != nil {
		return nil, err
	}
	return &IncentivizedOutboundChannelFilterer{contract: contract}, nil
}

// bindIncentivizedOutboundChannel binds a generic wrapper to an already deployed contract.
func bindIncentivizedOutboundChannel(address common.Address, caller bind.ContractCaller, transactor bind.ContractTransactor, filterer bind.ContractFilterer) (*bind.BoundContract, error) {
	parsed, err := abi.JSON(strings.NewReader(IncentivizedOutboundChannelABI))
	if err != nil {
		return nil, err
	}
	return bind.NewBoundContract(address, parsed, caller, transactor, filterer), nil
}

// Call invokes the (constant) contract method with params as input values and
// sets the output to result. The result type might be a single field for simple
// returns, a slice of interfaces for anonymous returns and a struct for named
// returns.
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelRaw) Call(opts *bind.CallOpts, result *[]interface{}, method string, params ...interface{}) error {
	return _IncentivizedOutboundChannel.Contract.IncentivizedOutboundChannelCaller.contract.Call(opts, result, method, params...)
}

// Transfer initiates a plain transaction to move funds to the contract, calling
// its default method if one is available.
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelRaw) Transfer(opts *bind.TransactOpts) (*types.Transaction, error) {
	return _IncentivizedOutboundChannel.Contract.IncentivizedOutboundChannelTransactor.contract.Transfer(opts)
}

// Transact invokes the (paid) contract method with params as input values.
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelRaw) Transact(opts *bind.TransactOpts, method string, params ...interface{}) (*types.Transaction, error) {
	return _IncentivizedOutboundChannel.Contract.IncentivizedOutboundChannelTransactor.contract.Transact(opts, method, params...)
}

// Call invokes the (constant) contract method with params as input values and
// sets the output to result. The result type might be a single field for simple
// returns, a slice of interfaces for anonymous returns and a struct for named
// returns.
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelCallerRaw) Call(opts *bind.CallOpts, result *[]interface{}, method string, params ...interface{}) error {
	return _IncentivizedOutboundChannel.Contract.contract.Call(opts, result, method, params...)
}

// Transfer initiates a plain transaction to move funds to the contract, calling
// its default method if one is available.
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelTransactorRaw) Transfer(opts *bind.TransactOpts) (*types.Transaction, error) {
	return _IncentivizedOutboundChannel.Contract.contract.Transfer(opts)
}

// Transact invokes the (paid) contract method with params as input values.
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelTransactorRaw) Transact(opts *bind.TransactOpts, method string, params ...interface{}) (*types.Transaction, error) {
	return _IncentivizedOutboundChannel.Contract.contract.Transact(opts, method, params...)
}

// Nonce is a free data retrieval call binding the contract method 0xaffed0e0.
//
// Solidity: function nonce() view returns(uint64)
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelCaller) Nonce(opts *bind.CallOpts) (uint64, error) {
	var out []interface{}
	err := _IncentivizedOutboundChannel.contract.Call(opts, &out, "nonce")

	if err != nil {
		return *new(uint64), err
	}

	out0 := *abi.ConvertType(out[0], new(uint64)).(*uint64)

	return out0, err

}

// Nonce is a free data retrieval call binding the contract method 0xaffed0e0.
//
// Solidity: function nonce() view returns(uint64)
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelSession) Nonce() (uint64, error) {
	return _IncentivizedOutboundChannel.Contract.Nonce(&_IncentivizedOutboundChannel.CallOpts)
}

// Nonce is a free data retrieval call binding the contract method 0xaffed0e0.
//
// Solidity: function nonce() view returns(uint64)
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelCallerSession) Nonce() (uint64, error) {
	return _IncentivizedOutboundChannel.Contract.Nonce(&_IncentivizedOutboundChannel.CallOpts)
}

// Submit is a paid mutator transaction binding the contract method 0x76846edd.
//
// Solidity: function submit(address , bytes payload) returns()
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelTransactor) Submit(opts *bind.TransactOpts, arg0 common.Address, payload []byte) (*types.Transaction, error) {
	return _IncentivizedOutboundChannel.contract.Transact(opts, "submit", arg0, payload)
}

// Submit is a paid mutator transaction binding the contract method 0x76846edd.
//
// Solidity: function submit(address , bytes payload) returns()
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelSession) Submit(arg0 common.Address, payload []byte) (*types.Transaction, error) {
	return _IncentivizedOutboundChannel.Contract.Submit(&_IncentivizedOutboundChannel.TransactOpts, arg0, payload)
}

// Submit is a paid mutator transaction binding the contract method 0x76846edd.
//
// Solidity: function submit(address , bytes payload) returns()
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelTransactorSession) Submit(arg0 common.Address, payload []byte) (*types.Transaction, error) {
	return _IncentivizedOutboundChannel.Contract.Submit(&_IncentivizedOutboundChannel.TransactOpts, arg0, payload)
}

// IncentivizedOutboundChannelMessageIterator is returned from FilterMessage and is used to iterate over the raw logs and unpacked data for Message events raised by the IncentivizedOutboundChannel contract.
type IncentivizedOutboundChannelMessageIterator struct {
	Event *IncentivizedOutboundChannelMessage // Event containing the contract specifics and raw log

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
func (it *IncentivizedOutboundChannelMessageIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(IncentivizedOutboundChannelMessage)
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
		it.Event = new(IncentivizedOutboundChannelMessage)
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
func (it *IncentivizedOutboundChannelMessageIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *IncentivizedOutboundChannelMessageIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// IncentivizedOutboundChannelMessage represents a Message event raised by the IncentivizedOutboundChannel contract.
type IncentivizedOutboundChannelMessage struct {
	Source  common.Address
	Nonce   uint64
	Payload []byte
	Raw     types.Log // Blockchain specific contextual infos
}

// FilterMessage is a free log retrieval operation binding the contract event 0x779b38144a38cfc4351816442048b17fe24ba2b0e0c63446b576e8281160b15b.
//
// Solidity: event Message(address source, uint64 nonce, bytes payload)
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelFilterer) FilterMessage(opts *bind.FilterOpts) (*IncentivizedOutboundChannelMessageIterator, error) {

	logs, sub, err := _IncentivizedOutboundChannel.contract.FilterLogs(opts, "Message")
	if err != nil {
		return nil, err
	}
	return &IncentivizedOutboundChannelMessageIterator{contract: _IncentivizedOutboundChannel.contract, event: "Message", logs: logs, sub: sub}, nil
}

// WatchMessage is a free log subscription operation binding the contract event 0x779b38144a38cfc4351816442048b17fe24ba2b0e0c63446b576e8281160b15b.
//
// Solidity: event Message(address source, uint64 nonce, bytes payload)
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelFilterer) WatchMessage(opts *bind.WatchOpts, sink chan<- *IncentivizedOutboundChannelMessage) (event.Subscription, error) {

	logs, sub, err := _IncentivizedOutboundChannel.contract.WatchLogs(opts, "Message")
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(IncentivizedOutboundChannelMessage)
				if err := _IncentivizedOutboundChannel.contract.UnpackLog(event, "Message", log); err != nil {
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
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelFilterer) ParseMessage(log types.Log) (*IncentivizedOutboundChannelMessage, error) {
	event := new(IncentivizedOutboundChannelMessage)
	if err := _IncentivizedOutboundChannel.contract.UnpackLog(event, "Message", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}
