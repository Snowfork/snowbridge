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
const BasicOutboundChannelABI = "[{\"inputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"constructor\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":false,\"internalType\":\"address\",\"name\":\"source\",\"type\":\"address\"},{\"indexed\":false,\"internalType\":\"uint64\",\"name\":\"nonce\",\"type\":\"uint64\"},{\"indexed\":false,\"internalType\":\"bytes\",\"name\":\"payload\",\"type\":\"bytes\"}],\"name\":\"Message\",\"type\":\"event\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":false,\"internalType\":\"address\",\"name\":\"operator\",\"type\":\"address\"},{\"indexed\":false,\"internalType\":\"address\",\"name\":\"user\",\"type\":\"address\"}],\"name\":\"OperatorAuthorized\",\"type\":\"event\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":false,\"internalType\":\"address\",\"name\":\"operator\",\"type\":\"address\"},{\"indexed\":false,\"internalType\":\"address\",\"name\":\"user\",\"type\":\"address\"}],\"name\":\"OperatorRevoked\",\"type\":\"event\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":true,\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\"},{\"indexed\":true,\"internalType\":\"bytes32\",\"name\":\"previousAdminRole\",\"type\":\"bytes32\"},{\"indexed\":true,\"internalType\":\"bytes32\",\"name\":\"newAdminRole\",\"type\":\"bytes32\"}],\"name\":\"RoleAdminChanged\",\"type\":\"event\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":true,\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\"},{\"indexed\":true,\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\"},{\"indexed\":true,\"internalType\":\"address\",\"name\":\"sender\",\"type\":\"address\"}],\"name\":\"RoleGranted\",\"type\":\"event\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":true,\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\"},{\"indexed\":true,\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\"},{\"indexed\":true,\"internalType\":\"address\",\"name\":\"sender\",\"type\":\"address\"}],\"name\":\"RoleRevoked\",\"type\":\"event\"},{\"inputs\":[],\"name\":\"CONFIG_UPDATE_ROLE\",\"outputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"DEFAULT_ADMIN_ROLE\",\"outputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"operator\",\"type\":\"address\"}],\"name\":\"authorizeDefaultOperator\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"operator\",\"type\":\"address\"}],\"name\":\"authorizeOperator\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\"}],\"name\":\"getRoleAdmin\",\"outputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\"},{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\"}],\"name\":\"grantRole\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\"},{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\"}],\"name\":\"hasRole\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"_configUpdater\",\"type\":\"address\"},{\"internalType\":\"address\",\"name\":\"_principal\",\"type\":\"address\"},{\"internalType\":\"address[]\",\"name\":\"defaultOperators\",\"type\":\"address[]\"}],\"name\":\"initialize\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"_operator\",\"type\":\"address\"},{\"internalType\":\"address\",\"name\":\"_origin\",\"type\":\"address\"}],\"name\":\"isOperatorFor\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"nonce\",\"outputs\":[{\"internalType\":\"uint64\",\"name\":\"\",\"type\":\"uint64\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"principal\",\"outputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\"},{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\"}],\"name\":\"renounceRole\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"operator\",\"type\":\"address\"}],\"name\":\"revokeDefaultOperator\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"operator\",\"type\":\"address\"}],\"name\":\"revokeOperator\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\"},{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\"}],\"name\":\"revokeRole\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"_principal\",\"type\":\"address\"}],\"name\":\"setPrincipal\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"_origin\",\"type\":\"address\"},{\"internalType\":\"bytes\",\"name\":\"_payload\",\"type\":\"bytes\"}],\"name\":\"submit\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"bytes4\",\"name\":\"interfaceId\",\"type\":\"bytes4\"}],\"name\":\"supportsInterface\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\"}],\"stateMutability\":\"view\",\"type\":\"function\"}]"

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

// CONFIGUPDATEROLE is a free data retrieval call binding the contract method 0xa2d6c6e5.
//
// Solidity: function CONFIG_UPDATE_ROLE() view returns(bytes32)
func (_BasicOutboundChannel *BasicOutboundChannelCaller) CONFIGUPDATEROLE(opts *bind.CallOpts) ([32]byte, error) {
	var out []interface{}
	err := _BasicOutboundChannel.contract.Call(opts, &out, "CONFIG_UPDATE_ROLE")

	if err != nil {
		return *new([32]byte), err
	}

	out0 := *abi.ConvertType(out[0], new([32]byte)).(*[32]byte)

	return out0, err

}

// CONFIGUPDATEROLE is a free data retrieval call binding the contract method 0xa2d6c6e5.
//
// Solidity: function CONFIG_UPDATE_ROLE() view returns(bytes32)
func (_BasicOutboundChannel *BasicOutboundChannelSession) CONFIGUPDATEROLE() ([32]byte, error) {
	return _BasicOutboundChannel.Contract.CONFIGUPDATEROLE(&_BasicOutboundChannel.CallOpts)
}

// CONFIGUPDATEROLE is a free data retrieval call binding the contract method 0xa2d6c6e5.
//
// Solidity: function CONFIG_UPDATE_ROLE() view returns(bytes32)
func (_BasicOutboundChannel *BasicOutboundChannelCallerSession) CONFIGUPDATEROLE() ([32]byte, error) {
	return _BasicOutboundChannel.Contract.CONFIGUPDATEROLE(&_BasicOutboundChannel.CallOpts)
}

