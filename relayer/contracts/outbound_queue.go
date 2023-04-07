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
)

// OutboundQueueMetaData contains all meta data concerning the OutboundQueue contract.
var OutboundQueueMetaData = &bind.MetaData{
	ABI: "[{\"inputs\":[{\"internalType\":\"contractIVault\",\"name\":\"_vault\",\"type\":\"address\"},{\"internalType\":\"uint256\",\"name\":\"_fee\",\"type\":\"uint256\"}],\"stateMutability\":\"nonpayable\",\"type\":\"constructor\"},{\"inputs\":[],\"name\":\"FeePaymentToLow\",\"type\":\"error\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":false,\"internalType\":\"uint256\",\"name\":\"fee\",\"type\":\"uint256\"}],\"name\":\"FeeUpdated\",\"type\":\"event\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":true,\"internalType\":\"bytes\",\"name\":\"dest\",\"type\":\"bytes\"},{\"indexed\":false,\"internalType\":\"uint64\",\"name\":\"nonce\",\"type\":\"uint64\"},{\"indexed\":false,\"internalType\":\"bytes\",\"name\":\"payload\",\"type\":\"bytes\"}],\"name\":\"Message\",\"type\":\"event\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":true,\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\"},{\"indexed\":true,\"internalType\":\"bytes32\",\"name\":\"previousAdminRole\",\"type\":\"bytes32\"},{\"indexed\":true,\"internalType\":\"bytes32\",\"name\":\"newAdminRole\",\"type\":\"bytes32\"}],\"name\":\"RoleAdminChanged\",\"type\":\"event\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":true,\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\"},{\"indexed\":true,\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\"},{\"indexed\":true,\"internalType\":\"address\",\"name\":\"sender\",\"type\":\"address\"}],\"name\":\"RoleGranted\",\"type\":\"event\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":true,\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\"},{\"indexed\":true,\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\"},{\"indexed\":true,\"internalType\":\"address\",\"name\":\"sender\",\"type\":\"address\"}],\"name\":\"RoleRevoked\",\"type\":\"event\"},{\"inputs\":[],\"name\":\"ADMIN_ROLE\",\"outputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"DEFAULT_ADMIN_ROLE\",\"outputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"SUBMIT_ROLE\",\"outputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"fee\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\"}],\"name\":\"getRoleAdmin\",\"outputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\"},{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\"}],\"name\":\"grantRole\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\"},{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\"}],\"name\":\"hasRole\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"bytes\",\"name\":\"\",\"type\":\"bytes\"}],\"name\":\"nonce\",\"outputs\":[{\"internalType\":\"uint64\",\"name\":\"\",\"type\":\"uint64\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\"},{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\"}],\"name\":\"renounceRole\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\"},{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\"}],\"name\":\"revokeRole\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"bytes\",\"name\":\"dest\",\"type\":\"bytes\"},{\"internalType\":\"bytes\",\"name\":\"payload\",\"type\":\"bytes\"}],\"name\":\"submit\",\"outputs\":[],\"stateMutability\":\"payable\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"bytes4\",\"name\":\"interfaceId\",\"type\":\"bytes4\"}],\"name\":\"supportsInterface\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"newFee\",\"type\":\"uint256\"}],\"name\":\"updateFee\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"vault\",\"outputs\":[{\"internalType\":\"contractIVault\",\"name\":\"\",\"type\":\"address\"}],\"stateMutability\":\"view\",\"type\":\"function\"}]",
}

// OutboundQueueABI is the input ABI used to generate the binding from.
// Deprecated: Use OutboundQueueMetaData.ABI instead.
var OutboundQueueABI = OutboundQueueMetaData.ABI

// OutboundQueue is an auto generated Go binding around an Ethereum contract.
type OutboundQueue struct {
	OutboundQueueCaller     // Read-only binding to the contract
	OutboundQueueTransactor // Write-only binding to the contract
	OutboundQueueFilterer   // Log filterer for contract events
}

// OutboundQueueCaller is an auto generated read-only Go binding around an Ethereum contract.
type OutboundQueueCaller struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// OutboundQueueTransactor is an auto generated write-only Go binding around an Ethereum contract.
type OutboundQueueTransactor struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// OutboundQueueFilterer is an auto generated log filtering Go binding around an Ethereum contract events.
type OutboundQueueFilterer struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// OutboundQueueSession is an auto generated Go binding around an Ethereum contract,
// with pre-set call and transact options.
type OutboundQueueSession struct {
	Contract     *OutboundQueue    // Generic contract binding to set the session for
	CallOpts     bind.CallOpts     // Call options to use throughout this session
	TransactOpts bind.TransactOpts // Transaction auth options to use throughout this session
}

// OutboundQueueCallerSession is an auto generated read-only Go binding around an Ethereum contract,
// with pre-set call options.
type OutboundQueueCallerSession struct {
	Contract *OutboundQueueCaller // Generic contract caller binding to set the session for
	CallOpts bind.CallOpts        // Call options to use throughout this session
}

// OutboundQueueTransactorSession is an auto generated write-only Go binding around an Ethereum contract,
// with pre-set transact options.
type OutboundQueueTransactorSession struct {
	Contract     *OutboundQueueTransactor // Generic contract transactor binding to set the session for
	TransactOpts bind.TransactOpts        // Transaction auth options to use throughout this session
}

// OutboundQueueRaw is an auto generated low-level Go binding around an Ethereum contract.
type OutboundQueueRaw struct {
	Contract *OutboundQueue // Generic contract binding to access the raw methods on
}

// OutboundQueueCallerRaw is an auto generated low-level read-only Go binding around an Ethereum contract.
type OutboundQueueCallerRaw struct {
	Contract *OutboundQueueCaller // Generic read-only contract binding to access the raw methods on
}

