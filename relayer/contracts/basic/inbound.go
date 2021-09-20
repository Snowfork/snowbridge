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

// BasicInboundChannelMessage is an auto generated low-level Go binding around an user-defined struct.
type BasicInboundChannelMessage struct {
	Target  common.Address
	Nonce   uint64
	Payload []byte
}

// ParachainLightClientBeefyMMRLeafPartial is an auto generated low-level Go binding around an user-defined struct.
type ParachainLightClientBeefyMMRLeafPartial struct {
	Version              uint8
	ParentNumber         uint32
	ParentHash           [32]byte
	NextAuthoritySetId   uint64
	NextAuthoritySetLen  uint32
	NextAuthoritySetRoot [32]byte
}

// ParachainLightClientParachainHeadProof is an auto generated low-level Go binding around an user-defined struct.
type ParachainLightClientParachainHeadProof struct {
	Pos   *big.Int
	Width *big.Int
	Proof [][32]byte
}

// ParachainLightClientParachainVerifyInput is an auto generated low-level Go binding around an user-defined struct.
type ParachainLightClientParachainVerifyInput struct {
	OwnParachainHeadPrefixBytes []byte
	OwnParachainHeadSuffixBytes []byte
	ParachainHeadProof          ParachainLightClientParachainHeadProof
}

// SimplifiedMMRProof is an auto generated low-level Go binding around an user-defined struct.
type SimplifiedMMRProof struct {
	MerkleProofItems         [][32]byte
	MerkleProofOrderBitField uint64
}

// BasicInboundChannelABI is the input ABI used to generate the binding from.
const BasicInboundChannelABI = "[{\"inputs\":[{\"internalType\":\"contractBeefyLightClient\",\"name\":\"_beefyLightClient\",\"type\":\"address\"}],\"stateMutability\":\"nonpayable\",\"type\":\"constructor\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":false,\"internalType\":\"uint64\",\"name\":\"nonce\",\"type\":\"uint64\"},{\"indexed\":false,\"internalType\":\"bool\",\"name\":\"result\",\"type\":\"bool\"}],\"name\":\"MessageDispatched\",\"type\":\"event\"},{\"inputs\":[],\"name\":\"GAS_BUFFER\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"MAX_GAS_PER_MESSAGE\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"beefyLightClient\",\"outputs\":[{\"internalType\":\"contractBeefyLightClient\",\"name\":\"\",\"type\":\"address\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"nonce\",\"outputs\":[{\"internalType\":\"uint64\",\"name\":\"\",\"type\":\"uint64\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"components\":[{\"internalType\":\"address\",\"name\":\"target\",\"type\":\"address\"},{\"internalType\":\"uint64\",\"name\":\"nonce\",\"type\":\"uint64\"},{\"internalType\":\"bytes\",\"name\":\"payload\",\"type\":\"bytes\"}],\"internalType\":\"structBasicInboundChannel.Message[]\",\"name\":\"_messages\",\"type\":\"tuple[]\"},{\"components\":[{\"internalType\":\"bytes\",\"name\":\"ownParachainHeadPrefixBytes\",\"type\":\"bytes\"},{\"internalType\":\"bytes\",\"name\":\"ownParachainHeadSuffixBytes\",\"type\":\"bytes\"},{\"components\":[{\"internalType\":\"uint256\",\"name\":\"pos\",\"type\":\"uint256\"},{\"internalType\":\"uint256\",\"name\":\"width\",\"type\":\"uint256\"},{\"internalType\":\"bytes32[]\",\"name\":\"proof\",\"type\":\"bytes32[]\"}],\"internalType\":\"structParachainLightClient.ParachainHeadProof\",\"name\":\"parachainHeadProof\",\"type\":\"tuple\"}],\"internalType\":\"structParachainLightClient.ParachainVerifyInput\",\"name\":\"_parachainVerifyInput\",\"type\":\"tuple\"},{\"components\":[{\"internalType\":\"uint8\",\"name\":\"version\",\"type\":\"uint8\"},{\"internalType\":\"uint32\",\"name\":\"parentNumber\",\"type\":\"uint32\"},{\"internalType\":\"bytes32\",\"name\":\"parentHash\",\"type\":\"bytes32\"},{\"internalType\":\"uint64\",\"name\":\"nextAuthoritySetId\",\"type\":\"uint64\"},{\"internalType\":\"uint32\",\"name\":\"nextAuthoritySetLen\",\"type\":\"uint32\"},{\"internalType\":\"bytes32\",\"name\":\"nextAuthoritySetRoot\",\"type\":\"bytes32\"}],\"internalType\":\"structParachainLightClient.BeefyMMRLeafPartial\",\"name\":\"_beefyMMRLeafPartial\",\"type\":\"tuple\"},{\"components\":[{\"internalType\":\"bytes32[]\",\"name\":\"merkleProofItems\",\"type\":\"bytes32[]\"},{\"internalType\":\"uint64\",\"name\":\"merkleProofOrderBitField\",\"type\":\"uint64\"}],\"internalType\":\"structSimplifiedMMRProof\",\"name\":\"proof\",\"type\":\"tuple\"}],\"name\":\"submit\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"}]"

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