// DEFAULTADMINROLE is a free data retrieval call binding the contract method 0xa217fddf.
//
// Solidity: function DEFAULT_ADMIN_ROLE() view returns(bytes32)
func (_BasicOutboundChannel *BasicOutboundChannelCaller) DEFAULTADMINROLE(opts *bind.CallOpts) ([32]byte, error) {
	var out []interface{}
	err := _BasicOutboundChannel.contract.Call(opts, &out, "DEFAULT_ADMIN_ROLE")

	if err != nil {
		return *new([32]byte), err
	}

	out0 := *abi.ConvertType(out[0], new([32]byte)).(*[32]byte)

	return out0, err

}

// DEFAULTADMINROLE is a free data retrieval call binding the contract method 0xa217fddf.
//
// Solidity: function DEFAULT_ADMIN_ROLE() view returns(bytes32)
func (_BasicOutboundChannel *BasicOutboundChannelSession) DEFAULTADMINROLE() ([32]byte, error) {
	return _BasicOutboundChannel.Contract.DEFAULTADMINROLE(&_BasicOutboundChannel.CallOpts)
}

// DEFAULTADMINROLE is a free data retrieval call binding the contract method 0xa217fddf.
//
// Solidity: function DEFAULT_ADMIN_ROLE() view returns(bytes32)
func (_BasicOutboundChannel *BasicOutboundChannelCallerSession) DEFAULTADMINROLE() ([32]byte, error) {
	return _BasicOutboundChannel.Contract.DEFAULTADMINROLE(&_BasicOutboundChannel.CallOpts)
}

// GetRoleAdmin is a free data retrieval call binding the contract method 0x248a9ca3.
//
// Solidity: function getRoleAdmin(bytes32 role) view returns(bytes32)
func (_BasicOutboundChannel *BasicOutboundChannelCaller) GetRoleAdmin(opts *bind.CallOpts, role [32]byte) ([32]byte, error) {
	var out []interface{}
	err := _BasicOutboundChannel.contract.Call(opts, &out, "getRoleAdmin", role)

	if err != nil {
		return *new([32]byte), err
	}

	out0 := *abi.ConvertType(out[0], new([32]byte)).(*[32]byte)

	return out0, err

}

// GetRoleAdmin is a free data retrieval call binding the contract method 0x248a9ca3.
//
// Solidity: function getRoleAdmin(bytes32 role) view returns(bytes32)
func (_BasicOutboundChannel *BasicOutboundChannelSession) GetRoleAdmin(role [32]byte) ([32]byte, error) {
	return _BasicOutboundChannel.Contract.GetRoleAdmin(&_BasicOutboundChannel.CallOpts, role)
}

// GetRoleAdmin is a free data retrieval call binding the contract method 0x248a9ca3.
//
// Solidity: function getRoleAdmin(bytes32 role) view returns(bytes32)
func (_BasicOutboundChannel *BasicOutboundChannelCallerSession) GetRoleAdmin(role [32]byte) ([32]byte, error) {
	return _BasicOutboundChannel.Contract.GetRoleAdmin(&_BasicOutboundChannel.CallOpts, role)
}

// HasRole is a free data retrieval call binding the contract method 0x91d14854.
//
// Solidity: function hasRole(bytes32 role, address account) view returns(bool)
func (_BasicOutboundChannel *BasicOutboundChannelCaller) HasRole(opts *bind.CallOpts, role [32]byte, account common.Address) (bool, error) {
	var out []interface{}
	err := _BasicOutboundChannel.contract.Call(opts, &out, "hasRole", role, account)

	if err != nil {
		return *new(bool), err
	}

	out0 := *abi.ConvertType(out[0], new(bool)).(*bool)

	return out0, err

}

// HasRole is a free data retrieval call binding the contract method 0x91d14854.
//
// Solidity: function hasRole(bytes32 role, address account) view returns(bool)
func (_BasicOutboundChannel *BasicOutboundChannelSession) HasRole(role [32]byte, account common.Address) (bool, error) {
	return _BasicOutboundChannel.Contract.HasRole(&_BasicOutboundChannel.CallOpts, role, account)
}

// HasRole is a free data retrieval call binding the contract method 0x91d14854.
//
// Solidity: function hasRole(bytes32 role, address account) view returns(bool)
func (_BasicOutboundChannel *BasicOutboundChannelCallerSession) HasRole(role [32]byte, account common.Address) (bool, error) {
	return _BasicOutboundChannel.Contract.HasRole(&_BasicOutboundChannel.CallOpts, role, account)
}

// IsOperatorFor is a free data retrieval call binding the contract method 0xd95b6371.
//
// Solidity: function isOperatorFor(address _operator, address _origin) view returns(bool)
func (_BasicOutboundChannel *BasicOutboundChannelCaller) IsOperatorFor(opts *bind.CallOpts, _operator common.Address, _origin common.Address) (bool, error) {
	var out []interface{}
	err := _BasicOutboundChannel.contract.Call(opts, &out, "isOperatorFor", _operator, _origin)

	if err != nil {
		return *new(bool), err
	}

	out0 := *abi.ConvertType(out[0], new(bool)).(*bool)

	return out0, err

}

// IsOperatorFor is a free data retrieval call binding the contract method 0xd95b6371.
//
// Solidity: function isOperatorFor(address _operator, address _origin) view returns(bool)
func (_BasicOutboundChannel *BasicOutboundChannelSession) IsOperatorFor(_operator common.Address, _origin common.Address) (bool, error) {
	return _BasicOutboundChannel.Contract.IsOperatorFor(&_BasicOutboundChannel.CallOpts, _operator, _origin)
}