// OutboundQueueTransactorRaw is an auto generated low-level write-only Go binding around an Ethereum contract.
type OutboundQueueTransactorRaw struct {
	Contract *OutboundQueueTransactor // Generic write-only contract binding to access the raw methods on
}

// NewOutboundQueue creates a new instance of OutboundQueue, bound to a specific deployed contract.
func NewOutboundQueue(address common.Address, backend bind.ContractBackend) (*OutboundQueue, error) {
	contract, err := bindOutboundQueue(address, backend, backend, backend)
	if err != nil {
		return nil, err
	}
	return &OutboundQueue{OutboundQueueCaller: OutboundQueueCaller{contract: contract}, OutboundQueueTransactor: OutboundQueueTransactor{contract: contract}, OutboundQueueFilterer: OutboundQueueFilterer{contract: contract}}, nil
}

// NewOutboundQueueCaller creates a new read-only instance of OutboundQueue, bound to a specific deployed contract.
func NewOutboundQueueCaller(address common.Address, caller bind.ContractCaller) (*OutboundQueueCaller, error) {
	contract, err := bindOutboundQueue(address, caller, nil, nil)
	if err != nil {
		return nil, err
	}
	return &OutboundQueueCaller{contract: contract}, nil
}

// NewOutboundQueueTransactor creates a new write-only instance of OutboundQueue, bound to a specific deployed contract.
func NewOutboundQueueTransactor(address common.Address, transactor bind.ContractTransactor) (*OutboundQueueTransactor, error) {
	contract, err := bindOutboundQueue(address, nil, transactor, nil)
	if err != nil {
		return nil, err
	}
	return &OutboundQueueTransactor{contract: contract}, nil
}

// NewOutboundQueueFilterer creates a new log filterer instance of OutboundQueue, bound to a specific deployed contract.
func NewOutboundQueueFilterer(address common.Address, filterer bind.ContractFilterer) (*OutboundQueueFilterer, error) {
	contract, err := bindOutboundQueue(address, nil, nil, filterer)
	if err != nil {
		return nil, err
	}
	return &OutboundQueueFilterer{contract: contract}, nil
}

// bindOutboundQueue binds a generic wrapper to an already deployed contract.
func bindOutboundQueue(address common.Address, caller bind.ContractCaller, transactor bind.ContractTransactor, filterer bind.ContractFilterer) (*bind.BoundContract, error) {
	parsed, err := abi.JSON(strings.NewReader(OutboundQueueABI))
	if err != nil {
		return nil, err
	}
	return bind.NewBoundContract(address, parsed, caller, transactor, filterer), nil
}

// Call invokes the (constant) contract method with params as input values and
// sets the output to result. The result type might be a single field for simple
// returns, a slice of interfaces for anonymous returns and a struct for named
// returns.
func (_OutboundQueue *OutboundQueueRaw) Call(opts *bind.CallOpts, result *[]interface{}, method string, params ...interface{}) error {
	return _OutboundQueue.Contract.OutboundQueueCaller.contract.Call(opts, result, method, params...)
}

// Transfer initiates a plain transaction to move funds to the contract, calling
// its default method if one is available.
func (_OutboundQueue *OutboundQueueRaw) Transfer(opts *bind.TransactOpts) (*types.Transaction, error) {
	return _OutboundQueue.Contract.OutboundQueueTransactor.contract.Transfer(opts)
}

// Transact invokes the (paid) contract method with params as input values.
func (_OutboundQueue *OutboundQueueRaw) Transact(opts *bind.TransactOpts, method string, params ...interface{}) (*types.Transaction, error) {
	return _OutboundQueue.Contract.OutboundQueueTransactor.contract.Transact(opts, method, params...)
}

// Call invokes the (constant) contract method with params as input values and
// sets the output to result. The result type might be a single field for simple
// returns, a slice of interfaces for anonymous returns and a struct for named
// returns.
func (_OutboundQueue *OutboundQueueCallerRaw) Call(opts *bind.CallOpts, result *[]interface{}, method string, params ...interface{}) error {
	return _OutboundQueue.Contract.contract.Call(opts, result, method, params...)
}

// Transfer initiates a plain transaction to move funds to the contract, calling
// its default method if one is available.
func (_OutboundQueue *OutboundQueueTransactorRaw) Transfer(opts *bind.TransactOpts) (*types.Transaction, error) {
	return _OutboundQueue.Contract.contract.Transfer(opts)
}

// Transact invokes the (paid) contract method with params as input values.
func (_OutboundQueue *OutboundQueueTransactorRaw) Transact(opts *bind.TransactOpts, method string, params ...interface{}) (*types.Transaction, error) {
	return _OutboundQueue.Contract.contract.Transact(opts, method, params...)
}

// ADMINROLE is a free data retrieval call binding the contract method 0x75b238fc.
//
// Solidity: function ADMIN_ROLE() view returns(bytes32)
func (_OutboundQueue *OutboundQueueCaller) ADMINROLE(opts *bind.CallOpts) ([32]byte, error) {
	var out []interface{}
	err := _OutboundQueue.contract.Call(opts, &out, "ADMIN_ROLE")

	if err != nil {
		return *new([32]byte), err
	}

	out0 := *abi.ConvertType(out[0], new([32]byte)).(*[32]byte)

	return out0, err

}

// ADMINROLE is a free data retrieval call binding the contract method 0x75b238fc.
//
// Solidity: function ADMIN_ROLE() view returns(bytes32)
func (_OutboundQueue *OutboundQueueSession) ADMINROLE() ([32]byte, error) {
	return _OutboundQueue.Contract.ADMINROLE(&_OutboundQueue.CallOpts)
}

