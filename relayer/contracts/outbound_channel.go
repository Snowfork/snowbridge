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

// OutboundChannelMetaData contains all meta data concerning the OutboundChannel contract.
var OutboundChannelMetaData = &bind.MetaData{
	ABI: "[{\"inputs\":[{\"internalType\":\"contractIVault\",\"name\":\"_vault\",\"type\":\"address\"},{\"internalType\":\"uint256\",\"name\":\"_fee\",\"type\":\"uint256\"}],\"stateMutability\":\"nonpayable\",\"type\":\"constructor\"},{\"inputs\":[],\"name\":\"FeePaymentToLow\",\"type\":\"error\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":false,\"internalType\":\"uint256\",\"name\":\"fee\",\"type\":\"uint256\"}],\"name\":\"FeeUpdated\",\"type\":\"event\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":true,\"internalType\":\"bytes\",\"name\":\"dest\",\"type\":\"bytes\"},{\"indexed\":false,\"internalType\":\"uint64\",\"name\":\"nonce\",\"type\":\"uint64\"},{\"indexed\":false,\"internalType\":\"bytes\",\"name\":\"payload\",\"type\":\"bytes\"}],\"name\":\"Message\",\"type\":\"event\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":true,\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\"},{\"indexed\":true,\"internalType\":\"bytes32\",\"name\":\"previousAdminRole\",\"type\":\"bytes32\"},{\"indexed\":true,\"internalType\":\"bytes32\",\"name\":\"newAdminRole\",\"type\":\"bytes32\"}],\"name\":\"RoleAdminChanged\",\"type\":\"event\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":true,\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\"},{\"indexed\":true,\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\"},{\"indexed\":true,\"internalType\":\"address\",\"name\":\"sender\",\"type\":\"address\"}],\"name\":\"RoleGranted\",\"type\":\"event\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":true,\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\"},{\"indexed\":true,\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\"},{\"indexed\":true,\"internalType\":\"address\",\"name\":\"sender\",\"type\":\"address\"}],\"name\":\"RoleRevoked\",\"type\":\"event\"},{\"inputs\":[],\"name\":\"ADMIN_ROLE\",\"outputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"DEFAULT_ADMIN_ROLE\",\"outputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"SUBMIT_ROLE\",\"outputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"fee\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\"}],\"name\":\"getRoleAdmin\",\"outputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\"},{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\"}],\"name\":\"grantRole\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\"},{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\"}],\"name\":\"hasRole\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"bytes\",\"name\":\"\",\"type\":\"bytes\"}],\"name\":\"nonce\",\"outputs\":[{\"internalType\":\"uint64\",\"name\":\"\",\"type\":\"uint64\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\"},{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\"}],\"name\":\"renounceRole\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\"},{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\"}],\"name\":\"revokeRole\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"bytes\",\"name\":\"dest\",\"type\":\"bytes\"},{\"internalType\":\"bytes\",\"name\":\"payload\",\"type\":\"bytes\"}],\"name\":\"submit\",\"outputs\":[],\"stateMutability\":\"payable\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"bytes4\",\"name\":\"interfaceId\",\"type\":\"bytes4\"}],\"name\":\"supportsInterface\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"newFee\",\"type\":\"uint256\"}],\"name\":\"updateFee\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"vault\",\"outputs\":[{\"internalType\":\"contractIVault\",\"name\":\"\",\"type\":\"address\"}],\"stateMutability\":\"view\",\"type\":\"function\"}]",
}

// OutboundChannelABI is the input ABI used to generate the binding from.
// Deprecated: Use OutboundChannelMetaData.ABI instead.
var OutboundChannelABI = OutboundChannelMetaData.ABI

// OutboundChannel is an auto generated Go binding around an Ethereum contract.
type OutboundChannel struct {
	OutboundChannelCaller     // Read-only binding to the contract
	OutboundChannelTransactor // Write-only binding to the contract
	OutboundChannelFilterer   // Log filterer for contract events
}

// OutboundChannelCaller is an auto generated read-only Go binding around an Ethereum contract.
type OutboundChannelCaller struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// OutboundChannelTransactor is an auto generated write-only Go binding around an Ethereum contract.
type OutboundChannelTransactor struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// OutboundChannelFilterer is an auto generated log filtering Go binding around an Ethereum contract events.
type OutboundChannelFilterer struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// OutboundChannelSession is an auto generated Go binding around an Ethereum contract,
// with pre-set call and transact options.
type OutboundChannelSession struct {
	Contract     *OutboundChannel  // Generic contract binding to set the session for
	CallOpts     bind.CallOpts     // Call options to use throughout this session
	TransactOpts bind.TransactOpts // Transaction auth options to use throughout this session
}

// OutboundChannelCallerSession is an auto generated read-only Go binding around an Ethereum contract,
// with pre-set call options.
type OutboundChannelCallerSession struct {
	Contract *OutboundChannelCaller // Generic contract caller binding to set the session for
	CallOpts bind.CallOpts          // Call options to use throughout this session
}

// OutboundChannelTransactorSession is an auto generated write-only Go binding around an Ethereum contract,
// with pre-set transact options.
type OutboundChannelTransactorSession struct {
	Contract     *OutboundChannelTransactor // Generic contract transactor binding to set the session for
	TransactOpts bind.TransactOpts          // Transaction auth options to use throughout this session
}