// IsOperatorFor is a free data retrieval call binding the contract method 0xd95b6371.
//
// Solidity: function isOperatorFor(address _operator, address _origin) view returns(bool)
func (_BasicOutboundChannel *BasicOutboundChannelCallerSession) IsOperatorFor(_operator common.Address, _origin common.Address) (bool, error) {
	return _BasicOutboundChannel.Contract.IsOperatorFor(&_BasicOutboundChannel.CallOpts, _operator, _origin)
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

// Principal is a free data retrieval call binding the contract method 0xba5d3078.
//
// Solidity: function principal() view returns(address)
func (_BasicOutboundChannel *BasicOutboundChannelCaller) Principal(opts *bind.CallOpts) (common.Address, error) {
	var out []interface{}
	err := _BasicOutboundChannel.contract.Call(opts, &out, "principal")

	if err != nil {
		return *new(common.Address), err
	}

	out0 := *abi.ConvertType(out[0], new(common.Address)).(*common.Address)

	return out0, err

}

// Principal is a free data retrieval call binding the contract method 0xba5d3078.
//
// Solidity: function principal() view returns(address)
func (_BasicOutboundChannel *BasicOutboundChannelSession) Principal() (common.Address, error) {
	return _BasicOutboundChannel.Contract.Principal(&_BasicOutboundChannel.CallOpts)
}

// Principal is a free data retrieval call binding the contract method 0xba5d3078.
//
// Solidity: function principal() view returns(address)
func (_BasicOutboundChannel *BasicOutboundChannelCallerSession) Principal() (common.Address, error) {
	return _BasicOutboundChannel.Contract.Principal(&_BasicOutboundChannel.CallOpts)
}

// SupportsInterface is a free data retrieval call binding the contract method 0x01ffc9a7.
//
// Solidity: function supportsInterface(bytes4 interfaceId) view returns(bool)
func (_BasicOutboundChannel *BasicOutboundChannelCaller) SupportsInterface(opts *bind.CallOpts, interfaceId [4]byte) (bool, error) {
	var out []interface{}
	err := _BasicOutboundChannel.contract.Call(opts, &out, "supportsInterface", interfaceId)

	if err != nil {
		return *new(bool), err
	}

	out0 := *abi.ConvertType(out[0], new(bool)).(*bool)

	return out0, err

}

// SupportsInterface is a free data retrieval call binding the contract method 0x01ffc9a7.
//
// Solidity: function supportsInterface(bytes4 interfaceId) view returns(bool)
func (_BasicOutboundChannel *BasicOutboundChannelSession) SupportsInterface(interfaceId [4]byte) (bool, error) {
	return _BasicOutboundChannel.Contract.SupportsInterface(&_BasicOutboundChannel.CallOpts, interfaceId)
}

// SupportsInterface is a free data retrieval call binding the contract method 0x01ffc9a7.
//
// Solidity: function supportsInterface(bytes4 interfaceId) view returns(bool)
func (_BasicOutboundChannel *BasicOutboundChannelCallerSession) SupportsInterface(interfaceId [4]byte) (bool, error) {
	return _BasicOutboundChannel.Contract.SupportsInterface(&_BasicOutboundChannel.CallOpts, interfaceId)
}

// AuthorizeDefaultOperator is a paid mutator transaction binding the contract method 0xb742a404.
//
// Solidity: function authorizeDefaultOperator(address operator) returns()
func (_BasicOutboundChannel *BasicOutboundChannelTransactor) AuthorizeDefaultOperator(opts *bind.TransactOpts, operator common.Address) (*types.Transaction, error) {
	return _BasicOutboundChannel.contract.Transact(opts, "authorizeDefaultOperator", operator)
}

// AuthorizeDefaultOperator is a paid mutator transaction binding the contract method 0xb742a404.
//
// Solidity: function authorizeDefaultOperator(address operator) returns()
func (_BasicOutboundChannel *BasicOutboundChannelSession) AuthorizeDefaultOperator(operator common.Address) (*types.Transaction, error) {
	return _BasicOutboundChannel.Contract.AuthorizeDefaultOperator(&_BasicOutboundChannel.TransactOpts, operator)
}

// AuthorizeDefaultOperator is a paid mutator transaction binding the contract method 0xb742a404.
//
// Solidity: function authorizeDefaultOperator(address operator) returns()
func (_BasicOutboundChannel *BasicOutboundChannelTransactorSession) AuthorizeDefaultOperator(operator common.Address) (*types.Transaction, error) {
	return _BasicOutboundChannel.Contract.AuthorizeDefaultOperator(&_BasicOutboundChannel.TransactOpts, operator)
}

// AuthorizeOperator is a paid mutator transaction binding the contract method 0x959b8c3f.
//
// Solidity: function authorizeOperator(address operator) returns()
func (_BasicOutboundChannel *BasicOutboundChannelTransactor) AuthorizeOperator(opts *bind.TransactOpts, operator common.Address) (*types.Transaction, error) {
	return _BasicOutboundChannel.contract.Transact(opts, "authorizeOperator", operator)
}

// AuthorizeOperator is a paid mutator transaction binding the contract method 0x959b8c3f.
//
// Solidity: function authorizeOperator(address operator) returns()
func (_BasicOutboundChannel *BasicOutboundChannelSession) AuthorizeOperator(operator common.Address) (*types.Transaction, error) {
	return _BasicOutboundChannel.Contract.AuthorizeOperator(&_BasicOutboundChannel.TransactOpts, operator)
}

// AuthorizeOperator is a paid mutator transaction binding the contract method 0x959b8c3f.
//
// Solidity: function authorizeOperator(address operator) returns()
func (_BasicOutboundChannel *BasicOutboundChannelTransactorSession) AuthorizeOperator(operator common.Address) (*types.Transaction, error) {
	return _BasicOutboundChannel.Contract.AuthorizeOperator(&_BasicOutboundChannel.TransactOpts, operator)
}

// GrantRole is a paid mutator transaction binding the contract method 0x2f2ff15d.
//
// Solidity: function grantRole(bytes32 role, address account) returns()
func (_BasicOutboundChannel *BasicOutboundChannelTransactor) GrantRole(opts *bind.TransactOpts, role [32]byte, account common.Address) (*types.Transaction, error) {
	return _BasicOutboundChannel.contract.Transact(opts, "grantRole", role, account)
}

// GrantRole is a paid mutator transaction binding the contract method 0x2f2ff15d.
//
// Solidity: function grantRole(bytes32 role, address account) returns()
func (_BasicOutboundChannel *BasicOutboundChannelSession) GrantRole(role [32]byte, account common.Address) (*types.Transaction, error) {
	return _BasicOutboundChannel.Contract.GrantRole(&_BasicOutboundChannel.TransactOpts, role, account)
}

// GrantRole is a paid mutator transaction binding the contract method 0x2f2ff15d.
//
// Solidity: function grantRole(bytes32 role, address account) returns()
func (_BasicOutboundChannel *BasicOutboundChannelTransactorSession) GrantRole(role [32]byte, account common.Address) (*types.Transaction, error) {
	return _BasicOutboundChannel.Contract.GrantRole(&_BasicOutboundChannel.TransactOpts, role, account)
}

// Initialize is a paid mutator transaction binding the contract method 0x77a24f36.
//
// Solidity: function initialize(address _configUpdater, address _principal, address[] defaultOperators) returns()
func (_BasicOutboundChannel *BasicOutboundChannelTransactor) Initialize(opts *bind.TransactOpts, _configUpdater common.Address, _principal common.Address, defaultOperators []common.Address) (*types.Transaction, error) {
	return _BasicOutboundChannel.contract.Transact(opts, "initialize", _configUpdater, _principal, defaultOperators)
}

// Initialize is a paid mutator transaction binding the contract method 0x77a24f36.
//
// Solidity: function initialize(address _configUpdater, address _principal, address[] defaultOperators) returns()
func (_BasicOutboundChannel *BasicOutboundChannelSession) Initialize(_configUpdater common.Address, _principal common.Address, defaultOperators []common.Address) (*types.Transaction, error) {
	return _BasicOutboundChannel.Contract.Initialize(&_BasicOutboundChannel.TransactOpts, _configUpdater, _principal, defaultOperators)
}

// Initialize is a paid mutator transaction binding the contract method 0x77a24f36.
//
// Solidity: function initialize(address _configUpdater, address _principal, address[] defaultOperators) returns()
func (_BasicOutboundChannel *BasicOutboundChannelTransactorSession) Initialize(_configUpdater common.Address, _principal common.Address, defaultOperators []common.Address) (*types.Transaction, error) {
	return _BasicOutboundChannel.Contract.Initialize(&_BasicOutboundChannel.TransactOpts, _configUpdater, _principal, defaultOperators)
}

// RenounceRole is a paid mutator transaction binding the contract method 0x36568abe.
//
// Solidity: function renounceRole(bytes32 role, address account) returns()
func (_BasicOutboundChannel *BasicOutboundChannelTransactor) RenounceRole(opts *bind.TransactOpts, role [32]byte, account common.Address) (*types.Transaction, error) {
	return _BasicOutboundChannel.contract.Transact(opts, "renounceRole", role, account)
}

// RenounceRole is a paid mutator transaction binding the contract method 0x36568abe.
//
// Solidity: function renounceRole(bytes32 role, address account) returns()
func (_BasicOutboundChannel *BasicOutboundChannelSession) RenounceRole(role [32]byte, account common.Address) (*types.Transaction, error) {
	return _BasicOutboundChannel.Contract.RenounceRole(&_BasicOutboundChannel.TransactOpts, role, account)
}

// RenounceRole is a paid mutator transaction binding the contract method 0x36568abe.
//
// Solidity: function renounceRole(bytes32 role, address account) returns()
func (_BasicOutboundChannel *BasicOutboundChannelTransactorSession) RenounceRole(role [32]byte, account common.Address) (*types.Transaction, error) {
	return _BasicOutboundChannel.Contract.RenounceRole(&_BasicOutboundChannel.TransactOpts, role, account)
}

// RevokeDefaultOperator is a paid mutator transaction binding the contract method 0x2a7534a3.
//
// Solidity: function revokeDefaultOperator(address operator) returns()
func (_BasicOutboundChannel *BasicOutboundChannelTransactor) RevokeDefaultOperator(opts *bind.TransactOpts, operator common.Address) (*types.Transaction, error) {
	return _BasicOutboundChannel.contract.Transact(opts, "revokeDefaultOperator", operator)
}

// RevokeDefaultOperator is a paid mutator transaction binding the contract method 0x2a7534a3.
//
// Solidity: function revokeDefaultOperator(address operator) returns()
func (_BasicOutboundChannel *BasicOutboundChannelSession) RevokeDefaultOperator(operator common.Address) (*types.Transaction, error) {
	return _BasicOutboundChannel.Contract.RevokeDefaultOperator(&_BasicOutboundChannel.TransactOpts, operator)
}

// RevokeDefaultOperator is a paid mutator transaction binding the contract method 0x2a7534a3.
//
// Solidity: function revokeDefaultOperator(address operator) returns()
func (_BasicOutboundChannel *BasicOutboundChannelTransactorSession) RevokeDefaultOperator(operator common.Address) (*types.Transaction, error) {
	return _BasicOutboundChannel.Contract.RevokeDefaultOperator(&_BasicOutboundChannel.TransactOpts, operator)
}

// RevokeOperator is a paid mutator transaction binding the contract method 0xfad8b32a.
//
// Solidity: function revokeOperator(address operator) returns()
func (_BasicOutboundChannel *BasicOutboundChannelTransactor) RevokeOperator(opts *bind.TransactOpts, operator common.Address) (*types.Transaction, error) {
	return _BasicOutboundChannel.contract.Transact(opts, "revokeOperator", operator)
}

// RevokeOperator is a paid mutator transaction binding the contract method 0xfad8b32a.
//
// Solidity: function revokeOperator(address operator) returns()
func (_BasicOutboundChannel *BasicOutboundChannelSession) RevokeOperator(operator common.Address) (*types.Transaction, error) {
	return _BasicOutboundChannel.Contract.RevokeOperator(&_BasicOutboundChannel.TransactOpts, operator)
}

// RevokeOperator is a paid mutator transaction binding the contract method 0xfad8b32a.
//
// Solidity: function revokeOperator(address operator) returns()
func (_BasicOutboundChannel *BasicOutboundChannelTransactorSession) RevokeOperator(operator common.Address) (*types.Transaction, error) {
	return _BasicOutboundChannel.Contract.RevokeOperator(&_BasicOutboundChannel.TransactOpts, operator)
}

// RevokeRole is a paid mutator transaction binding the contract method 0xd547741f.
//
// Solidity: function revokeRole(bytes32 role, address account) returns()
func (_BasicOutboundChannel *BasicOutboundChannelTransactor) RevokeRole(opts *bind.TransactOpts, role [32]byte, account common.Address) (*types.Transaction, error) {
	return _BasicOutboundChannel.contract.Transact(opts, "revokeRole", role, account)
}

// RevokeRole is a paid mutator transaction binding the contract method 0xd547741f.
//
// Solidity: function revokeRole(bytes32 role, address account) returns()
func (_BasicOutboundChannel *BasicOutboundChannelSession) RevokeRole(role [32]byte, account common.Address) (*types.Transaction, error) {
	return _BasicOutboundChannel.Contract.RevokeRole(&_BasicOutboundChannel.TransactOpts, role, account)
}

// RevokeRole is a paid mutator transaction binding the contract method 0xd547741f.
//
// Solidity: function revokeRole(bytes32 role, address account) returns()
func (_BasicOutboundChannel *BasicOutboundChannelTransactorSession) RevokeRole(role [32]byte, account common.Address) (*types.Transaction, error) {
	return _BasicOutboundChannel.Contract.RevokeRole(&_BasicOutboundChannel.TransactOpts, role, account)
}

// SetPrincipal is a paid mutator transaction binding the contract method 0x847fd5bf.
//
// Solidity: function setPrincipal(address _principal) returns()
func (_BasicOutboundChannel *BasicOutboundChannelTransactor) SetPrincipal(opts *bind.TransactOpts, _principal common.Address) (*types.Transaction, error) {
	return _BasicOutboundChannel.contract.Transact(opts, "setPrincipal", _principal)
}

// SetPrincipal is a paid mutator transaction binding the contract method 0x847fd5bf.
//
// Solidity: function setPrincipal(address _principal) returns()
func (_BasicOutboundChannel *BasicOutboundChannelSession) SetPrincipal(_principal common.Address) (*types.Transaction, error) {
	return _BasicOutboundChannel.Contract.SetPrincipal(&_BasicOutboundChannel.TransactOpts, _principal)
}

// SetPrincipal is a paid mutator transaction binding the contract method 0x847fd5bf.
//
// Solidity: function setPrincipal(address _principal) returns()
func (_BasicOutboundChannel *BasicOutboundChannelTransactorSession) SetPrincipal(_principal common.Address) (*types.Transaction, error) {
	return _BasicOutboundChannel.Contract.SetPrincipal(&_BasicOutboundChannel.TransactOpts, _principal)
}

// Submit is a paid mutator transaction binding the contract method 0x76846edd.
//
// Solidity: function submit(address _origin, bytes _payload) returns()
func (_BasicOutboundChannel *BasicOutboundChannelTransactor) Submit(opts *bind.TransactOpts, _origin common.Address, _payload []byte) (*types.Transaction, error) {
	return _BasicOutboundChannel.contract.Transact(opts, "submit", _origin, _payload)
}

// Submit is a paid mutator transaction binding the contract method 0x76846edd.
//
// Solidity: function submit(address _origin, bytes _payload) returns()
func (_BasicOutboundChannel *BasicOutboundChannelSession) Submit(_origin common.Address, _payload []byte) (*types.Transaction, error) {
	return _BasicOutboundChannel.Contract.Submit(&_BasicOutboundChannel.TransactOpts, _origin, _payload)
}

// Submit is a paid mutator transaction binding the contract method 0x76846edd.
//
// Solidity: function submit(address _origin, bytes _payload) returns()
func (_BasicOutboundChannel *BasicOutboundChannelTransactorSession) Submit(_origin common.Address, _payload []byte) (*types.Transaction, error) {
	return _BasicOutboundChannel.Contract.Submit(&_BasicOutboundChannel.TransactOpts, _origin, _payload)
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

// BasicOutboundChannelOperatorAuthorizedIterator is returned from FilterOperatorAuthorized and is used to iterate over the raw logs and unpacked data for OperatorAuthorized events raised by the BasicOutboundChannel contract.
type BasicOutboundChannelOperatorAuthorizedIterator struct {
	Event *BasicOutboundChannelOperatorAuthorized // Event containing the contract specifics and raw log

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
func (it *BasicOutboundChannelOperatorAuthorizedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(BasicOutboundChannelOperatorAuthorized)
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
		it.Event = new(BasicOutboundChannelOperatorAuthorized)
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
func (it *BasicOutboundChannelOperatorAuthorizedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *BasicOutboundChannelOperatorAuthorizedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// BasicOutboundChannelOperatorAuthorized represents a OperatorAuthorized event raised by the BasicOutboundChannel contract.
type BasicOutboundChannelOperatorAuthorized struct {
	Operator common.Address
	User     common.Address
	Raw      types.Log // Blockchain specific contextual infos
}

// FilterOperatorAuthorized is a free log retrieval operation binding the contract event 0x4d8c877a9def059a7322c328d7394f8640d101d29c811e10268b9b6125e90253.
//
// Solidity: event OperatorAuthorized(address operator, address user)
func (_BasicOutboundChannel *BasicOutboundChannelFilterer) FilterOperatorAuthorized(opts *bind.FilterOpts) (*BasicOutboundChannelOperatorAuthorizedIterator, error) {

	logs, sub, err := _BasicOutboundChannel.contract.FilterLogs(opts, "OperatorAuthorized")
	if err != nil {
		return nil, err
	}
	return &BasicOutboundChannelOperatorAuthorizedIterator{contract: _BasicOutboundChannel.contract, event: "OperatorAuthorized", logs: logs, sub: sub}, nil
}

// WatchOperatorAuthorized is a free log subscription operation binding the contract event 0x4d8c877a9def059a7322c328d7394f8640d101d29c811e10268b9b6125e90253.
//
// Solidity: event OperatorAuthorized(address operator, address user)
func (_BasicOutboundChannel *BasicOutboundChannelFilterer) WatchOperatorAuthorized(opts *bind.WatchOpts, sink chan<- *BasicOutboundChannelOperatorAuthorized) (event.Subscription, error) {

	logs, sub, err := _BasicOutboundChannel.contract.WatchLogs(opts, "OperatorAuthorized")
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(BasicOutboundChannelOperatorAuthorized)
				if err := _BasicOutboundChannel.contract.UnpackLog(event, "OperatorAuthorized", log); err != nil {
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

// ParseOperatorAuthorized is a log parse operation binding the contract event 0x4d8c877a9def059a7322c328d7394f8640d101d29c811e10268b9b6125e90253.
//
// Solidity: event OperatorAuthorized(address operator, address user)
func (_BasicOutboundChannel *BasicOutboundChannelFilterer) ParseOperatorAuthorized(log types.Log) (*BasicOutboundChannelOperatorAuthorized, error) {
	event := new(BasicOutboundChannelOperatorAuthorized)
	if err := _BasicOutboundChannel.contract.UnpackLog(event, "OperatorAuthorized", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// BasicOutboundChannelOperatorRevokedIterator is returned from FilterOperatorRevoked and is used to iterate over the raw logs and unpacked data for OperatorRevoked events raised by the BasicOutboundChannel contract.
type BasicOutboundChannelOperatorRevokedIterator struct {
	Event *BasicOutboundChannelOperatorRevoked // Event containing the contract specifics and raw log

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
func (it *BasicOutboundChannelOperatorRevokedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(BasicOutboundChannelOperatorRevoked)
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
		it.Event = new(BasicOutboundChannelOperatorRevoked)
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
func (it *BasicOutboundChannelOperatorRevokedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *BasicOutboundChannelOperatorRevokedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// BasicOutboundChannelOperatorRevoked represents a OperatorRevoked event raised by the BasicOutboundChannel contract.
type BasicOutboundChannelOperatorRevoked struct {
	Operator common.Address
	User     common.Address
	Raw      types.Log // Blockchain specific contextual infos
}

// FilterOperatorRevoked is a free log retrieval operation binding the contract event 0xa8082fae8d1bd57faeb4dde45721b46afb72c45e72e5deb2a355bd997f6251a9.
//
// Solidity: event OperatorRevoked(address operator, address user)
func (_BasicOutboundChannel *BasicOutboundChannelFilterer) FilterOperatorRevoked(opts *bind.FilterOpts) (*BasicOutboundChannelOperatorRevokedIterator, error) {

	logs, sub, err := _BasicOutboundChannel.contract.FilterLogs(opts, "OperatorRevoked")
	if err != nil {
		return nil, err
	}
	return &BasicOutboundChannelOperatorRevokedIterator{contract: _BasicOutboundChannel.contract, event: "OperatorRevoked", logs: logs, sub: sub}, nil
}

// WatchOperatorRevoked is a free log subscription operation binding the contract event 0xa8082fae8d1bd57faeb4dde45721b46afb72c45e72e5deb2a355bd997f6251a9.
//
// Solidity: event OperatorRevoked(address operator, address user)
func (_BasicOutboundChannel *BasicOutboundChannelFilterer) WatchOperatorRevoked(opts *bind.WatchOpts, sink chan<- *BasicOutboundChannelOperatorRevoked) (event.Subscription, error) {

	logs, sub, err := _BasicOutboundChannel.contract.WatchLogs(opts, "OperatorRevoked")
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(BasicOutboundChannelOperatorRevoked)
				if err := _BasicOutboundChannel.contract.UnpackLog(event, "OperatorRevoked", log); err != nil {
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

// ParseOperatorRevoked is a log parse operation binding the contract event 0xa8082fae8d1bd57faeb4dde45721b46afb72c45e72e5deb2a355bd997f6251a9.
//
// Solidity: event OperatorRevoked(address operator, address user)
func (_BasicOutboundChannel *BasicOutboundChannelFilterer) ParseOperatorRevoked(log types.Log) (*BasicOutboundChannelOperatorRevoked, error) {
	event := new(BasicOutboundChannelOperatorRevoked)
	if err := _BasicOutboundChannel.contract.UnpackLog(event, "OperatorRevoked", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// BasicOutboundChannelRoleAdminChangedIterator is returned from FilterRoleAdminChanged and is used to iterate over the raw logs and unpacked data for RoleAdminChanged events raised by the BasicOutboundChannel contract.
type BasicOutboundChannelRoleAdminChangedIterator struct {
	Event *BasicOutboundChannelRoleAdminChanged // Event containing the contract specifics and raw log

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
func (it *BasicOutboundChannelRoleAdminChangedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(BasicOutboundChannelRoleAdminChanged)
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
		it.Event = new(BasicOutboundChannelRoleAdminChanged)
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
func (it *BasicOutboundChannelRoleAdminChangedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *BasicOutboundChannelRoleAdminChangedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// BasicOutboundChannelRoleAdminChanged represents a RoleAdminChanged event raised by the BasicOutboundChannel contract.
type BasicOutboundChannelRoleAdminChanged struct {
	Role              [32]byte
	PreviousAdminRole [32]byte
	NewAdminRole      [32]byte
	Raw               types.Log // Blockchain specific contextual infos
}

// FilterRoleAdminChanged is a free log retrieval operation binding the contract event 0xbd79b86ffe0ab8e8776151514217cd7cacd52c909f66475c3af44e129f0b00ff.
//
// Solidity: event RoleAdminChanged(bytes32 indexed role, bytes32 indexed previousAdminRole, bytes32 indexed newAdminRole)
func (_BasicOutboundChannel *BasicOutboundChannelFilterer) FilterRoleAdminChanged(opts *bind.FilterOpts, role [][32]byte, previousAdminRole [][32]byte, newAdminRole [][32]byte) (*BasicOutboundChannelRoleAdminChangedIterator, error) {

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

	logs, sub, err := _BasicOutboundChannel.contract.FilterLogs(opts, "RoleAdminChanged", roleRule, previousAdminRoleRule, newAdminRoleRule)
	if err != nil {
		return nil, err
	}
	return &BasicOutboundChannelRoleAdminChangedIterator{contract: _BasicOutboundChannel.contract, event: "RoleAdminChanged", logs: logs, sub: sub}, nil
}

// WatchRoleAdminChanged is a free log subscription operation binding the contract event 0xbd79b86ffe0ab8e8776151514217cd7cacd52c909f66475c3af44e129f0b00ff.
//
// Solidity: event RoleAdminChanged(bytes32 indexed role, bytes32 indexed previousAdminRole, bytes32 indexed newAdminRole)
func (_BasicOutboundChannel *BasicOutboundChannelFilterer) WatchRoleAdminChanged(opts *bind.WatchOpts, sink chan<- *BasicOutboundChannelRoleAdminChanged, role [][32]byte, previousAdminRole [][32]byte, newAdminRole [][32]byte) (event.Subscription, error) {

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

	logs, sub, err := _BasicOutboundChannel.contract.WatchLogs(opts, "RoleAdminChanged", roleRule, previousAdminRoleRule, newAdminRoleRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(BasicOutboundChannelRoleAdminChanged)
				if err := _BasicOutboundChannel.contract.UnpackLog(event, "RoleAdminChanged", log); err != nil {
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
func (_BasicOutboundChannel *BasicOutboundChannelFilterer) ParseRoleAdminChanged(log types.Log) (*BasicOutboundChannelRoleAdminChanged, error) {
	event := new(BasicOutboundChannelRoleAdminChanged)
	if err := _BasicOutboundChannel.contract.UnpackLog(event, "RoleAdminChanged", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// BasicOutboundChannelRoleGrantedIterator is returned from FilterRoleGranted and is used to iterate over the raw logs and unpacked data for RoleGranted events raised by the BasicOutboundChannel contract.
type BasicOutboundChannelRoleGrantedIterator struct {
	Event *BasicOutboundChannelRoleGranted // Event containing the contract specifics and raw log

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
func (it *BasicOutboundChannelRoleGrantedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(BasicOutboundChannelRoleGranted)
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
		it.Event = new(BasicOutboundChannelRoleGranted)
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
func (it *BasicOutboundChannelRoleGrantedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *BasicOutboundChannelRoleGrantedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// BasicOutboundChannelRoleGranted represents a RoleGranted event raised by the BasicOutboundChannel contract.
type BasicOutboundChannelRoleGranted struct {
	Role    [32]byte
	Account common.Address
	Sender  common.Address
	Raw     types.Log // Blockchain specific contextual infos
}

// FilterRoleGranted is a free log retrieval operation binding the contract event 0x2f8788117e7eff1d82e926ec794901d17c78024a50270940304540a733656f0d.
//
// Solidity: event RoleGranted(bytes32 indexed role, address indexed account, address indexed sender)
func (_BasicOutboundChannel *BasicOutboundChannelFilterer) FilterRoleGranted(opts *bind.FilterOpts, role [][32]byte, account []common.Address, sender []common.Address) (*BasicOutboundChannelRoleGrantedIterator, error) {

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

	logs, sub, err := _BasicOutboundChannel.contract.FilterLogs(opts, "RoleGranted", roleRule, accountRule, senderRule)
	if err != nil {
		return nil, err
	}
	return &BasicOutboundChannelRoleGrantedIterator{contract: _BasicOutboundChannel.contract, event: "RoleGranted", logs: logs, sub: sub}, nil
}

// WatchRoleGranted is a free log subscription operation binding the contract event 0x2f8788117e7eff1d82e926ec794901d17c78024a50270940304540a733656f0d.
//
// Solidity: event RoleGranted(bytes32 indexed role, address indexed account, address indexed sender)
func (_BasicOutboundChannel *BasicOutboundChannelFilterer) WatchRoleGranted(opts *bind.WatchOpts, sink chan<- *BasicOutboundChannelRoleGranted, role [][32]byte, account []common.Address, sender []common.Address) (event.Subscription, error) {

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

	logs, sub, err := _BasicOutboundChannel.contract.WatchLogs(opts, "RoleGranted", roleRule, accountRule, senderRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(BasicOutboundChannelRoleGranted)
				if err := _BasicOutboundChannel.contract.UnpackLog(event, "RoleGranted", log); err != nil {
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
func (_BasicOutboundChannel *BasicOutboundChannelFilterer) ParseRoleGranted(log types.Log) (*BasicOutboundChannelRoleGranted, error) {
	event := new(BasicOutboundChannelRoleGranted)
	if err := _BasicOutboundChannel.contract.UnpackLog(event, "RoleGranted", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// BasicOutboundChannelRoleRevokedIterator is returned from FilterRoleRevoked and is used to iterate over the raw logs and unpacked data for RoleRevoked events raised by the BasicOutboundChannel contract.
type BasicOutboundChannelRoleRevokedIterator struct {
	Event *BasicOutboundChannelRoleRevoked // Event containing the contract specifics and raw log

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
func (it *BasicOutboundChannelRoleRevokedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(BasicOutboundChannelRoleRevoked)
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
		it.Event = new(BasicOutboundChannelRoleRevoked)
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
func (it *BasicOutboundChannelRoleRevokedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *BasicOutboundChannelRoleRevokedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// BasicOutboundChannelRoleRevoked represents a RoleRevoked event raised by the BasicOutboundChannel contract.
type BasicOutboundChannelRoleRevoked struct {
	Role    [32]byte
	Account common.Address
	Sender  common.Address
	Raw     types.Log // Blockchain specific contextual infos
}

// FilterRoleRevoked is a free log retrieval operation binding the contract event 0xf6391f5c32d9c69d2a47ea670b442974b53935d1edc7fd64eb21e047a839171b.
//
// Solidity: event RoleRevoked(bytes32 indexed role, address indexed account, address indexed sender)
func (_BasicOutboundChannel *BasicOutboundChannelFilterer) FilterRoleRevoked(opts *bind.FilterOpts, role [][32]byte, account []common.Address, sender []common.Address) (*BasicOutboundChannelRoleRevokedIterator, error) {

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

	logs, sub, err := _BasicOutboundChannel.contract.FilterLogs(opts, "RoleRevoked", roleRule, accountRule, senderRule)
	if err != nil {
		return nil, err
	}
	return &BasicOutboundChannelRoleRevokedIterator{contract: _BasicOutboundChannel.contract, event: "RoleRevoked", logs: logs, sub: sub}, nil
}

// WatchRoleRevoked is a free log subscription operation binding the contract event 0xf6391f5c32d9c69d2a47ea670b442974b53935d1edc7fd64eb21e047a839171b.
//
// Solidity: event RoleRevoked(bytes32 indexed role, address indexed account, address indexed sender)
func (_BasicOutboundChannel *BasicOutboundChannelFilterer) WatchRoleRevoked(opts *bind.WatchOpts, sink chan<- *BasicOutboundChannelRoleRevoked, role [][32]byte, account []common.Address, sender []common.Address) (event.Subscription, error) {

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

	logs, sub, err := _BasicOutboundChannel.contract.WatchLogs(opts, "RoleRevoked", roleRule, accountRule, senderRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(BasicOutboundChannelRoleRevoked)
				if err := _BasicOutboundChannel.contract.UnpackLog(event, "RoleRevoked", log); err != nil {
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
func (_BasicOutboundChannel *BasicOutboundChannelFilterer) ParseRoleRevoked(log types.Log) (*BasicOutboundChannelRoleRevoked, error) {
	event := new(BasicOutboundChannelRoleRevoked)
	if err := _BasicOutboundChannel.contract.UnpackLog(event, "RoleRevoked", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}