// ADMINROLE is a free data retrieval call binding the contract method 0x75b238fc.
//
// Solidity: function ADMIN_ROLE() view returns(bytes32)
func (_OutboundQueue *OutboundQueueCallerSession) ADMINROLE() ([32]byte, error) {
	return _OutboundQueue.Contract.ADMINROLE(&_OutboundQueue.CallOpts)
}

// DEFAULTADMINROLE is a free data retrieval call binding the contract method 0xa217fddf.
//
// Solidity: function DEFAULT_ADMIN_ROLE() view returns(bytes32)
func (_OutboundQueue *OutboundQueueCaller) DEFAULTADMINROLE(opts *bind.CallOpts) ([32]byte, error) {
	var out []interface{}
	err := _OutboundQueue.contract.Call(opts, &out, "DEFAULT_ADMIN_ROLE")

	if err != nil {
		return *new([32]byte), err
	}

	out0 := *abi.ConvertType(out[0], new([32]byte)).(*[32]byte)

	return out0, err

}

// DEFAULTADMINROLE is a free data retrieval call binding the contract method 0xa217fddf.
//
// Solidity: function DEFAULT_ADMIN_ROLE() view returns(bytes32)
func (_OutboundQueue *OutboundQueueSession) DEFAULTADMINROLE() ([32]byte, error) {
	return _OutboundQueue.Contract.DEFAULTADMINROLE(&_OutboundQueue.CallOpts)
}

// DEFAULTADMINROLE is a free data retrieval call binding the contract method 0xa217fddf.
//
// Solidity: function DEFAULT_ADMIN_ROLE() view returns(bytes32)
func (_OutboundQueue *OutboundQueueCallerSession) DEFAULTADMINROLE() ([32]byte, error) {
	return _OutboundQueue.Contract.DEFAULTADMINROLE(&_OutboundQueue.CallOpts)
}

// SUBMITROLE is a free data retrieval call binding the contract method 0xa9c0c694.
//
// Solidity: function SUBMIT_ROLE() view returns(bytes32)
func (_OutboundQueue *OutboundQueueCaller) SUBMITROLE(opts *bind.CallOpts) ([32]byte, error) {
	var out []interface{}
	err := _OutboundQueue.contract.Call(opts, &out, "SUBMIT_ROLE")

	if err != nil {
		return *new([32]byte), err
	}

	out0 := *abi.ConvertType(out[0], new([32]byte)).(*[32]byte)

	return out0, err

}

// SUBMITROLE is a free data retrieval call binding the contract method 0xa9c0c694.
//
// Solidity: function SUBMIT_ROLE() view returns(bytes32)
func (_OutboundQueue *OutboundQueueSession) SUBMITROLE() ([32]byte, error) {
	return _OutboundQueue.Contract.SUBMITROLE(&_OutboundQueue.CallOpts)
}

// SUBMITROLE is a free data retrieval call binding the contract method 0xa9c0c694.
//
// Solidity: function SUBMIT_ROLE() view returns(bytes32)
func (_OutboundQueue *OutboundQueueCallerSession) SUBMITROLE() ([32]byte, error) {
	return _OutboundQueue.Contract.SUBMITROLE(&_OutboundQueue.CallOpts)
}