// OutboundChannelRaw is an auto generated low-level Go binding around an Ethereum contract.
type OutboundChannelRaw struct {
	Contract *OutboundChannel // Generic contract binding to access the raw methods on
}

// OutboundChannelCallerRaw is an auto generated low-level read-only Go binding around an Ethereum contract.
type OutboundChannelCallerRaw struct {
	Contract *OutboundChannelCaller // Generic read-only contract binding to access the raw methods on
}

// OutboundChannelTransactorRaw is an auto generated low-level write-only Go binding around an Ethereum contract.
type OutboundChannelTransactorRaw struct {
	Contract *OutboundChannelTransactor // Generic write-only contract binding to access the raw methods on
}

// NewOutboundChannel creates a new instance of OutboundChannel, bound to a specific deployed contract.
func NewOutboundChannel(address common.Address, backend bind.ContractBackend) (*OutboundChannel, error) {
	contract, err := bindOutboundChannel(address, backend, backend, backend)
	if err != nil {
		return nil, err
	}
	return &OutboundChannel{OutboundChannelCaller: OutboundChannelCaller{contract: contract}, OutboundChannelTransactor: OutboundChannelTransactor{contract: contract}, OutboundChannelFilterer: OutboundChannelFilterer{contract: contract}}, nil
}

// NewOutboundChannelCaller creates a new read-only instance of OutboundChannel, bound to a specific deployed contract.
func NewOutboundChannelCaller(address common.Address, caller bind.ContractCaller) (*OutboundChannelCaller, error) {
	contract, err := bindOutboundChannel(address, caller, nil, nil)
	if err != nil {
		return nil, err
	}
	return &OutboundChannelCaller{contract: contract}, nil
}

// NewOutboundChannelTransactor creates a new write-only instance of OutboundChannel, bound to a specific deployed contract.
func NewOutboundChannelTransactor(address common.Address, transactor bind.ContractTransactor) (*OutboundChannelTransactor, error) {
	contract, err := bindOutboundChannel(address, nil, transactor, nil)
	if err != nil {
		return nil, err
	}
	return &OutboundChannelTransactor{contract: contract}, nil
}

// NewOutboundChannelFilterer creates a new log filterer instance of OutboundChannel, bound to a specific deployed contract.
func NewOutboundChannelFilterer(address common.Address, filterer bind.ContractFilterer) (*OutboundChannelFilterer, error) {
	contract, err := bindOutboundChannel(address, nil, nil, filterer)
	if err != nil {
		return nil, err
	}
	return &OutboundChannelFilterer{contract: contract}, nil
}

// bindOutboundChannel binds a generic wrapper to an already deployed contract.
func bindOutboundChannel(address common.Address, caller bind.ContractCaller, transactor bind.ContractTransactor, filterer bind.ContractFilterer) (*bind.BoundContract, error) {
	parsed, err := abi.JSON(strings.NewReader(OutboundChannelABI))
	if err != nil {
		return nil, err
	}
	return bind.NewBoundContract(address, parsed, caller, transactor, filterer), nil
}

// Call invokes the (constant) contract method with params as input values and
// sets the output to result. The result type might be a single field for simple
// returns, a slice of interfaces for anonymous returns and a struct for named
// returns.
func (_OutboundChannel *OutboundChannelRaw) Call(opts *bind.CallOpts, result *[]interface{}, method string, params ...interface{}) error {
	return _OutboundChannel.Contract.OutboundChannelCaller.contract.Call(opts, result, method, params...)
}

// Transfer initiates a plain transaction to move funds to the contract, calling
// its default method if one is available.
func (_OutboundChannel *OutboundChannelRaw) Transfer(opts *bind.TransactOpts) (*types.Transaction, error) {
	return _OutboundChannel.Contract.OutboundChannelTransactor.contract.Transfer(opts)
}

// Transact invokes the (paid) contract method with params as input values.
func (_OutboundChannel *OutboundChannelRaw) Transact(opts *bind.TransactOpts, method string, params ...interface{}) (*types.Transaction, error) {
	return _OutboundChannel.Contract.OutboundChannelTransactor.contract.Transact(opts, method, params...)
}

// Call invokes the (constant) contract method with params as input values and
// sets the output to result. The result type might be a single field for simple
// returns, a slice of interfaces for anonymous returns and a struct for named
// returns.
func (_OutboundChannel *OutboundChannelCallerRaw) Call(opts *bind.CallOpts, result *[]interface{}, method string, params ...interface{}) error {
	return _OutboundChannel.Contract.contract.Call(opts, result, method, params...)
}

// Transfer initiates a plain transaction to move funds to the contract, calling
// its default method if one is available.
func (_OutboundChannel *OutboundChannelTransactorRaw) Transfer(opts *bind.TransactOpts) (*types.Transaction, error) {
	return _OutboundChannel.Contract.contract.Transfer(opts)
}

// Transact invokes the (paid) contract method with params as input values.
func (_OutboundChannel *OutboundChannelTransactorRaw) Transact(opts *bind.TransactOpts, method string, params ...interface{}) (*types.Transaction, error) {
	return _OutboundChannel.Contract.contract.Transact(opts, method, params...)
}