// GASBUFFER is a free data retrieval call binding the contract method 0xc7e67360.
//
// Solidity: function GAS_BUFFER() view returns(uint256)
func (_BasicInboundChannel *BasicInboundChannelCaller) GASBUFFER(opts *bind.CallOpts) (*big.Int, error) {
	var out []interface{}
	err := _BasicInboundChannel.contract.Call(opts, &out, "GAS_BUFFER")

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// GASBUFFER is a free data retrieval call binding the contract method 0xc7e67360.
//
// Solidity: function GAS_BUFFER() view returns(uint256)
func (_BasicInboundChannel *BasicInboundChannelSession) GASBUFFER() (*big.Int, error) {
	return _BasicInboundChannel.Contract.GASBUFFER(&_BasicInboundChannel.CallOpts)
}

// GASBUFFER is a free data retrieval call binding the contract method 0xc7e67360.
//
// Solidity: function GAS_BUFFER() view returns(uint256)
func (_BasicInboundChannel *BasicInboundChannelCallerSession) GASBUFFER() (*big.Int, error) {
	return _BasicInboundChannel.Contract.GASBUFFER(&_BasicInboundChannel.CallOpts)
}

// MAXGASPERMESSAGE is a free data retrieval call binding the contract method 0x49bee574.
//
// Solidity: function MAX_GAS_PER_MESSAGE() view returns(uint256)
func (_BasicInboundChannel *BasicInboundChannelCaller) MAXGASPERMESSAGE(opts *bind.CallOpts) (*big.Int, error) {
	var out []interface{}
	err := _BasicInboundChannel.contract.Call(opts, &out, "MAX_GAS_PER_MESSAGE")

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// MAXGASPERMESSAGE is a free data retrieval call binding the contract method 0x49bee574.
//
// Solidity: function MAX_GAS_PER_MESSAGE() view returns(uint256)
func (_BasicInboundChannel *BasicInboundChannelSession) MAXGASPERMESSAGE() (*big.Int, error) {
	return _BasicInboundChannel.Contract.MAXGASPERMESSAGE(&_BasicInboundChannel.CallOpts)
}

// MAXGASPERMESSAGE is a free data retrieval call binding the contract method 0x49bee574.
//
// Solidity: function MAX_GAS_PER_MESSAGE() view returns(uint256)
func (_BasicInboundChannel *BasicInboundChannelCallerSession) MAXGASPERMESSAGE() (*big.Int, error) {
	return _BasicInboundChannel.Contract.MAXGASPERMESSAGE(&_BasicInboundChannel.CallOpts)
}

// BeefyLightClient is a free data retrieval call binding the contract method 0xaf41c33e.
//
// Solidity: function beefyLightClient() view returns(address)
func (_BasicInboundChannel *BasicInboundChannelCaller) BeefyLightClient(opts *bind.CallOpts) (common.Address, error) {
	var out []interface{}
	err := _BasicInboundChannel.contract.Call(opts, &out, "beefyLightClient")

	if err != nil {
		return *new(common.Address), err
	}

	out0 := *abi.ConvertType(out[0], new(common.Address)).(*common.Address)

	return out0, err

}

// BeefyLightClient is a free data retrieval call binding the contract method 0xaf41c33e.
//
// Solidity: function beefyLightClient() view returns(address)
func (_BasicInboundChannel *BasicInboundChannelSession) BeefyLightClient() (common.Address, error) {
	return _BasicInboundChannel.Contract.BeefyLightClient(&_BasicInboundChannel.CallOpts)
}

// BeefyLightClient is a free data retrieval call binding the contract method 0xaf41c33e.
//
// Solidity: function beefyLightClient() view returns(address)
func (_BasicInboundChannel *BasicInboundChannelCallerSession) BeefyLightClient() (common.Address, error) {
	return _BasicInboundChannel.Contract.BeefyLightClient(&_BasicInboundChannel.CallOpts)
}

// Nonce is a free data retrieval call binding the contract method 0xaffed0e0.
//
// Solidity: function nonce() view returns(uint64)
func (_BasicInboundChannel *BasicInboundChannelCaller) Nonce(opts *bind.CallOpts) (uint64, error) {
	var out []interface{}
	err := _BasicInboundChannel.contract.Call(opts, &out, "nonce")

	if err != nil {
		return *new(uint64), err
	}

	out0 := *abi.ConvertType(out[0], new(uint64)).(*uint64)

	return out0, err

}

// Nonce is a free data retrieval call binding the contract method 0xaffed0e0.
//
// Solidity: function nonce() view returns(uint64)
func (_BasicInboundChannel *BasicInboundChannelSession) Nonce() (uint64, error) {
	return _BasicInboundChannel.Contract.Nonce(&_BasicInboundChannel.CallOpts)
}

// Nonce is a free data retrieval call binding the contract method 0xaffed0e0.
//
// Solidity: function nonce() view returns(uint64)
func (_BasicInboundChannel *BasicInboundChannelCallerSession) Nonce() (uint64, error) {
	return _BasicInboundChannel.Contract.Nonce(&_BasicInboundChannel.CallOpts)
}

// Submit is a paid mutator transaction binding the contract method 0x1085473a.
//
// Solidity: function submit((address,uint64,bytes)[] _messages, (bytes,bytes,(uint256,uint256,bytes32[])) _parachainVerifyInput, (uint8,uint32,bytes32,uint64,uint32,bytes32) _beefyMMRLeafPartial, (bytes32[],uint64) proof) returns()
func (_BasicInboundChannel *BasicInboundChannelTransactor) Submit(opts *bind.TransactOpts, _messages []BasicInboundChannelMessage, _parachainVerifyInput ParachainLightClientParachainVerifyInput, _beefyMMRLeafPartial ParachainLightClientBeefyMMRLeafPartial, proof SimplifiedMMRProof) (*types.Transaction, error) {
	return _BasicInboundChannel.contract.Transact(opts, "submit", _messages, _parachainVerifyInput, _beefyMMRLeafPartial, proof)
}

// Submit is a paid mutator transaction binding the contract method 0x1085473a.
//
// Solidity: function submit((address,uint64,bytes)[] _messages, (bytes,bytes,(uint256,uint256,bytes32[])) _parachainVerifyInput, (uint8,uint32,bytes32,uint64,uint32,bytes32) _beefyMMRLeafPartial, (bytes32[],uint64) proof) returns()
func (_BasicInboundChannel *BasicInboundChannelSession) Submit(_messages []BasicInboundChannelMessage, _parachainVerifyInput ParachainLightClientParachainVerifyInput, _beefyMMRLeafPartial ParachainLightClientBeefyMMRLeafPartial, proof SimplifiedMMRProof) (*types.Transaction, error) {
	return _BasicInboundChannel.Contract.Submit(&_BasicInboundChannel.TransactOpts, _messages, _parachainVerifyInput, _beefyMMRLeafPartial, proof)
}

// Submit is a paid mutator transaction binding the contract method 0x1085473a.
//
// Solidity: function submit((address,uint64,bytes)[] _messages, (bytes,bytes,(uint256,uint256,bytes32[])) _parachainVerifyInput, (uint8,uint32,bytes32,uint64,uint32,bytes32) _beefyMMRLeafPartial, (bytes32[],uint64) proof) returns()
func (_BasicInboundChannel *BasicInboundChannelTransactorSession) Submit(_messages []BasicInboundChannelMessage, _parachainVerifyInput ParachainLightClientParachainVerifyInput, _beefyMMRLeafPartial ParachainLightClientBeefyMMRLeafPartial, proof SimplifiedMMRProof) (*types.Transaction, error) {
	return _BasicInboundChannel.Contract.Submit(&_BasicInboundChannel.TransactOpts, _messages, _parachainVerifyInput, _beefyMMRLeafPartial, proof)
}

// BasicInboundChannelMessageDispatchedIterator is returned from FilterMessageDispatched and is used to iterate over the raw logs and unpacked data for MessageDispatched events raised by the BasicInboundChannel contract.
type BasicInboundChannelMessageDispatchedIterator struct {
	Event *BasicInboundChannelMessageDispatched // Event containing the contract specifics and raw log

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
func (it *BasicInboundChannelMessageDispatchedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(BasicInboundChannelMessageDispatched)
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
		it.Event = new(BasicInboundChannelMessageDispatched)
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
func (it *BasicInboundChannelMessageDispatchedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *BasicInboundChannelMessageDispatchedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// BasicInboundChannelMessageDispatched represents a MessageDispatched event raised by the BasicInboundChannel contract.
type BasicInboundChannelMessageDispatched struct {
	Nonce  uint64
	Result bool
	Raw    types.Log // Blockchain specific contextual infos
}

// FilterMessageDispatched is a free log retrieval operation binding the contract event 0x504b093d860dc827c72a879d052fd8ac6b4c2af80c5f3a634654f172690bf10a.
//
// Solidity: event MessageDispatched(uint64 nonce, bool result)
func (_BasicInboundChannel *BasicInboundChannelFilterer) FilterMessageDispatched(opts *bind.FilterOpts) (*BasicInboundChannelMessageDispatchedIterator, error) {

	logs, sub, err := _BasicInboundChannel.contract.FilterLogs(opts, "MessageDispatched")
	if err != nil {
		return nil, err
	}
	return &BasicInboundChannelMessageDispatchedIterator{contract: _BasicInboundChannel.contract, event: "MessageDispatched", logs: logs, sub: sub}, nil
}

// WatchMessageDispatched is a free log subscription operation binding the contract event 0x504b093d860dc827c72a879d052fd8ac6b4c2af80c5f3a634654f172690bf10a.
//
// Solidity: event MessageDispatched(uint64 nonce, bool result)
func (_BasicInboundChannel *BasicInboundChannelFilterer) WatchMessageDispatched(opts *bind.WatchOpts, sink chan<- *BasicInboundChannelMessageDispatched) (event.Subscription, error) {

	logs, sub, err := _BasicInboundChannel.contract.WatchLogs(opts, "MessageDispatched")
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(BasicInboundChannelMessageDispatched)
				if err := _BasicInboundChannel.contract.UnpackLog(event, "MessageDispatched", log); err != nil {
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
// Solidity: event MessageDispatched(uint64 nonce, bool result)
func (_BasicInboundChannel *BasicInboundChannelFilterer) ParseMessageDispatched(log types.Log) (*BasicInboundChannelMessageDispatched, error) {
	event := new(BasicInboundChannelMessageDispatched)
	if err := _BasicInboundChannel.contract.UnpackLog(event, "MessageDispatched", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}