// Fee is a free data retrieval call binding the contract method 0xddca3f43.
//
// Solidity: function fee() view returns(uint256)
func (_OutboundQueue *OutboundQueueCaller) Fee(opts *bind.CallOpts) (*big.Int, error) {
	var out []interface{}
	err := _OutboundQueue.contract.Call(opts, &out, "fee")

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// Fee is a free data retrieval call binding the contract method 0xddca3f43.
//
// Solidity: function fee() view returns(uint256)
func (_OutboundQueue *OutboundQueueSession) Fee() (*big.Int, error) {
	return _OutboundQueue.Contract.Fee(&_OutboundQueue.CallOpts)
}

// Fee is a free data retrieval call binding the contract method 0xddca3f43.
//
// Solidity: function fee() view returns(uint256)
func (_OutboundQueue *OutboundQueueCallerSession) Fee() (*big.Int, error) {
	return _OutboundQueue.Contract.Fee(&_OutboundQueue.CallOpts)
}

// GetRoleAdmin is a free data retrieval call binding the contract method 0x248a9ca3.
//
// Solidity: function getRoleAdmin(bytes32 role) view returns(bytes32)
func (_OutboundQueue *OutboundQueueCaller) GetRoleAdmin(opts *bind.CallOpts, role [32]byte) ([32]byte, error) {
	var out []interface{}
	err := _OutboundQueue.contract.Call(opts, &out, "getRoleAdmin", role)

	if err != nil {
		return *new([32]byte), err
	}

	out0 := *abi.ConvertType(out[0], new([32]byte)).(*[32]byte)

	return out0, err

}

// GetRoleAdmin is a free data retrieval call binding the contract method 0x248a9ca3.
//
// Solidity: function getRoleAdmin(bytes32 role) view returns(bytes32)
func (_OutboundQueue *OutboundQueueSession) GetRoleAdmin(role [32]byte) ([32]byte, error) {
	return _OutboundQueue.Contract.GetRoleAdmin(&_OutboundQueue.CallOpts, role)
}

// GetRoleAdmin is a free data retrieval call binding the contract method 0x248a9ca3.
//
// Solidity: function getRoleAdmin(bytes32 role) view returns(bytes32)
func (_OutboundQueue *OutboundQueueCallerSession) GetRoleAdmin(role [32]byte) ([32]byte, error) {
	return _OutboundQueue.Contract.GetRoleAdmin(&_OutboundQueue.CallOpts, role)
}

// HasRole is a free data retrieval call binding the contract method 0x91d14854.
//
// Solidity: function hasRole(bytes32 role, address account) view returns(bool)
func (_OutboundQueue *OutboundQueueCaller) HasRole(opts *bind.CallOpts, role [32]byte, account common.Address) (bool, error) {
	var out []interface{}
	err := _OutboundQueue.contract.Call(opts, &out, "hasRole", role, account)

	if err != nil {
		return *new(bool), err
	}

	out0 := *abi.ConvertType(out[0], new(bool)).(*bool)

	return out0, err

}

// HasRole is a free data retrieval call binding the contract method 0x91d14854.
//
// Solidity: function hasRole(bytes32 role, address account) view returns(bool)
func (_OutboundQueue *OutboundQueueSession) HasRole(role [32]byte, account common.Address) (bool, error) {
	return _OutboundQueue.Contract.HasRole(&_OutboundQueue.CallOpts, role, account)
}

// HasRole is a free data retrieval call binding the contract method 0x91d14854.
//
// Solidity: function hasRole(bytes32 role, address account) view returns(bool)
func (_OutboundQueue *OutboundQueueCallerSession) HasRole(role [32]byte, account common.Address) (bool, error) {
	return _OutboundQueue.Contract.HasRole(&_OutboundQueue.CallOpts, role, account)
}

// Nonce is a free data retrieval call binding the contract method 0x4e765004.
//
// Solidity: function nonce(bytes ) view returns(uint64)
func (_OutboundQueue *OutboundQueueCaller) Nonce(opts *bind.CallOpts, arg0 []byte) (uint64, error) {
	var out []interface{}
	err := _OutboundQueue.contract.Call(opts, &out, "nonce", arg0)

	if err != nil {
		return *new(uint64), err
	}

	out0 := *abi.ConvertType(out[0], new(uint64)).(*uint64)

	return out0, err

}

// Nonce is a free data retrieval call binding the contract method 0x4e765004.
//
// Solidity: function nonce(bytes ) view returns(uint64)
func (_OutboundQueue *OutboundQueueSession) Nonce(arg0 []byte) (uint64, error) {
	return _OutboundQueue.Contract.Nonce(&_OutboundQueue.CallOpts, arg0)
}

// Nonce is a free data retrieval call binding the contract method 0x4e765004.
//
// Solidity: function nonce(bytes ) view returns(uint64)
func (_OutboundQueue *OutboundQueueCallerSession) Nonce(arg0 []byte) (uint64, error) {
	return _OutboundQueue.Contract.Nonce(&_OutboundQueue.CallOpts, arg0)
}

// SupportsInterface is a free data retrieval call binding the contract method 0x01ffc9a7.
//
// Solidity: function supportsInterface(bytes4 interfaceId) view returns(bool)
func (_OutboundQueue *OutboundQueueCaller) SupportsInterface(opts *bind.CallOpts, interfaceId [4]byte) (bool, error) {
	var out []interface{}
	err := _OutboundQueue.contract.Call(opts, &out, "supportsInterface", interfaceId)

	if err != nil {
		return *new(bool), err
	}

	out0 := *abi.ConvertType(out[0], new(bool)).(*bool)

	return out0, err

}

// SupportsInterface is a free data retrieval call binding the contract method 0x01ffc9a7.
//
// Solidity: function supportsInterface(bytes4 interfaceId) view returns(bool)
func (_OutboundQueue *OutboundQueueSession) SupportsInterface(interfaceId [4]byte) (bool, error) {
	return _OutboundQueue.Contract.SupportsInterface(&_OutboundQueue.CallOpts, interfaceId)
}

// SupportsInterface is a free data retrieval call binding the contract method 0x01ffc9a7.
//
// Solidity: function supportsInterface(bytes4 interfaceId) view returns(bool)
func (_OutboundQueue *OutboundQueueCallerSession) SupportsInterface(interfaceId [4]byte) (bool, error) {
	return _OutboundQueue.Contract.SupportsInterface(&_OutboundQueue.CallOpts, interfaceId)
}

// Vault is a free data retrieval call binding the contract method 0xfbfa77cf.
//
// Solidity: function vault() view returns(address)
func (_OutboundQueue *OutboundQueueCaller) Vault(opts *bind.CallOpts) (common.Address, error) {
	var out []interface{}
	err := _OutboundQueue.contract.Call(opts, &out, "vault")

	if err != nil {
		return *new(common.Address), err
	}

	out0 := *abi.ConvertType(out[0], new(common.Address)).(*common.Address)

	return out0, err

}

// Vault is a free data retrieval call binding the contract method 0xfbfa77cf.
//
// Solidity: function vault() view returns(address)
func (_OutboundQueue *OutboundQueueSession) Vault() (common.Address, error) {
	return _OutboundQueue.Contract.Vault(&_OutboundQueue.CallOpts)
}

// Vault is a free data retrieval call binding the contract method 0xfbfa77cf.
//
// Solidity: function vault() view returns(address)
func (_OutboundQueue *OutboundQueueCallerSession) Vault() (common.Address, error) {
	return _OutboundQueue.Contract.Vault(&_OutboundQueue.CallOpts)
}

// GrantRole is a paid mutator transaction binding the contract method 0x2f2ff15d.
//
// Solidity: function grantRole(bytes32 role, address account) returns()
func (_OutboundQueue *OutboundQueueTransactor) GrantRole(opts *bind.TransactOpts, role [32]byte, account common.Address) (*types.Transaction, error) {
	return _OutboundQueue.contract.Transact(opts, "grantRole", role, account)
}

// GrantRole is a paid mutator transaction binding the contract method 0x2f2ff15d.
//
// Solidity: function grantRole(bytes32 role, address account) returns()
func (_OutboundQueue *OutboundQueueSession) GrantRole(role [32]byte, account common.Address) (*types.Transaction, error) {
	return _OutboundQueue.Contract.GrantRole(&_OutboundQueue.TransactOpts, role, account)
}

// GrantRole is a paid mutator transaction binding the contract method 0x2f2ff15d.
//
// Solidity: function grantRole(bytes32 role, address account) returns()
func (_OutboundQueue *OutboundQueueTransactorSession) GrantRole(role [32]byte, account common.Address) (*types.Transaction, error) {
	return _OutboundQueue.Contract.GrantRole(&_OutboundQueue.TransactOpts, role, account)
}

// RenounceRole is a paid mutator transaction binding the contract method 0x36568abe.
//
// Solidity: function renounceRole(bytes32 role, address account) returns()
func (_OutboundQueue *OutboundQueueTransactor) RenounceRole(opts *bind.TransactOpts, role [32]byte, account common.Address) (*types.Transaction, error) {
	return _OutboundQueue.contract.Transact(opts, "renounceRole", role, account)
}

// RenounceRole is a paid mutator transaction binding the contract method 0x36568abe.
//
// Solidity: function renounceRole(bytes32 role, address account) returns()
func (_OutboundQueue *OutboundQueueSession) RenounceRole(role [32]byte, account common.Address) (*types.Transaction, error) {
	return _OutboundQueue.Contract.RenounceRole(&_OutboundQueue.TransactOpts, role, account)
}

// RenounceRole is a paid mutator transaction binding the contract method 0x36568abe.
//
// Solidity: function renounceRole(bytes32 role, address account) returns()
func (_OutboundQueue *OutboundQueueTransactorSession) RenounceRole(role [32]byte, account common.Address) (*types.Transaction, error) {
	return _OutboundQueue.Contract.RenounceRole(&_OutboundQueue.TransactOpts, role, account)
}

// RevokeRole is a paid mutator transaction binding the contract method 0xd547741f.
//
// Solidity: function revokeRole(bytes32 role, address account) returns()
func (_OutboundQueue *OutboundQueueTransactor) RevokeRole(opts *bind.TransactOpts, role [32]byte, account common.Address) (*types.Transaction, error) {
	return _OutboundQueue.contract.Transact(opts, "revokeRole", role, account)
}

// RevokeRole is a paid mutator transaction binding the contract method 0xd547741f.
//
// Solidity: function revokeRole(bytes32 role, address account) returns()
func (_OutboundQueue *OutboundQueueSession) RevokeRole(role [32]byte, account common.Address) (*types.Transaction, error) {
	return _OutboundQueue.Contract.RevokeRole(&_OutboundQueue.TransactOpts, role, account)
}

// RevokeRole is a paid mutator transaction binding the contract method 0xd547741f.
//
// Solidity: function revokeRole(bytes32 role, address account) returns()
func (_OutboundQueue *OutboundQueueTransactorSession) RevokeRole(role [32]byte, account common.Address) (*types.Transaction, error) {
	return _OutboundQueue.Contract.RevokeRole(&_OutboundQueue.TransactOpts, role, account)
}

// Submit is a paid mutator transaction binding the contract method 0x39b973ad.
//
// Solidity: function submit(bytes dest, bytes payload) payable returns()
func (_OutboundQueue *OutboundQueueTransactor) Submit(opts *bind.TransactOpts, dest []byte, payload []byte) (*types.Transaction, error) {
	return _OutboundQueue.contract.Transact(opts, "submit", dest, payload)
}

// Submit is a paid mutator transaction binding the contract method 0x39b973ad.
//
// Solidity: function submit(bytes dest, bytes payload) payable returns()
func (_OutboundQueue *OutboundQueueSession) Submit(dest []byte, payload []byte) (*types.Transaction, error) {
	return _OutboundQueue.Contract.Submit(&_OutboundQueue.TransactOpts, dest, payload)
}

// Submit is a paid mutator transaction binding the contract method 0x39b973ad.
//
// Solidity: function submit(bytes dest, bytes payload) payable returns()
func (_OutboundQueue *OutboundQueueTransactorSession) Submit(dest []byte, payload []byte) (*types.Transaction, error) {
	return _OutboundQueue.Contract.Submit(&_OutboundQueue.TransactOpts, dest, payload)
}

// UpdateFee is a paid mutator transaction binding the contract method 0x9012c4a8.
//
// Solidity: function updateFee(uint256 newFee) returns()
func (_OutboundQueue *OutboundQueueTransactor) UpdateFee(opts *bind.TransactOpts, newFee *big.Int) (*types.Transaction, error) {
	return _OutboundQueue.contract.Transact(opts, "updateFee", newFee)
}

// UpdateFee is a paid mutator transaction binding the contract method 0x9012c4a8.
//
// Solidity: function updateFee(uint256 newFee) returns()
func (_OutboundQueue *OutboundQueueSession) UpdateFee(newFee *big.Int) (*types.Transaction, error) {
	return _OutboundQueue.Contract.UpdateFee(&_OutboundQueue.TransactOpts, newFee)
}

// UpdateFee is a paid mutator transaction binding the contract method 0x9012c4a8.
//
// Solidity: function updateFee(uint256 newFee) returns()
func (_OutboundQueue *OutboundQueueTransactorSession) UpdateFee(newFee *big.Int) (*types.Transaction, error) {
	return _OutboundQueue.Contract.UpdateFee(&_OutboundQueue.TransactOpts, newFee)
}

// OutboundQueueFeeUpdatedIterator is returned from FilterFeeUpdated and is used to iterate over the raw logs and unpacked data for FeeUpdated events raised by the OutboundQueue contract.
type OutboundQueueFeeUpdatedIterator struct {
	Event *OutboundQueueFeeUpdated // Event containing the contract specifics and raw log

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
func (it *OutboundQueueFeeUpdatedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(OutboundQueueFeeUpdated)
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
		it.Event = new(OutboundQueueFeeUpdated)
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
func (it *OutboundQueueFeeUpdatedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *OutboundQueueFeeUpdatedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// OutboundQueueFeeUpdated represents a FeeUpdated event raised by the OutboundQueue contract.
type OutboundQueueFeeUpdated struct {
	Fee *big.Int
	Raw types.Log // Blockchain specific contextual infos
}

// FilterFeeUpdated is a free log retrieval operation binding the contract event 0x8c4d35e54a3f2ef1134138fd8ea3daee6a3c89e10d2665996babdf70261e2c76.
//
// Solidity: event FeeUpdated(uint256 fee)
func (_OutboundQueue *OutboundQueueFilterer) FilterFeeUpdated(opts *bind.FilterOpts) (*OutboundQueueFeeUpdatedIterator, error) {

	logs, sub, err := _OutboundQueue.contract.FilterLogs(opts, "FeeUpdated")
	if err != nil {
		return nil, err
	}
	return &OutboundQueueFeeUpdatedIterator{contract: _OutboundQueue.contract, event: "FeeUpdated", logs: logs, sub: sub}, nil
}

// WatchFeeUpdated is a free log subscription operation binding the contract event 0x8c4d35e54a3f2ef1134138fd8ea3daee6a3c89e10d2665996babdf70261e2c76.
//
// Solidity: event FeeUpdated(uint256 fee)
func (_OutboundQueue *OutboundQueueFilterer) WatchFeeUpdated(opts *bind.WatchOpts, sink chan<- *OutboundQueueFeeUpdated) (event.Subscription, error) {

	logs, sub, err := _OutboundQueue.contract.WatchLogs(opts, "FeeUpdated")
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(OutboundQueueFeeUpdated)
				if err := _OutboundQueue.contract.UnpackLog(event, "FeeUpdated", log); err != nil {
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

// ParseFeeUpdated is a log parse operation binding the contract event 0x8c4d35e54a3f2ef1134138fd8ea3daee6a3c89e10d2665996babdf70261e2c76.
//
// Solidity: event FeeUpdated(uint256 fee)
func (_OutboundQueue *OutboundQueueFilterer) ParseFeeUpdated(log types.Log) (*OutboundQueueFeeUpdated, error) {
	event := new(OutboundQueueFeeUpdated)
	if err := _OutboundQueue.contract.UnpackLog(event, "FeeUpdated", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// OutboundQueueMessageIterator is returned from FilterMessage and is used to iterate over the raw logs and unpacked data for Message events raised by the OutboundQueue contract.
type OutboundQueueMessageIterator struct {
	Event *OutboundQueueMessage // Event containing the contract specifics and raw log

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
func (it *OutboundQueueMessageIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(OutboundQueueMessage)
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
		it.Event = new(OutboundQueueMessage)
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
func (it *OutboundQueueMessageIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *OutboundQueueMessageIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// OutboundQueueMessage represents a Message event raised by the OutboundQueue contract.
type OutboundQueueMessage struct {
	Dest    common.Hash
	Nonce   uint64
	Payload []byte
	Raw     types.Log // Blockchain specific contextual infos
}

// FilterMessage is a free log retrieval operation binding the contract event 0xb68e760461a2cd8661cea966252574a5355ad3ed096afdd6572c65ef50bb7cd7.
//
// Solidity: event Message(bytes indexed dest, uint64 nonce, bytes payload)
func (_OutboundQueue *OutboundQueueFilterer) FilterMessage(opts *bind.FilterOpts, dest [][]byte) (*OutboundQueueMessageIterator, error) {

	var destRule []interface{}
	for _, destItem := range dest {
		destRule = append(destRule, destItem)
	}

	logs, sub, err := _OutboundQueue.contract.FilterLogs(opts, "Message", destRule)
	if err != nil {
		return nil, err
	}
	return &OutboundQueueMessageIterator{contract: _OutboundQueue.contract, event: "Message", logs: logs, sub: sub}, nil
}

// WatchMessage is a free log subscription operation binding the contract event 0xb68e760461a2cd8661cea966252574a5355ad3ed096afdd6572c65ef50bb7cd7.
//
// Solidity: event Message(bytes indexed dest, uint64 nonce, bytes payload)
func (_OutboundQueue *OutboundQueueFilterer) WatchMessage(opts *bind.WatchOpts, sink chan<- *OutboundQueueMessage, dest [][]byte) (event.Subscription, error) {

	var destRule []interface{}
	for _, destItem := range dest {
		destRule = append(destRule, destItem)
	}

	logs, sub, err := _OutboundQueue.contract.WatchLogs(opts, "Message", destRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(OutboundQueueMessage)
				if err := _OutboundQueue.contract.UnpackLog(event, "Message", log); err != nil {
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

// ParseMessage is a log parse operation binding the contract event 0xb68e760461a2cd8661cea966252574a5355ad3ed096afdd6572c65ef50bb7cd7.
//
// Solidity: event Message(bytes indexed dest, uint64 nonce, bytes payload)
func (_OutboundQueue *OutboundQueueFilterer) ParseMessage(log types.Log) (*OutboundQueueMessage, error) {
	event := new(OutboundQueueMessage)
	if err := _OutboundQueue.contract.UnpackLog(event, "Message", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// OutboundQueueRoleAdminChangedIterator is returned from FilterRoleAdminChanged and is used to iterate over the raw logs and unpacked data for RoleAdminChanged events raised by the OutboundQueue contract.
type OutboundQueueRoleAdminChangedIterator struct {
	Event *OutboundQueueRoleAdminChanged // Event containing the contract specifics and raw log

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
func (it *OutboundQueueRoleAdminChangedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(OutboundQueueRoleAdminChanged)
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
		it.Event = new(OutboundQueueRoleAdminChanged)
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
func (it *OutboundQueueRoleAdminChangedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *OutboundQueueRoleAdminChangedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// OutboundQueueRoleAdminChanged represents a RoleAdminChanged event raised by the OutboundQueue contract.
type OutboundQueueRoleAdminChanged struct {
	Role              [32]byte
	PreviousAdminRole [32]byte
	NewAdminRole      [32]byte
	Raw               types.Log // Blockchain specific contextual infos
}

// FilterRoleAdminChanged is a free log retrieval operation binding the contract event 0xbd79b86ffe0ab8e8776151514217cd7cacd52c909f66475c3af44e129f0b00ff.
//
// Solidity: event RoleAdminChanged(bytes32 indexed role, bytes32 indexed previousAdminRole, bytes32 indexed newAdminRole)
func (_OutboundQueue *OutboundQueueFilterer) FilterRoleAdminChanged(opts *bind.FilterOpts, role [][32]byte, previousAdminRole [][32]byte, newAdminRole [][32]byte) (*OutboundQueueRoleAdminChangedIterator, error) {

	var roleRule []interface{}
	for _, roleItem := range role {
		roleRule = append(roleRule, roleItem)
	}
	var previousAdminRoleRule []interface{}
	for _, previousAdminRoleItem := range previousAdminRole {
		previousAdminRoleRule = append(previousAdminRoleRule, previousAdminRoleItem)
	}
	var newAdminRoleRule []interface{}
	for _, newAdminRoleItem := range newAdminRole {
		newAdminRoleRule = append(newAdminRoleRule, newAdminRoleItem)
	}

	logs, sub, err := _OutboundQueue.contract.FilterLogs(opts, "RoleAdminChanged", roleRule, previousAdminRoleRule, newAdminRoleRule)
	if err != nil {
		return nil, err
	}
	return &OutboundQueueRoleAdminChangedIterator{contract: _OutboundQueue.contract, event: "RoleAdminChanged", logs: logs, sub: sub}, nil
}

// WatchRoleAdminChanged is a free log subscription operation binding the contract event 0xbd79b86ffe0ab8e8776151514217cd7cacd52c909f66475c3af44e129f0b00ff.
//
// Solidity: event RoleAdminChanged(bytes32 indexed role, bytes32 indexed previousAdminRole, bytes32 indexed newAdminRole)
func (_OutboundQueue *OutboundQueueFilterer) WatchRoleAdminChanged(opts *bind.WatchOpts, sink chan<- *OutboundQueueRoleAdminChanged, role [][32]byte, previousAdminRole [][32]byte, newAdminRole [][32]byte) (event.Subscription, error) {

	var roleRule []interface{}
	for _, roleItem := range role {
		roleRule = append(roleRule, roleItem)
	}
	var previousAdminRoleRule []interface{}
	for _, previousAdminRoleItem := range previousAdminRole {
		previousAdminRoleRule = append(previousAdminRoleRule, previousAdminRoleItem)
	}
	var newAdminRoleRule []interface{}
	for _, newAdminRoleItem := range newAdminRole {
		newAdminRoleRule = append(newAdminRoleRule, newAdminRoleItem)
	}

	logs, sub, err := _OutboundQueue.contract.WatchLogs(opts, "RoleAdminChanged", roleRule, previousAdminRoleRule, newAdminRoleRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(OutboundQueueRoleAdminChanged)
				if err := _OutboundQueue.contract.UnpackLog(event, "RoleAdminChanged", log); err != nil {
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

// ParseRoleAdminChanged is a log parse operation binding the contract event 0xbd79b86ffe0ab8e8776151514217cd7cacd52c909f66475c3af44e129f0b00ff.
//
// Solidity: event RoleAdminChanged(bytes32 indexed role, bytes32 indexed previousAdminRole, bytes32 indexed newAdminRole)
func (_OutboundQueue *OutboundQueueFilterer) ParseRoleAdminChanged(log types.Log) (*OutboundQueueRoleAdminChanged, error) {
	event := new(OutboundQueueRoleAdminChanged)
	if err := _OutboundQueue.contract.UnpackLog(event, "RoleAdminChanged", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// OutboundQueueRoleGrantedIterator is returned from FilterRoleGranted and is used to iterate over the raw logs and unpacked data for RoleGranted events raised by the OutboundQueue contract.
type OutboundQueueRoleGrantedIterator struct {
	Event *OutboundQueueRoleGranted // Event containing the contract specifics and raw log

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
func (it *OutboundQueueRoleGrantedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(OutboundQueueRoleGranted)
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
		it.Event = new(OutboundQueueRoleGranted)
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
func (it *OutboundQueueRoleGrantedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *OutboundQueueRoleGrantedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// OutboundQueueRoleGranted represents a RoleGranted event raised by the OutboundQueue contract.
type OutboundQueueRoleGranted struct {
	Role    [32]byte
	Account common.Address
	Sender  common.Address
	Raw     types.Log // Blockchain specific contextual infos
}

// FilterRoleGranted is a free log retrieval operation binding the contract event 0x2f8788117e7eff1d82e926ec794901d17c78024a50270940304540a733656f0d.
//
// Solidity: event RoleGranted(bytes32 indexed role, address indexed account, address indexed sender)
func (_OutboundQueue *OutboundQueueFilterer) FilterRoleGranted(opts *bind.FilterOpts, role [][32]byte, account []common.Address, sender []common.Address) (*OutboundQueueRoleGrantedIterator, error) {

	var roleRule []interface{}
	for _, roleItem := range role {
		roleRule = append(roleRule, roleItem)
	}
	var accountRule []interface{}
	for _, accountItem := range account {
		accountRule = append(accountRule, accountItem)
	}
	var senderRule []interface{}
	for _, senderItem := range sender {
		senderRule = append(senderRule, senderItem)
	}

	logs, sub, err := _OutboundQueue.contract.FilterLogs(opts, "RoleGranted", roleRule, accountRule, senderRule)
	if err != nil {
		return nil, err
	}
	return &OutboundQueueRoleGrantedIterator{contract: _OutboundQueue.contract, event: "RoleGranted", logs: logs, sub: sub}, nil
}

// WatchRoleGranted is a free log subscription operation binding the contract event 0x2f8788117e7eff1d82e926ec794901d17c78024a50270940304540a733656f0d.
//
// Solidity: event RoleGranted(bytes32 indexed role, address indexed account, address indexed sender)
func (_OutboundQueue *OutboundQueueFilterer) WatchRoleGranted(opts *bind.WatchOpts, sink chan<- *OutboundQueueRoleGranted, role [][32]byte, account []common.Address, sender []common.Address) (event.Subscription, error) {

	var roleRule []interface{}
	for _, roleItem := range role {
		roleRule = append(roleRule, roleItem)
	}
	var accountRule []interface{}
	for _, accountItem := range account {
		accountRule = append(accountRule, accountItem)
	}
	var senderRule []interface{}
	for _, senderItem := range sender {
		senderRule = append(senderRule, senderItem)
	}

	logs, sub, err := _OutboundQueue.contract.WatchLogs(opts, "RoleGranted", roleRule, accountRule, senderRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(OutboundQueueRoleGranted)
				if err := _OutboundQueue.contract.UnpackLog(event, "RoleGranted", log); err != nil {
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

// ParseRoleGranted is a log parse operation binding the contract event 0x2f8788117e7eff1d82e926ec794901d17c78024a50270940304540a733656f0d.
//
// Solidity: event RoleGranted(bytes32 indexed role, address indexed account, address indexed sender)
func (_OutboundQueue *OutboundQueueFilterer) ParseRoleGranted(log types.Log) (*OutboundQueueRoleGranted, error) {
	event := new(OutboundQueueRoleGranted)
	if err := _OutboundQueue.contract.UnpackLog(event, "RoleGranted", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// OutboundQueueRoleRevokedIterator is returned from FilterRoleRevoked and is used to iterate over the raw logs and unpacked data for RoleRevoked events raised by the OutboundQueue contract.
type OutboundQueueRoleRevokedIterator struct {
	Event *OutboundQueueRoleRevoked // Event containing the contract specifics and raw log

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
func (it *OutboundQueueRoleRevokedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(OutboundQueueRoleRevoked)
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
		it.Event = new(OutboundQueueRoleRevoked)
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
func (it *OutboundQueueRoleRevokedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *OutboundQueueRoleRevokedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// OutboundQueueRoleRevoked represents a RoleRevoked event raised by the OutboundQueue contract.
type OutboundQueueRoleRevoked struct {
	Role    [32]byte
	Account common.Address
	Sender  common.Address
	Raw     types.Log // Blockchain specific contextual infos
}

// FilterRoleRevoked is a free log retrieval operation binding the contract event 0xf6391f5c32d9c69d2a47ea670b442974b53935d1edc7fd64eb21e047a839171b.
//
// Solidity: event RoleRevoked(bytes32 indexed role, address indexed account, address indexed sender)
func (_OutboundQueue *OutboundQueueFilterer) FilterRoleRevoked(opts *bind.FilterOpts, role [][32]byte, account []common.Address, sender []common.Address) (*OutboundQueueRoleRevokedIterator, error) {

	var roleRule []interface{}
	for _, roleItem := range role {
		roleRule = append(roleRule, roleItem)
	}
	var accountRule []interface{}
	for _, accountItem := range account {
		accountRule = append(accountRule, accountItem)
	}
	var senderRule []interface{}
	for _, senderItem := range sender {
		senderRule = append(senderRule, senderItem)
	}

	logs, sub, err := _OutboundQueue.contract.FilterLogs(opts, "RoleRevoked", roleRule, accountRule, senderRule)
	if err != nil {
		return nil, err
	}
	return &OutboundQueueRoleRevokedIterator{contract: _OutboundQueue.contract, event: "RoleRevoked", logs: logs, sub: sub}, nil
}

// WatchRoleRevoked is a free log subscription operation binding the contract event 0xf6391f5c32d9c69d2a47ea670b442974b53935d1edc7fd64eb21e047a839171b.
//
// Solidity: event RoleRevoked(bytes32 indexed role, address indexed account, address indexed sender)
func (_OutboundQueue *OutboundQueueFilterer) WatchRoleRevoked(opts *bind.WatchOpts, sink chan<- *OutboundQueueRoleRevoked, role [][32]byte, account []common.Address, sender []common.Address) (event.Subscription, error) {

	var roleRule []interface{}
	for _, roleItem := range role {
		roleRule = append(roleRule, roleItem)
	}
	var accountRule []interface{}
	for _, accountItem := range account {
		accountRule = append(accountRule, accountItem)
	}
	var senderRule []interface{}
	for _, senderItem := range sender {
		senderRule = append(senderRule, senderItem)
	}

	logs, sub, err := _OutboundQueue.contract.WatchLogs(opts, "RoleRevoked", roleRule, accountRule, senderRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(OutboundQueueRoleRevoked)
				if err := _OutboundQueue.contract.UnpackLog(event, "RoleRevoked", log); err != nil {
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

// ParseRoleRevoked is a log parse operation binding the contract event 0xf6391f5c32d9c69d2a47ea670b442974b53935d1edc7fd64eb21e047a839171b.
//
// Solidity: event RoleRevoked(bytes32 indexed role, address indexed account, address indexed sender)
func (_OutboundQueue *OutboundQueueFilterer) ParseRoleRevoked(log types.Log) (*OutboundQueueRoleRevoked, error) {
	event := new(OutboundQueueRoleRevoked)
	if err := _OutboundQueue.contract.UnpackLog(event, "RoleRevoked", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}