// ADMINROLE is a free data retrieval call binding the contract method 0x75b238fc.
//
// Solidity: function ADMIN_ROLE() view returns(bytes32)
func (_OutboundChannel *OutboundChannelCaller) ADMINROLE(opts *bind.CallOpts) ([32]byte, error) {
	var out []interface{}
	err := _OutboundChannel.contract.Call(opts, &out, "ADMIN_ROLE")

	if err != nil {
		return *new([32]byte), err
	}

	out0 := *abi.ConvertType(out[0], new([32]byte)).(*[32]byte)

	return out0, err

}

// ADMINROLE is a free data retrieval call binding the contract method 0x75b238fc.
//
// Solidity: function ADMIN_ROLE() view returns(bytes32)
func (_OutboundChannel *OutboundChannelSession) ADMINROLE() ([32]byte, error) {
	return _OutboundChannel.Contract.ADMINROLE(&_OutboundChannel.CallOpts)
}

// ADMINROLE is a free data retrieval call binding the contract method 0x75b238fc.
//
// Solidity: function ADMIN_ROLE() view returns(bytes32)
func (_OutboundChannel *OutboundChannelCallerSession) ADMINROLE() ([32]byte, error) {
	return _OutboundChannel.Contract.ADMINROLE(&_OutboundChannel.CallOpts)
}

// DEFAULTADMINROLE is a free data retrieval call binding the contract method 0xa217fddf.
//
// Solidity: function DEFAULT_ADMIN_ROLE() view returns(bytes32)
func (_OutboundChannel *OutboundChannelCaller) DEFAULTADMINROLE(opts *bind.CallOpts) ([32]byte, error) {
	var out []interface{}
	err := _OutboundChannel.contract.Call(opts, &out, "DEFAULT_ADMIN_ROLE")

	if err != nil {
		return *new([32]byte), err
	}

	out0 := *abi.ConvertType(out[0], new([32]byte)).(*[32]byte)

	return out0, err

}

// DEFAULTADMINROLE is a free data retrieval call binding the contract method 0xa217fddf.
//
// Solidity: function DEFAULT_ADMIN_ROLE() view returns(bytes32)
func (_OutboundChannel *OutboundChannelSession) DEFAULTADMINROLE() ([32]byte, error) {
	return _OutboundChannel.Contract.DEFAULTADMINROLE(&_OutboundChannel.CallOpts)
}

// DEFAULTADMINROLE is a free data retrieval call binding the contract method 0xa217fddf.
//
// Solidity: function DEFAULT_ADMIN_ROLE() view returns(bytes32)
func (_OutboundChannel *OutboundChannelCallerSession) DEFAULTADMINROLE() ([32]byte, error) {
	return _OutboundChannel.Contract.DEFAULTADMINROLE(&_OutboundChannel.CallOpts)
}

// SUBMITROLE is a free data retrieval call binding the contract method 0xa9c0c694.
//
// Solidity: function SUBMIT_ROLE() view returns(bytes32)
func (_OutboundChannel *OutboundChannelCaller) SUBMITROLE(opts *bind.CallOpts) ([32]byte, error) {
	var out []interface{}
	err := _OutboundChannel.contract.Call(opts, &out, "SUBMIT_ROLE")

	if err != nil {
		return *new([32]byte), err
	}

	out0 := *abi.ConvertType(out[0], new([32]byte)).(*[32]byte)

	return out0, err

}

// SUBMITROLE is a free data retrieval call binding the contract method 0xa9c0c694.
//
// Solidity: function SUBMIT_ROLE() view returns(bytes32)
func (_OutboundChannel *OutboundChannelSession) SUBMITROLE() ([32]byte, error) {
	return _OutboundChannel.Contract.SUBMITROLE(&_OutboundChannel.CallOpts)
}

// SUBMITROLE is a free data retrieval call binding the contract method 0xa9c0c694.
//
// Solidity: function SUBMIT_ROLE() view returns(bytes32)
func (_OutboundChannel *OutboundChannelCallerSession) SUBMITROLE() ([32]byte, error) {
	return _OutboundChannel.Contract.SUBMITROLE(&_OutboundChannel.CallOpts)
}

// Fee is a free data retrieval call binding the contract method 0xddca3f43.
//
// Solidity: function fee() view returns(uint256)
func (_OutboundChannel *OutboundChannelCaller) Fee(opts *bind.CallOpts) (*big.Int, error) {
	var out []interface{}
	err := _OutboundChannel.contract.Call(opts, &out, "fee")

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// Fee is a free data retrieval call binding the contract method 0xddca3f43.
//
// Solidity: function fee() view returns(uint256)
func (_OutboundChannel *OutboundChannelSession) Fee() (*big.Int, error) {
	return _OutboundChannel.Contract.Fee(&_OutboundChannel.CallOpts)
}

// Fee is a free data retrieval call binding the contract method 0xddca3f43.
//
// Solidity: function fee() view returns(uint256)
func (_OutboundChannel *OutboundChannelCallerSession) Fee() (*big.Int, error) {
	return _OutboundChannel.Contract.Fee(&_OutboundChannel.CallOpts)
}

// GetRoleAdmin is a free data retrieval call binding the contract method 0x248a9ca3.
//
// Solidity: function getRoleAdmin(bytes32 role) view returns(bytes32)
func (_OutboundChannel *OutboundChannelCaller) GetRoleAdmin(opts *bind.CallOpts, role [32]byte) ([32]byte, error) {
	var out []interface{}
	err := _OutboundChannel.contract.Call(opts, &out, "getRoleAdmin", role)

	if err != nil {
		return *new([32]byte), err
	}

	out0 := *abi.ConvertType(out[0], new([32]byte)).(*[32]byte)

	return out0, err

}

// GetRoleAdmin is a free data retrieval call binding the contract method 0x248a9ca3.
//
// Solidity: function getRoleAdmin(bytes32 role) view returns(bytes32)
func (_OutboundChannel *OutboundChannelSession) GetRoleAdmin(role [32]byte) ([32]byte, error) {
	return _OutboundChannel.Contract.GetRoleAdmin(&_OutboundChannel.CallOpts, role)
}

// GetRoleAdmin is a free data retrieval call binding the contract method 0x248a9ca3.
//
// Solidity: function getRoleAdmin(bytes32 role) view returns(bytes32)
func (_OutboundChannel *OutboundChannelCallerSession) GetRoleAdmin(role [32]byte) ([32]byte, error) {
	return _OutboundChannel.Contract.GetRoleAdmin(&_OutboundChannel.CallOpts, role)
}

// HasRole is a free data retrieval call binding the contract method 0x91d14854.
//
// Solidity: function hasRole(bytes32 role, address account) view returns(bool)
func (_OutboundChannel *OutboundChannelCaller) HasRole(opts *bind.CallOpts, role [32]byte, account common.Address) (bool, error) {
	var out []interface{}
	err := _OutboundChannel.contract.Call(opts, &out, "hasRole", role, account)

	if err != nil {
		return *new(bool), err
	}

	out0 := *abi.ConvertType(out[0], new(bool)).(*bool)

	return out0, err

}

// HasRole is a free data retrieval call binding the contract method 0x91d14854.
//
// Solidity: function hasRole(bytes32 role, address account) view returns(bool)
func (_OutboundChannel *OutboundChannelSession) HasRole(role [32]byte, account common.Address) (bool, error) {
	return _OutboundChannel.Contract.HasRole(&_OutboundChannel.CallOpts, role, account)
}

// HasRole is a free data retrieval call binding the contract method 0x91d14854.
//
// Solidity: function hasRole(bytes32 role, address account) view returns(bool)
func (_OutboundChannel *OutboundChannelCallerSession) HasRole(role [32]byte, account common.Address) (bool, error) {
	return _OutboundChannel.Contract.HasRole(&_OutboundChannel.CallOpts, role, account)
}

// Nonce is a free data retrieval call binding the contract method 0x4e765004.
//
// Solidity: function nonce(bytes ) view returns(uint64)
func (_OutboundChannel *OutboundChannelCaller) Nonce(opts *bind.CallOpts, arg0 []byte) (uint64, error) {
	var out []interface{}
	err := _OutboundChannel.contract.Call(opts, &out, "nonce", arg0)

	if err != nil {
		return *new(uint64), err
	}

	out0 := *abi.ConvertType(out[0], new(uint64)).(*uint64)

	return out0, err

}

// Nonce is a free data retrieval call binding the contract method 0x4e765004.
//
// Solidity: function nonce(bytes ) view returns(uint64)
func (_OutboundChannel *OutboundChannelSession) Nonce(arg0 []byte) (uint64, error) {
	return _OutboundChannel.Contract.Nonce(&_OutboundChannel.CallOpts, arg0)
}

// Nonce is a free data retrieval call binding the contract method 0x4e765004.
//
// Solidity: function nonce(bytes ) view returns(uint64)
func (_OutboundChannel *OutboundChannelCallerSession) Nonce(arg0 []byte) (uint64, error) {
	return _OutboundChannel.Contract.Nonce(&_OutboundChannel.CallOpts, arg0)
}

// SupportsInterface is a free data retrieval call binding the contract method 0x01ffc9a7.
//
// Solidity: function supportsInterface(bytes4 interfaceId) view returns(bool)
func (_OutboundChannel *OutboundChannelCaller) SupportsInterface(opts *bind.CallOpts, interfaceId [4]byte) (bool, error) {
	var out []interface{}
	err := _OutboundChannel.contract.Call(opts, &out, "supportsInterface", interfaceId)

	if err != nil {
		return *new(bool), err
	}

	out0 := *abi.ConvertType(out[0], new(bool)).(*bool)

	return out0, err

}

// SupportsInterface is a free data retrieval call binding the contract method 0x01ffc9a7.
//
// Solidity: function supportsInterface(bytes4 interfaceId) view returns(bool)
func (_OutboundChannel *OutboundChannelSession) SupportsInterface(interfaceId [4]byte) (bool, error) {
	return _OutboundChannel.Contract.SupportsInterface(&_OutboundChannel.CallOpts, interfaceId)
}

// SupportsInterface is a free data retrieval call binding the contract method 0x01ffc9a7.
//
// Solidity: function supportsInterface(bytes4 interfaceId) view returns(bool)
func (_OutboundChannel *OutboundChannelCallerSession) SupportsInterface(interfaceId [4]byte) (bool, error) {
	return _OutboundChannel.Contract.SupportsInterface(&_OutboundChannel.CallOpts, interfaceId)
}

// Vault is a free data retrieval call binding the contract method 0xfbfa77cf.
//
// Solidity: function vault() view returns(address)
func (_OutboundChannel *OutboundChannelCaller) Vault(opts *bind.CallOpts) (common.Address, error) {
	var out []interface{}
	err := _OutboundChannel.contract.Call(opts, &out, "vault")

	if err != nil {
		return *new(common.Address), err
	}

	out0 := *abi.ConvertType(out[0], new(common.Address)).(*common.Address)

	return out0, err

}

// Vault is a free data retrieval call binding the contract method 0xfbfa77cf.
//
// Solidity: function vault() view returns(address)
func (_OutboundChannel *OutboundChannelSession) Vault() (common.Address, error) {
	return _OutboundChannel.Contract.Vault(&_OutboundChannel.CallOpts)
}

// Vault is a free data retrieval call binding the contract method 0xfbfa77cf.
//
// Solidity: function vault() view returns(address)
func (_OutboundChannel *OutboundChannelCallerSession) Vault() (common.Address, error) {
	return _OutboundChannel.Contract.Vault(&_OutboundChannel.CallOpts)
}

// GrantRole is a paid mutator transaction binding the contract method 0x2f2ff15d.
//
// Solidity: function grantRole(bytes32 role, address account) returns()
func (_OutboundChannel *OutboundChannelTransactor) GrantRole(opts *bind.TransactOpts, role [32]byte, account common.Address) (*types.Transaction, error) {
	return _OutboundChannel.contract.Transact(opts, "grantRole", role, account)
}

// GrantRole is a paid mutator transaction binding the contract method 0x2f2ff15d.
//
// Solidity: function grantRole(bytes32 role, address account) returns()
func (_OutboundChannel *OutboundChannelSession) GrantRole(role [32]byte, account common.Address) (*types.Transaction, error) {
	return _OutboundChannel.Contract.GrantRole(&_OutboundChannel.TransactOpts, role, account)
}

// GrantRole is a paid mutator transaction binding the contract method 0x2f2ff15d.
//
// Solidity: function grantRole(bytes32 role, address account) returns()
func (_OutboundChannel *OutboundChannelTransactorSession) GrantRole(role [32]byte, account common.Address) (*types.Transaction, error) {
	return _OutboundChannel.Contract.GrantRole(&_OutboundChannel.TransactOpts, role, account)
}

// RenounceRole is a paid mutator transaction binding the contract method 0x36568abe.
//
// Solidity: function renounceRole(bytes32 role, address account) returns()
func (_OutboundChannel *OutboundChannelTransactor) RenounceRole(opts *bind.TransactOpts, role [32]byte, account common.Address) (*types.Transaction, error) {
	return _OutboundChannel.contract.Transact(opts, "renounceRole", role, account)
}

// RenounceRole is a paid mutator transaction binding the contract method 0x36568abe.
//
// Solidity: function renounceRole(bytes32 role, address account) returns()
func (_OutboundChannel *OutboundChannelSession) RenounceRole(role [32]byte, account common.Address) (*types.Transaction, error) {
	return _OutboundChannel.Contract.RenounceRole(&_OutboundChannel.TransactOpts, role, account)
}

// RenounceRole is a paid mutator transaction binding the contract method 0x36568abe.
//
// Solidity: function renounceRole(bytes32 role, address account) returns()
func (_OutboundChannel *OutboundChannelTransactorSession) RenounceRole(role [32]byte, account common.Address) (*types.Transaction, error) {
	return _OutboundChannel.Contract.RenounceRole(&_OutboundChannel.TransactOpts, role, account)
}

// RevokeRole is a paid mutator transaction binding the contract method 0xd547741f.
//
// Solidity: function revokeRole(bytes32 role, address account) returns()
func (_OutboundChannel *OutboundChannelTransactor) RevokeRole(opts *bind.TransactOpts, role [32]byte, account common.Address) (*types.Transaction, error) {
	return _OutboundChannel.contract.Transact(opts, "revokeRole", role, account)
}

// RevokeRole is a paid mutator transaction binding the contract method 0xd547741f.
//
// Solidity: function revokeRole(bytes32 role, address account) returns()
func (_OutboundChannel *OutboundChannelSession) RevokeRole(role [32]byte, account common.Address) (*types.Transaction, error) {
	return _OutboundChannel.Contract.RevokeRole(&_OutboundChannel.TransactOpts, role, account)
}

// RevokeRole is a paid mutator transaction binding the contract method 0xd547741f.
//
// Solidity: function revokeRole(bytes32 role, address account) returns()
func (_OutboundChannel *OutboundChannelTransactorSession) RevokeRole(role [32]byte, account common.Address) (*types.Transaction, error) {
	return _OutboundChannel.Contract.RevokeRole(&_OutboundChannel.TransactOpts, role, account)
}

// Submit is a paid mutator transaction binding the contract method 0x39b973ad.
//
// Solidity: function submit(bytes dest, bytes payload) payable returns()
func (_OutboundChannel *OutboundChannelTransactor) Submit(opts *bind.TransactOpts, dest []byte, payload []byte) (*types.Transaction, error) {
	return _OutboundChannel.contract.Transact(opts, "submit", dest, payload)
}

// Submit is a paid mutator transaction binding the contract method 0x39b973ad.
//
// Solidity: function submit(bytes dest, bytes payload) payable returns()
func (_OutboundChannel *OutboundChannelSession) Submit(dest []byte, payload []byte) (*types.Transaction, error) {
	return _OutboundChannel.Contract.Submit(&_OutboundChannel.TransactOpts, dest, payload)
}

// Submit is a paid mutator transaction binding the contract method 0x39b973ad.
//
// Solidity: function submit(bytes dest, bytes payload) payable returns()
func (_OutboundChannel *OutboundChannelTransactorSession) Submit(dest []byte, payload []byte) (*types.Transaction, error) {
	return _OutboundChannel.Contract.Submit(&_OutboundChannel.TransactOpts, dest, payload)
}

// UpdateFee is a paid mutator transaction binding the contract method 0x9012c4a8.
//
// Solidity: function updateFee(uint256 newFee) returns()
func (_OutboundChannel *OutboundChannelTransactor) UpdateFee(opts *bind.TransactOpts, newFee *big.Int) (*types.Transaction, error) {
	return _OutboundChannel.contract.Transact(opts, "updateFee", newFee)
}

// UpdateFee is a paid mutator transaction binding the contract method 0x9012c4a8.
//
// Solidity: function updateFee(uint256 newFee) returns()
func (_OutboundChannel *OutboundChannelSession) UpdateFee(newFee *big.Int) (*types.Transaction, error) {
	return _OutboundChannel.Contract.UpdateFee(&_OutboundChannel.TransactOpts, newFee)
}

// UpdateFee is a paid mutator transaction binding the contract method 0x9012c4a8.
//
// Solidity: function updateFee(uint256 newFee) returns()
func (_OutboundChannel *OutboundChannelTransactorSession) UpdateFee(newFee *big.Int) (*types.Transaction, error) {
	return _OutboundChannel.Contract.UpdateFee(&_OutboundChannel.TransactOpts, newFee)
}

// OutboundChannelFeeUpdatedIterator is returned from FilterFeeUpdated and is used to iterate over the raw logs and unpacked data for FeeUpdated events raised by the OutboundChannel contract.
type OutboundChannelFeeUpdatedIterator struct {
	Event *OutboundChannelFeeUpdated // Event containing the contract specifics and raw log

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
func (it *OutboundChannelFeeUpdatedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(OutboundChannelFeeUpdated)
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
		it.Event = new(OutboundChannelFeeUpdated)
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
func (it *OutboundChannelFeeUpdatedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *OutboundChannelFeeUpdatedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// OutboundChannelFeeUpdated represents a FeeUpdated event raised by the OutboundChannel contract.
type OutboundChannelFeeUpdated struct {
	Fee *big.Int
	Raw types.Log // Blockchain specific contextual infos
}

// FilterFeeUpdated is a free log retrieval operation binding the contract event 0x8c4d35e54a3f2ef1134138fd8ea3daee6a3c89e10d2665996babdf70261e2c76.
//
// Solidity: event FeeUpdated(uint256 fee)
func (_OutboundChannel *OutboundChannelFilterer) FilterFeeUpdated(opts *bind.FilterOpts) (*OutboundChannelFeeUpdatedIterator, error) {

	logs, sub, err := _OutboundChannel.contract.FilterLogs(opts, "FeeUpdated")
	if err != nil {
		return nil, err
	}
	return &OutboundChannelFeeUpdatedIterator{contract: _OutboundChannel.contract, event: "FeeUpdated", logs: logs, sub: sub}, nil
}

// WatchFeeUpdated is a free log subscription operation binding the contract event 0x8c4d35e54a3f2ef1134138fd8ea3daee6a3c89e10d2665996babdf70261e2c76.
//
// Solidity: event FeeUpdated(uint256 fee)
func (_OutboundChannel *OutboundChannelFilterer) WatchFeeUpdated(opts *bind.WatchOpts, sink chan<- *OutboundChannelFeeUpdated) (event.Subscription, error) {

	logs, sub, err := _OutboundChannel.contract.WatchLogs(opts, "FeeUpdated")
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(OutboundChannelFeeUpdated)
				if err := _OutboundChannel.contract.UnpackLog(event, "FeeUpdated", log); err != nil {
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
func (_OutboundChannel *OutboundChannelFilterer) ParseFeeUpdated(log types.Log) (*OutboundChannelFeeUpdated, error) {
	event := new(OutboundChannelFeeUpdated)
	if err := _OutboundChannel.contract.UnpackLog(event, "FeeUpdated", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// OutboundChannelMessageIterator is returned from FilterMessage and is used to iterate over the raw logs and unpacked data for Message events raised by the OutboundChannel contract.
type OutboundChannelMessageIterator struct {
	Event *OutboundChannelMessage // Event containing the contract specifics and raw log

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
func (it *OutboundChannelMessageIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(OutboundChannelMessage)
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
		it.Event = new(OutboundChannelMessage)
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
func (it *OutboundChannelMessageIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *OutboundChannelMessageIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// OutboundChannelMessage represents a Message event raised by the OutboundChannel contract.
type OutboundChannelMessage struct {
	Dest    common.Hash
	Nonce   uint64
	Payload []byte
	Raw     types.Log // Blockchain specific contextual infos
}

// FilterMessage is a free log retrieval operation binding the contract event 0xb68e760461a2cd8661cea966252574a5355ad3ed096afdd6572c65ef50bb7cd7.
//
// Solidity: event Message(bytes indexed dest, uint64 nonce, bytes payload)
func (_OutboundChannel *OutboundChannelFilterer) FilterMessage(opts *bind.FilterOpts, dest [][]byte) (*OutboundChannelMessageIterator, error) {

	var destRule []interface{}
	for _, destItem := range dest {
		destRule = append(destRule, destItem)
	}

	logs, sub, err := _OutboundChannel.contract.FilterLogs(opts, "Message", destRule)
	if err != nil {
		return nil, err
	}
	return &OutboundChannelMessageIterator{contract: _OutboundChannel.contract, event: "Message", logs: logs, sub: sub}, nil
}

// WatchMessage is a free log subscription operation binding the contract event 0xb68e760461a2cd8661cea966252574a5355ad3ed096afdd6572c65ef50bb7cd7.
//
// Solidity: event Message(bytes indexed dest, uint64 nonce, bytes payload)
func (_OutboundChannel *OutboundChannelFilterer) WatchMessage(opts *bind.WatchOpts, sink chan<- *OutboundChannelMessage, dest [][]byte) (event.Subscription, error) {

	var destRule []interface{}
	for _, destItem := range dest {
		destRule = append(destRule, destItem)
	}

	logs, sub, err := _OutboundChannel.contract.WatchLogs(opts, "Message", destRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(OutboundChannelMessage)
				if err := _OutboundChannel.contract.UnpackLog(event, "Message", log); err != nil {
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
func (_OutboundChannel *OutboundChannelFilterer) ParseMessage(log types.Log) (*OutboundChannelMessage, error) {
	event := new(OutboundChannelMessage)
	if err := _OutboundChannel.contract.UnpackLog(event, "Message", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// OutboundChannelRoleAdminChangedIterator is returned from FilterRoleAdminChanged and is used to iterate over the raw logs and unpacked data for RoleAdminChanged events raised by the OutboundChannel contract.
type OutboundChannelRoleAdminChangedIterator struct {
	Event *OutboundChannelRoleAdminChanged // Event containing the contract specifics and raw log

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
func (it *OutboundChannelRoleAdminChangedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(OutboundChannelRoleAdminChanged)
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
		it.Event = new(OutboundChannelRoleAdminChanged)
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
func (it *OutboundChannelRoleAdminChangedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *OutboundChannelRoleAdminChangedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// OutboundChannelRoleAdminChanged represents a RoleAdminChanged event raised by the OutboundChannel contract.
type OutboundChannelRoleAdminChanged struct {
	Role              [32]byte
	PreviousAdminRole [32]byte
	NewAdminRole      [32]byte
	Raw               types.Log // Blockchain specific contextual infos
}

// FilterRoleAdminChanged is a free log retrieval operation binding the contract event 0xbd79b86ffe0ab8e8776151514217cd7cacd52c909f66475c3af44e129f0b00ff.
//
// Solidity: event RoleAdminChanged(bytes32 indexed role, bytes32 indexed previousAdminRole, bytes32 indexed newAdminRole)
func (_OutboundChannel *OutboundChannelFilterer) FilterRoleAdminChanged(opts *bind.FilterOpts, role [][32]byte, previousAdminRole [][32]byte, newAdminRole [][32]byte) (*OutboundChannelRoleAdminChangedIterator, error) {

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

	logs, sub, err := _OutboundChannel.contract.FilterLogs(opts, "RoleAdminChanged", roleRule, previousAdminRoleRule, newAdminRoleRule)
	if err != nil {
		return nil, err
	}
	return &OutboundChannelRoleAdminChangedIterator{contract: _OutboundChannel.contract, event: "RoleAdminChanged", logs: logs, sub: sub}, nil
}

// WatchRoleAdminChanged is a free log subscription operation binding the contract event 0xbd79b86ffe0ab8e8776151514217cd7cacd52c909f66475c3af44e129f0b00ff.
//
// Solidity: event RoleAdminChanged(bytes32 indexed role, bytes32 indexed previousAdminRole, bytes32 indexed newAdminRole)
func (_OutboundChannel *OutboundChannelFilterer) WatchRoleAdminChanged(opts *bind.WatchOpts, sink chan<- *OutboundChannelRoleAdminChanged, role [][32]byte, previousAdminRole [][32]byte, newAdminRole [][32]byte) (event.Subscription, error) {

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

	logs, sub, err := _OutboundChannel.contract.WatchLogs(opts, "RoleAdminChanged", roleRule, previousAdminRoleRule, newAdminRoleRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(OutboundChannelRoleAdminChanged)
				if err := _OutboundChannel.contract.UnpackLog(event, "RoleAdminChanged", log); err != nil {
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
func (_OutboundChannel *OutboundChannelFilterer) ParseRoleAdminChanged(log types.Log) (*OutboundChannelRoleAdminChanged, error) {
	event := new(OutboundChannelRoleAdminChanged)
	if err := _OutboundChannel.contract.UnpackLog(event, "RoleAdminChanged", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// OutboundChannelRoleGrantedIterator is returned from FilterRoleGranted and is used to iterate over the raw logs and unpacked data for RoleGranted events raised by the OutboundChannel contract.
type OutboundChannelRoleGrantedIterator struct {
	Event *OutboundChannelRoleGranted // Event containing the contract specifics and raw log

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
func (it *OutboundChannelRoleGrantedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(OutboundChannelRoleGranted)
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
		it.Event = new(OutboundChannelRoleGranted)
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
func (it *OutboundChannelRoleGrantedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *OutboundChannelRoleGrantedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// OutboundChannelRoleGranted represents a RoleGranted event raised by the OutboundChannel contract.
type OutboundChannelRoleGranted struct {
	Role    [32]byte
	Account common.Address
	Sender  common.Address
	Raw     types.Log // Blockchain specific contextual infos
}

// FilterRoleGranted is a free log retrieval operation binding the contract event 0x2f8788117e7eff1d82e926ec794901d17c78024a50270940304540a733656f0d.
//
// Solidity: event RoleGranted(bytes32 indexed role, address indexed account, address indexed sender)
func (_OutboundChannel *OutboundChannelFilterer) FilterRoleGranted(opts *bind.FilterOpts, role [][32]byte, account []common.Address, sender []common.Address) (*OutboundChannelRoleGrantedIterator, error) {

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

	logs, sub, err := _OutboundChannel.contract.FilterLogs(opts, "RoleGranted", roleRule, accountRule, senderRule)
	if err != nil {
		return nil, err
	}
	return &OutboundChannelRoleGrantedIterator{contract: _OutboundChannel.contract, event: "RoleGranted", logs: logs, sub: sub}, nil
}

// WatchRoleGranted is a free log subscription operation binding the contract event 0x2f8788117e7eff1d82e926ec794901d17c78024a50270940304540a733656f0d.
//
// Solidity: event RoleGranted(bytes32 indexed role, address indexed account, address indexed sender)
func (_OutboundChannel *OutboundChannelFilterer) WatchRoleGranted(opts *bind.WatchOpts, sink chan<- *OutboundChannelRoleGranted, role [][32]byte, account []common.Address, sender []common.Address) (event.Subscription, error) {

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

	logs, sub, err := _OutboundChannel.contract.WatchLogs(opts, "RoleGranted", roleRule, accountRule, senderRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(OutboundChannelRoleGranted)
				if err := _OutboundChannel.contract.UnpackLog(event, "RoleGranted", log); err != nil {
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
func (_OutboundChannel *OutboundChannelFilterer) ParseRoleGranted(log types.Log) (*OutboundChannelRoleGranted, error) {
	event := new(OutboundChannelRoleGranted)
	if err := _OutboundChannel.contract.UnpackLog(event, "RoleGranted", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// OutboundChannelRoleRevokedIterator is returned from FilterRoleRevoked and is used to iterate over the raw logs and unpacked data for RoleRevoked events raised by the OutboundChannel contract.
type OutboundChannelRoleRevokedIterator struct {
	Event *OutboundChannelRoleRevoked // Event containing the contract specifics and raw log

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
func (it *OutboundChannelRoleRevokedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(OutboundChannelRoleRevoked)
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
		it.Event = new(OutboundChannelRoleRevoked)
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
func (it *OutboundChannelRoleRevokedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *OutboundChannelRoleRevokedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// OutboundChannelRoleRevoked represents a RoleRevoked event raised by the OutboundChannel contract.
type OutboundChannelRoleRevoked struct {
	Role    [32]byte
	Account common.Address
	Sender  common.Address
	Raw     types.Log // Blockchain specific contextual infos
}

// FilterRoleRevoked is a free log retrieval operation binding the contract event 0xf6391f5c32d9c69d2a47ea670b442974b53935d1edc7fd64eb21e047a839171b.
//
// Solidity: event RoleRevoked(bytes32 indexed role, address indexed account, address indexed sender)
func (_OutboundChannel *OutboundChannelFilterer) FilterRoleRevoked(opts *bind.FilterOpts, role [][32]byte, account []common.Address, sender []common.Address) (*OutboundChannelRoleRevokedIterator, error) {

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

	logs, sub, err := _OutboundChannel.contract.FilterLogs(opts, "RoleRevoked", roleRule, accountRule, senderRule)
	if err != nil {
		return nil, err
	}
	return &OutboundChannelRoleRevokedIterator{contract: _OutboundChannel.contract, event: "RoleRevoked", logs: logs, sub: sub}, nil
}

// WatchRoleRevoked is a free log subscription operation binding the contract event 0xf6391f5c32d9c69d2a47ea670b442974b53935d1edc7fd64eb21e047a839171b.
//
// Solidity: event RoleRevoked(bytes32 indexed role, address indexed account, address indexed sender)
func (_OutboundChannel *OutboundChannelFilterer) WatchRoleRevoked(opts *bind.WatchOpts, sink chan<- *OutboundChannelRoleRevoked, role [][32]byte, account []common.Address, sender []common.Address) (event.Subscription, error) {

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

	logs, sub, err := _OutboundChannel.contract.WatchLogs(opts, "RoleRevoked", roleRule, accountRule, senderRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(OutboundChannelRoleRevoked)
				if err := _OutboundChannel.contract.UnpackLog(event, "RoleRevoked", log); err != nil {
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
func (_OutboundChannel *OutboundChannelFilterer) ParseRoleRevoked(log types.Log) (*OutboundChannelRoleRevoked, error) {
	event := new(OutboundChannelRoleRevoked)
	if err := _OutboundChannel.contract.UnpackLog(event, "RoleRevoked", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}
