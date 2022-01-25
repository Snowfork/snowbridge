// Code generated - DO NOT EDIT.
// This file is a generated binding and any manual changes will be lost.

package incentivized

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
const IncentivizedOutboundChannelABI = "[{\"inputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"constructor\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":false,\"internalType\":\"uint256\",\"name\":\"oldFee\",\"type\":\"uint256\"},{\"indexed\":false,\"internalType\":\"uint256\",\"name\":\"newFee\",\"type\":\"uint256\"}],\"name\":\"FeeChanged\",\"type\":\"event\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":false,\"internalType\":\"address\",\"name\":\"source\",\"type\":\"address\"},{\"indexed\":false,\"internalType\":\"uint64\",\"name\":\"nonce\",\"type\":\"uint64\"},{\"indexed\":false,\"internalType\":\"uint256\",\"name\":\"fee\",\"type\":\"uint256\"},{\"indexed\":false,\"internalType\":\"bytes\",\"name\":\"payload\",\"type\":\"bytes\"}],\"name\":\"Message\",\"type\":\"event\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":false,\"internalType\":\"address\",\"name\":\"operator\",\"type\":\"address\"},{\"indexed\":false,\"internalType\":\"address\",\"name\":\"user\",\"type\":\"address\"}],\"name\":\"OperatorAuthorized\",\"type\":\"event\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":false,\"internalType\":\"address\",\"name\":\"operator\",\"type\":\"address\"},{\"indexed\":false,\"internalType\":\"address\",\"name\":\"user\",\"type\":\"address\"}],\"name\":\"OperatorRevoked\",\"type\":\"event\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":true,\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\"},{\"indexed\":true,\"internalType\":\"bytes32\",\"name\":\"previousAdminRole\",\"type\":\"bytes32\"},{\"indexed\":true,\"internalType\":\"bytes32\",\"name\":\"newAdminRole\",\"type\":\"bytes32\"}],\"name\":\"RoleAdminChanged\",\"type\":\"event\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":true,\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\"},{\"indexed\":true,\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\"},{\"indexed\":true,\"internalType\":\"address\",\"name\":\"sender\",\"type\":\"address\"}],\"name\":\"RoleGranted\",\"type\":\"event\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":true,\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\"},{\"indexed\":true,\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\"},{\"indexed\":true,\"internalType\":\"address\",\"name\":\"sender\",\"type\":\"address\"}],\"name\":\"RoleRevoked\",\"type\":\"event\"},{\"inputs\":[],\"name\":\"CONFIG_UPDATE_ROLE\",\"outputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"DEFAULT_ADMIN_ROLE\",\"outputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"operator\",\"type\":\"address\"}],\"name\":\"authorizeDefaultOperator\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"operator\",\"type\":\"address\"}],\"name\":\"authorizeOperator\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"fee\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"feeSource\",\"outputs\":[{\"internalType\":\"contractFeeSource\",\"name\":\"\",\"type\":\"address\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\"}],\"name\":\"getRoleAdmin\",\"outputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\"},{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\"}],\"name\":\"grantRole\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\"},{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\"}],\"name\":\"hasRole\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"_configUpdater\",\"type\":\"address\"},{\"internalType\":\"address\",\"name\":\"_feeSource\",\"type\":\"address\"},{\"internalType\":\"address[]\",\"name\":\"defaultOperators\",\"type\":\"address[]\"}],\"name\":\"initialize\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"_operator\",\"type\":\"address\"},{\"internalType\":\"address\",\"name\":\"_origin\",\"type\":\"address\"}],\"name\":\"isOperatorFor\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"nonce\",\"outputs\":[{\"internalType\":\"uint64\",\"name\":\"\",\"type\":\"uint64\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\"},{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\"}],\"name\":\"renounceRole\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"operator\",\"type\":\"address\"}],\"name\":\"revokeDefaultOperator\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"operator\",\"type\":\"address\"}],\"name\":\"revokeOperator\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\"},{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\"}],\"name\":\"revokeRole\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"_amount\",\"type\":\"uint256\"}],\"name\":\"setFee\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"feePayer\",\"type\":\"address\"},{\"internalType\":\"bytes\",\"name\":\"payload\",\"type\":\"bytes\"}],\"name\":\"submit\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"bytes4\",\"name\":\"interfaceId\",\"type\":\"bytes4\"}],\"name\":\"supportsInterface\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\"}],\"stateMutability\":\"view\",\"type\":\"function\"}]"

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

// CONFIGUPDATEROLE is a free data retrieval call binding the contract method 0xa2d6c6e5.
//
// Solidity: function CONFIG_UPDATE_ROLE() view returns(bytes32)
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelCaller) CONFIGUPDATEROLE(opts *bind.CallOpts) ([32]byte, error) {
	var out []interface{}
	err := _IncentivizedOutboundChannel.contract.Call(opts, &out, "CONFIG_UPDATE_ROLE")

	if err != nil {
		return *new([32]byte), err
	}

	out0 := *abi.ConvertType(out[0], new([32]byte)).(*[32]byte)

	return out0, err

}

// CONFIGUPDATEROLE is a free data retrieval call binding the contract method 0xa2d6c6e5.
//
// Solidity: function CONFIG_UPDATE_ROLE() view returns(bytes32)
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelSession) CONFIGUPDATEROLE() ([32]byte, error) {
	return _IncentivizedOutboundChannel.Contract.CONFIGUPDATEROLE(&_IncentivizedOutboundChannel.CallOpts)
}

// CONFIGUPDATEROLE is a free data retrieval call binding the contract method 0xa2d6c6e5.
//
// Solidity: function CONFIG_UPDATE_ROLE() view returns(bytes32)
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelCallerSession) CONFIGUPDATEROLE() ([32]byte, error) {
	return _IncentivizedOutboundChannel.Contract.CONFIGUPDATEROLE(&_IncentivizedOutboundChannel.CallOpts)
}

// DEFAULTADMINROLE is a free data retrieval call binding the contract method 0xa217fddf.
//
// Solidity: function DEFAULT_ADMIN_ROLE() view returns(bytes32)
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelCaller) DEFAULTADMINROLE(opts *bind.CallOpts) ([32]byte, error) {
	var out []interface{}
	err := _IncentivizedOutboundChannel.contract.Call(opts, &out, "DEFAULT_ADMIN_ROLE")

	if err != nil {
		return *new([32]byte), err
	}

	out0 := *abi.ConvertType(out[0], new([32]byte)).(*[32]byte)

	return out0, err

}

// DEFAULTADMINROLE is a free data retrieval call binding the contract method 0xa217fddf.
//
// Solidity: function DEFAULT_ADMIN_ROLE() view returns(bytes32)
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelSession) DEFAULTADMINROLE() ([32]byte, error) {
	return _IncentivizedOutboundChannel.Contract.DEFAULTADMINROLE(&_IncentivizedOutboundChannel.CallOpts)
}

// DEFAULTADMINROLE is a free data retrieval call binding the contract method 0xa217fddf.
//
// Solidity: function DEFAULT_ADMIN_ROLE() view returns(bytes32)
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelCallerSession) DEFAULTADMINROLE() ([32]byte, error) {
	return _IncentivizedOutboundChannel.Contract.DEFAULTADMINROLE(&_IncentivizedOutboundChannel.CallOpts)
}

// Fee is a free data retrieval call binding the contract method 0xddca3f43.
//
// Solidity: function fee() view returns(uint256)
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelCaller) Fee(opts *bind.CallOpts) (*big.Int, error) {
	var out []interface{}
	err := _IncentivizedOutboundChannel.contract.Call(opts, &out, "fee")

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// Fee is a free data retrieval call binding the contract method 0xddca3f43.
//
// Solidity: function fee() view returns(uint256)
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelSession) Fee() (*big.Int, error) {
	return _IncentivizedOutboundChannel.Contract.Fee(&_IncentivizedOutboundChannel.CallOpts)
}

// Fee is a free data retrieval call binding the contract method 0xddca3f43.
//
// Solidity: function fee() view returns(uint256)
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelCallerSession) Fee() (*big.Int, error) {
	return _IncentivizedOutboundChannel.Contract.Fee(&_IncentivizedOutboundChannel.CallOpts)
}

// FeeSource is a free data retrieval call binding the contract method 0xb47a9bfa.
//
// Solidity: function feeSource() view returns(address)
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelCaller) FeeSource(opts *bind.CallOpts) (common.Address, error) {
	var out []interface{}
	err := _IncentivizedOutboundChannel.contract.Call(opts, &out, "feeSource")

	if err != nil {
		return *new(common.Address), err
	}

	out0 := *abi.ConvertType(out[0], new(common.Address)).(*common.Address)

	return out0, err

}

// FeeSource is a free data retrieval call binding the contract method 0xb47a9bfa.
//
// Solidity: function feeSource() view returns(address)
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelSession) FeeSource() (common.Address, error) {
	return _IncentivizedOutboundChannel.Contract.FeeSource(&_IncentivizedOutboundChannel.CallOpts)
}

// FeeSource is a free data retrieval call binding the contract method 0xb47a9bfa.
//
// Solidity: function feeSource() view returns(address)
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelCallerSession) FeeSource() (common.Address, error) {
	return _IncentivizedOutboundChannel.Contract.FeeSource(&_IncentivizedOutboundChannel.CallOpts)
}

// GetRoleAdmin is a free data retrieval call binding the contract method 0x248a9ca3.
//
// Solidity: function getRoleAdmin(bytes32 role) view returns(bytes32)
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelCaller) GetRoleAdmin(opts *bind.CallOpts, role [32]byte) ([32]byte, error) {
	var out []interface{}
	err := _IncentivizedOutboundChannel.contract.Call(opts, &out, "getRoleAdmin", role)

	if err != nil {
		return *new([32]byte), err
	}

	out0 := *abi.ConvertType(out[0], new([32]byte)).(*[32]byte)

	return out0, err

}

// GetRoleAdmin is a free data retrieval call binding the contract method 0x248a9ca3.
//
// Solidity: function getRoleAdmin(bytes32 role) view returns(bytes32)
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelSession) GetRoleAdmin(role [32]byte) ([32]byte, error) {
	return _IncentivizedOutboundChannel.Contract.GetRoleAdmin(&_IncentivizedOutboundChannel.CallOpts, role)
}

// GetRoleAdmin is a free data retrieval call binding the contract method 0x248a9ca3.
//
// Solidity: function getRoleAdmin(bytes32 role) view returns(bytes32)
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelCallerSession) GetRoleAdmin(role [32]byte) ([32]byte, error) {
	return _IncentivizedOutboundChannel.Contract.GetRoleAdmin(&_IncentivizedOutboundChannel.CallOpts, role)
}

// HasRole is a free data retrieval call binding the contract method 0x91d14854.
//
// Solidity: function hasRole(bytes32 role, address account) view returns(bool)
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelCaller) HasRole(opts *bind.CallOpts, role [32]byte, account common.Address) (bool, error) {
	var out []interface{}
	err := _IncentivizedOutboundChannel.contract.Call(opts, &out, "hasRole", role, account)

	if err != nil {
		return *new(bool), err
	}

	out0 := *abi.ConvertType(out[0], new(bool)).(*bool)

	return out0, err

}

// HasRole is a free data retrieval call binding the contract method 0x91d14854.
//
// Solidity: function hasRole(bytes32 role, address account) view returns(bool)
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelSession) HasRole(role [32]byte, account common.Address) (bool, error) {
	return _IncentivizedOutboundChannel.Contract.HasRole(&_IncentivizedOutboundChannel.CallOpts, role, account)
}

// HasRole is a free data retrieval call binding the contract method 0x91d14854.
//
// Solidity: function hasRole(bytes32 role, address account) view returns(bool)
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelCallerSession) HasRole(role [32]byte, account common.Address) (bool, error) {
	return _IncentivizedOutboundChannel.Contract.HasRole(&_IncentivizedOutboundChannel.CallOpts, role, account)
}

// IsOperatorFor is a free data retrieval call binding the contract method 0xd95b6371.
//
// Solidity: function isOperatorFor(address _operator, address _origin) view returns(bool)
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelCaller) IsOperatorFor(opts *bind.CallOpts, _operator common.Address, _origin common.Address) (bool, error) {
	var out []interface{}
	err := _IncentivizedOutboundChannel.contract.Call(opts, &out, "isOperatorFor", _operator, _origin)

	if err != nil {
		return *new(bool), err
	}

	out0 := *abi.ConvertType(out[0], new(bool)).(*bool)

	return out0, err

}

// IsOperatorFor is a free data retrieval call binding the contract method 0xd95b6371.
//
// Solidity: function isOperatorFor(address _operator, address _origin) view returns(bool)
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelSession) IsOperatorFor(_operator common.Address, _origin common.Address) (bool, error) {
	return _IncentivizedOutboundChannel.Contract.IsOperatorFor(&_IncentivizedOutboundChannel.CallOpts, _operator, _origin)
}

// IsOperatorFor is a free data retrieval call binding the contract method 0xd95b6371.
//
// Solidity: function isOperatorFor(address _operator, address _origin) view returns(bool)
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelCallerSession) IsOperatorFor(_operator common.Address, _origin common.Address) (bool, error) {
	return _IncentivizedOutboundChannel.Contract.IsOperatorFor(&_IncentivizedOutboundChannel.CallOpts, _operator, _origin)
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

// SupportsInterface is a free data retrieval call binding the contract method 0x01ffc9a7.
//
// Solidity: function supportsInterface(bytes4 interfaceId) view returns(bool)
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelCaller) SupportsInterface(opts *bind.CallOpts, interfaceId [4]byte) (bool, error) {
	var out []interface{}
	err := _IncentivizedOutboundChannel.contract.Call(opts, &out, "supportsInterface", interfaceId)

	if err != nil {
		return *new(bool), err
	}

	out0 := *abi.ConvertType(out[0], new(bool)).(*bool)

	return out0, err

}

// SupportsInterface is a free data retrieval call binding the contract method 0x01ffc9a7.
//
// Solidity: function supportsInterface(bytes4 interfaceId) view returns(bool)
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelSession) SupportsInterface(interfaceId [4]byte) (bool, error) {
	return _IncentivizedOutboundChannel.Contract.SupportsInterface(&_IncentivizedOutboundChannel.CallOpts, interfaceId)
}

// SupportsInterface is a free data retrieval call binding the contract method 0x01ffc9a7.
//
// Solidity: function supportsInterface(bytes4 interfaceId) view returns(bool)
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelCallerSession) SupportsInterface(interfaceId [4]byte) (bool, error) {
	return _IncentivizedOutboundChannel.Contract.SupportsInterface(&_IncentivizedOutboundChannel.CallOpts, interfaceId)
}

// AuthorizeDefaultOperator is a paid mutator transaction binding the contract method 0xb742a404.
//
// Solidity: function authorizeDefaultOperator(address operator) returns()
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelTransactor) AuthorizeDefaultOperator(opts *bind.TransactOpts, operator common.Address) (*types.Transaction, error) {
	return _IncentivizedOutboundChannel.contract.Transact(opts, "authorizeDefaultOperator", operator)
}

// AuthorizeDefaultOperator is a paid mutator transaction binding the contract method 0xb742a404.
//
// Solidity: function authorizeDefaultOperator(address operator) returns()
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelSession) AuthorizeDefaultOperator(operator common.Address) (*types.Transaction, error) {
	return _IncentivizedOutboundChannel.Contract.AuthorizeDefaultOperator(&_IncentivizedOutboundChannel.TransactOpts, operator)
}

// AuthorizeDefaultOperator is a paid mutator transaction binding the contract method 0xb742a404.
//
// Solidity: function authorizeDefaultOperator(address operator) returns()
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelTransactorSession) AuthorizeDefaultOperator(operator common.Address) (*types.Transaction, error) {
	return _IncentivizedOutboundChannel.Contract.AuthorizeDefaultOperator(&_IncentivizedOutboundChannel.TransactOpts, operator)
}

// AuthorizeOperator is a paid mutator transaction binding the contract method 0x959b8c3f.
//
// Solidity: function authorizeOperator(address operator) returns()
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelTransactor) AuthorizeOperator(opts *bind.TransactOpts, operator common.Address) (*types.Transaction, error) {
	return _IncentivizedOutboundChannel.contract.Transact(opts, "authorizeOperator", operator)
}

// AuthorizeOperator is a paid mutator transaction binding the contract method 0x959b8c3f.
//
// Solidity: function authorizeOperator(address operator) returns()
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelSession) AuthorizeOperator(operator common.Address) (*types.Transaction, error) {
	return _IncentivizedOutboundChannel.Contract.AuthorizeOperator(&_IncentivizedOutboundChannel.TransactOpts, operator)
}

// AuthorizeOperator is a paid mutator transaction binding the contract method 0x959b8c3f.
//
// Solidity: function authorizeOperator(address operator) returns()
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelTransactorSession) AuthorizeOperator(operator common.Address) (*types.Transaction, error) {
	return _IncentivizedOutboundChannel.Contract.AuthorizeOperator(&_IncentivizedOutboundChannel.TransactOpts, operator)
}

// GrantRole is a paid mutator transaction binding the contract method 0x2f2ff15d.
//
// Solidity: function grantRole(bytes32 role, address account) returns()
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelTransactor) GrantRole(opts *bind.TransactOpts, role [32]byte, account common.Address) (*types.Transaction, error) {
	return _IncentivizedOutboundChannel.contract.Transact(opts, "grantRole", role, account)
}

// GrantRole is a paid mutator transaction binding the contract method 0x2f2ff15d.
//
// Solidity: function grantRole(bytes32 role, address account) returns()
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelSession) GrantRole(role [32]byte, account common.Address) (*types.Transaction, error) {
	return _IncentivizedOutboundChannel.Contract.GrantRole(&_IncentivizedOutboundChannel.TransactOpts, role, account)
}

// GrantRole is a paid mutator transaction binding the contract method 0x2f2ff15d.
//
// Solidity: function grantRole(bytes32 role, address account) returns()
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelTransactorSession) GrantRole(role [32]byte, account common.Address) (*types.Transaction, error) {
	return _IncentivizedOutboundChannel.Contract.GrantRole(&_IncentivizedOutboundChannel.TransactOpts, role, account)
}

// Initialize is a paid mutator transaction binding the contract method 0x77a24f36.
//
// Solidity: function initialize(address _configUpdater, address _feeSource, address[] defaultOperators) returns()
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelTransactor) Initialize(opts *bind.TransactOpts, _configUpdater common.Address, _feeSource common.Address, defaultOperators []common.Address) (*types.Transaction, error) {
	return _IncentivizedOutboundChannel.contract.Transact(opts, "initialize", _configUpdater, _feeSource, defaultOperators)
}

// Initialize is a paid mutator transaction binding the contract method 0x77a24f36.
//
// Solidity: function initialize(address _configUpdater, address _feeSource, address[] defaultOperators) returns()
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelSession) Initialize(_configUpdater common.Address, _feeSource common.Address, defaultOperators []common.Address) (*types.Transaction, error) {
	return _IncentivizedOutboundChannel.Contract.Initialize(&_IncentivizedOutboundChannel.TransactOpts, _configUpdater, _feeSource, defaultOperators)
}

// Initialize is a paid mutator transaction binding the contract method 0x77a24f36.
//
// Solidity: function initialize(address _configUpdater, address _feeSource, address[] defaultOperators) returns()
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelTransactorSession) Initialize(_configUpdater common.Address, _feeSource common.Address, defaultOperators []common.Address) (*types.Transaction, error) {
	return _IncentivizedOutboundChannel.Contract.Initialize(&_IncentivizedOutboundChannel.TransactOpts, _configUpdater, _feeSource, defaultOperators)
}

// RenounceRole is a paid mutator transaction binding the contract method 0x36568abe.
//
// Solidity: function renounceRole(bytes32 role, address account) returns()
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelTransactor) RenounceRole(opts *bind.TransactOpts, role [32]byte, account common.Address) (*types.Transaction, error) {
	return _IncentivizedOutboundChannel.contract.Transact(opts, "renounceRole", role, account)
}

// RenounceRole is a paid mutator transaction binding the contract method 0x36568abe.
//
// Solidity: function renounceRole(bytes32 role, address account) returns()
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelSession) RenounceRole(role [32]byte, account common.Address) (*types.Transaction, error) {
	return _IncentivizedOutboundChannel.Contract.RenounceRole(&_IncentivizedOutboundChannel.TransactOpts, role, account)
}

// RenounceRole is a paid mutator transaction binding the contract method 0x36568abe.
//
// Solidity: function renounceRole(bytes32 role, address account) returns()
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelTransactorSession) RenounceRole(role [32]byte, account common.Address) (*types.Transaction, error) {
	return _IncentivizedOutboundChannel.Contract.RenounceRole(&_IncentivizedOutboundChannel.TransactOpts, role, account)
}

// RevokeDefaultOperator is a paid mutator transaction binding the contract method 0x2a7534a3.
//
// Solidity: function revokeDefaultOperator(address operator) returns()
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelTransactor) RevokeDefaultOperator(opts *bind.TransactOpts, operator common.Address) (*types.Transaction, error) {
	return _IncentivizedOutboundChannel.contract.Transact(opts, "revokeDefaultOperator", operator)
}

// RevokeDefaultOperator is a paid mutator transaction binding the contract method 0x2a7534a3.
//
// Solidity: function revokeDefaultOperator(address operator) returns()
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelSession) RevokeDefaultOperator(operator common.Address) (*types.Transaction, error) {
	return _IncentivizedOutboundChannel.Contract.RevokeDefaultOperator(&_IncentivizedOutboundChannel.TransactOpts, operator)
}

// RevokeDefaultOperator is a paid mutator transaction binding the contract method 0x2a7534a3.
//
// Solidity: function revokeDefaultOperator(address operator) returns()
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelTransactorSession) RevokeDefaultOperator(operator common.Address) (*types.Transaction, error) {
	return _IncentivizedOutboundChannel.Contract.RevokeDefaultOperator(&_IncentivizedOutboundChannel.TransactOpts, operator)
}

// RevokeOperator is a paid mutator transaction binding the contract method 0xfad8b32a.
//
// Solidity: function revokeOperator(address operator) returns()
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelTransactor) RevokeOperator(opts *bind.TransactOpts, operator common.Address) (*types.Transaction, error) {
	return _IncentivizedOutboundChannel.contract.Transact(opts, "revokeOperator", operator)
}

// RevokeOperator is a paid mutator transaction binding the contract method 0xfad8b32a.
//
// Solidity: function revokeOperator(address operator) returns()
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelSession) RevokeOperator(operator common.Address) (*types.Transaction, error) {
	return _IncentivizedOutboundChannel.Contract.RevokeOperator(&_IncentivizedOutboundChannel.TransactOpts, operator)
}

// RevokeOperator is a paid mutator transaction binding the contract method 0xfad8b32a.
//
// Solidity: function revokeOperator(address operator) returns()
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelTransactorSession) RevokeOperator(operator common.Address) (*types.Transaction, error) {
	return _IncentivizedOutboundChannel.Contract.RevokeOperator(&_IncentivizedOutboundChannel.TransactOpts, operator)
}

// RevokeRole is a paid mutator transaction binding the contract method 0xd547741f.
//
// Solidity: function revokeRole(bytes32 role, address account) returns()
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelTransactor) RevokeRole(opts *bind.TransactOpts, role [32]byte, account common.Address) (*types.Transaction, error) {
	return _IncentivizedOutboundChannel.contract.Transact(opts, "revokeRole", role, account)
}

// RevokeRole is a paid mutator transaction binding the contract method 0xd547741f.
//
// Solidity: function revokeRole(bytes32 role, address account) returns()
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelSession) RevokeRole(role [32]byte, account common.Address) (*types.Transaction, error) {
	return _IncentivizedOutboundChannel.Contract.RevokeRole(&_IncentivizedOutboundChannel.TransactOpts, role, account)
}

// RevokeRole is a paid mutator transaction binding the contract method 0xd547741f.
//
// Solidity: function revokeRole(bytes32 role, address account) returns()
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelTransactorSession) RevokeRole(role [32]byte, account common.Address) (*types.Transaction, error) {
	return _IncentivizedOutboundChannel.Contract.RevokeRole(&_IncentivizedOutboundChannel.TransactOpts, role, account)
}

// SetFee is a paid mutator transaction binding the contract method 0x69fe0e2d.
//
// Solidity: function setFee(uint256 _amount) returns()
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelTransactor) SetFee(opts *bind.TransactOpts, _amount *big.Int) (*types.Transaction, error) {
	return _IncentivizedOutboundChannel.contract.Transact(opts, "setFee", _amount)
}

// SetFee is a paid mutator transaction binding the contract method 0x69fe0e2d.
//
// Solidity: function setFee(uint256 _amount) returns()
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelSession) SetFee(_amount *big.Int) (*types.Transaction, error) {
	return _IncentivizedOutboundChannel.Contract.SetFee(&_IncentivizedOutboundChannel.TransactOpts, _amount)
}

// SetFee is a paid mutator transaction binding the contract method 0x69fe0e2d.
//
// Solidity: function setFee(uint256 _amount) returns()
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelTransactorSession) SetFee(_amount *big.Int) (*types.Transaction, error) {
	return _IncentivizedOutboundChannel.Contract.SetFee(&_IncentivizedOutboundChannel.TransactOpts, _amount)
}

// Submit is a paid mutator transaction binding the contract method 0x76846edd.
//
// Solidity: function submit(address feePayer, bytes payload) returns()
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelTransactor) Submit(opts *bind.TransactOpts, feePayer common.Address, payload []byte) (*types.Transaction, error) {
	return _IncentivizedOutboundChannel.contract.Transact(opts, "submit", feePayer, payload)
}

// Submit is a paid mutator transaction binding the contract method 0x76846edd.
//
// Solidity: function submit(address feePayer, bytes payload) returns()
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelSession) Submit(feePayer common.Address, payload []byte) (*types.Transaction, error) {
	return _IncentivizedOutboundChannel.Contract.Submit(&_IncentivizedOutboundChannel.TransactOpts, feePayer, payload)
}

// Submit is a paid mutator transaction binding the contract method 0x76846edd.
//
// Solidity: function submit(address feePayer, bytes payload) returns()
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelTransactorSession) Submit(feePayer common.Address, payload []byte) (*types.Transaction, error) {
	return _IncentivizedOutboundChannel.Contract.Submit(&_IncentivizedOutboundChannel.TransactOpts, feePayer, payload)
}

// IncentivizedOutboundChannelFeeChangedIterator is returned from FilterFeeChanged and is used to iterate over the raw logs and unpacked data for FeeChanged events raised by the IncentivizedOutboundChannel contract.
type IncentivizedOutboundChannelFeeChangedIterator struct {
	Event *IncentivizedOutboundChannelFeeChanged // Event containing the contract specifics and raw log

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
func (it *IncentivizedOutboundChannelFeeChangedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(IncentivizedOutboundChannelFeeChanged)
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
		it.Event = new(IncentivizedOutboundChannelFeeChanged)
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
func (it *IncentivizedOutboundChannelFeeChangedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *IncentivizedOutboundChannelFeeChangedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// IncentivizedOutboundChannelFeeChanged represents a FeeChanged event raised by the IncentivizedOutboundChannel contract.
type IncentivizedOutboundChannelFeeChanged struct {
	OldFee *big.Int
	NewFee *big.Int
	Raw    types.Log // Blockchain specific contextual infos
}

// FilterFeeChanged is a free log retrieval operation binding the contract event 0x5fc463da23c1b063e66f9e352006a7fbe8db7223c455dc429e881a2dfe2f94f1.
//
// Solidity: event FeeChanged(uint256 oldFee, uint256 newFee)
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelFilterer) FilterFeeChanged(opts *bind.FilterOpts) (*IncentivizedOutboundChannelFeeChangedIterator, error) {

	logs, sub, err := _IncentivizedOutboundChannel.contract.FilterLogs(opts, "FeeChanged")
	if err != nil {
		return nil, err
	}
	return &IncentivizedOutboundChannelFeeChangedIterator{contract: _IncentivizedOutboundChannel.contract, event: "FeeChanged", logs: logs, sub: sub}, nil
}

// WatchFeeChanged is a free log subscription operation binding the contract event 0x5fc463da23c1b063e66f9e352006a7fbe8db7223c455dc429e881a2dfe2f94f1.
//
// Solidity: event FeeChanged(uint256 oldFee, uint256 newFee)
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelFilterer) WatchFeeChanged(opts *bind.WatchOpts, sink chan<- *IncentivizedOutboundChannelFeeChanged) (event.Subscription, error) {

	logs, sub, err := _IncentivizedOutboundChannel.contract.WatchLogs(opts, "FeeChanged")
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(IncentivizedOutboundChannelFeeChanged)
				if err := _IncentivizedOutboundChannel.contract.UnpackLog(event, "FeeChanged", log); err != nil {
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

// ParseFeeChanged is a log parse operation binding the contract event 0x5fc463da23c1b063e66f9e352006a7fbe8db7223c455dc429e881a2dfe2f94f1.
//
// Solidity: event FeeChanged(uint256 oldFee, uint256 newFee)
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelFilterer) ParseFeeChanged(log types.Log) (*IncentivizedOutboundChannelFeeChanged, error) {
	event := new(IncentivizedOutboundChannelFeeChanged)
	if err := _IncentivizedOutboundChannel.contract.UnpackLog(event, "FeeChanged", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
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
	Fee     *big.Int
	Payload []byte
	Raw     types.Log // Blockchain specific contextual infos
}

// FilterMessage is a free log retrieval operation binding the contract event 0x5e9ae1d7c484f74d554a503aa825e823725531d97e784dd9b1aacdb58d1f7076.
//
// Solidity: event Message(address source, uint64 nonce, uint256 fee, bytes payload)
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelFilterer) FilterMessage(opts *bind.FilterOpts) (*IncentivizedOutboundChannelMessageIterator, error) {

	logs, sub, err := _IncentivizedOutboundChannel.contract.FilterLogs(opts, "Message")
	if err != nil {
		return nil, err
	}
	return &IncentivizedOutboundChannelMessageIterator{contract: _IncentivizedOutboundChannel.contract, event: "Message", logs: logs, sub: sub}, nil
}

// WatchMessage is a free log subscription operation binding the contract event 0x5e9ae1d7c484f74d554a503aa825e823725531d97e784dd9b1aacdb58d1f7076.
//
// Solidity: event Message(address source, uint64 nonce, uint256 fee, bytes payload)
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

// ParseMessage is a log parse operation binding the contract event 0x5e9ae1d7c484f74d554a503aa825e823725531d97e784dd9b1aacdb58d1f7076.
//
// Solidity: event Message(address source, uint64 nonce, uint256 fee, bytes payload)
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelFilterer) ParseMessage(log types.Log) (*IncentivizedOutboundChannelMessage, error) {
	event := new(IncentivizedOutboundChannelMessage)
	if err := _IncentivizedOutboundChannel.contract.UnpackLog(event, "Message", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// IncentivizedOutboundChannelOperatorAuthorizedIterator is returned from FilterOperatorAuthorized and is used to iterate over the raw logs and unpacked data for OperatorAuthorized events raised by the IncentivizedOutboundChannel contract.
type IncentivizedOutboundChannelOperatorAuthorizedIterator struct {
	Event *IncentivizedOutboundChannelOperatorAuthorized // Event containing the contract specifics and raw log

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
func (it *IncentivizedOutboundChannelOperatorAuthorizedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(IncentivizedOutboundChannelOperatorAuthorized)
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
		it.Event = new(IncentivizedOutboundChannelOperatorAuthorized)
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
func (it *IncentivizedOutboundChannelOperatorAuthorizedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *IncentivizedOutboundChannelOperatorAuthorizedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// IncentivizedOutboundChannelOperatorAuthorized represents a OperatorAuthorized event raised by the IncentivizedOutboundChannel contract.
type IncentivizedOutboundChannelOperatorAuthorized struct {
	Operator common.Address
	User     common.Address
	Raw      types.Log // Blockchain specific contextual infos
}

// FilterOperatorAuthorized is a free log retrieval operation binding the contract event 0x4d8c877a9def059a7322c328d7394f8640d101d29c811e10268b9b6125e90253.
//
// Solidity: event OperatorAuthorized(address operator, address user)
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelFilterer) FilterOperatorAuthorized(opts *bind.FilterOpts) (*IncentivizedOutboundChannelOperatorAuthorizedIterator, error) {

	logs, sub, err := _IncentivizedOutboundChannel.contract.FilterLogs(opts, "OperatorAuthorized")
	if err != nil {
		return nil, err
	}
	return &IncentivizedOutboundChannelOperatorAuthorizedIterator{contract: _IncentivizedOutboundChannel.contract, event: "OperatorAuthorized", logs: logs, sub: sub}, nil
}

// WatchOperatorAuthorized is a free log subscription operation binding the contract event 0x4d8c877a9def059a7322c328d7394f8640d101d29c811e10268b9b6125e90253.
//
// Solidity: event OperatorAuthorized(address operator, address user)
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelFilterer) WatchOperatorAuthorized(opts *bind.WatchOpts, sink chan<- *IncentivizedOutboundChannelOperatorAuthorized) (event.Subscription, error) {

	logs, sub, err := _IncentivizedOutboundChannel.contract.WatchLogs(opts, "OperatorAuthorized")
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(IncentivizedOutboundChannelOperatorAuthorized)
				if err := _IncentivizedOutboundChannel.contract.UnpackLog(event, "OperatorAuthorized", log); err != nil {
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
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelFilterer) ParseOperatorAuthorized(log types.Log) (*IncentivizedOutboundChannelOperatorAuthorized, error) {
	event := new(IncentivizedOutboundChannelOperatorAuthorized)
	if err := _IncentivizedOutboundChannel.contract.UnpackLog(event, "OperatorAuthorized", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// IncentivizedOutboundChannelOperatorRevokedIterator is returned from FilterOperatorRevoked and is used to iterate over the raw logs and unpacked data for OperatorRevoked events raised by the IncentivizedOutboundChannel contract.
type IncentivizedOutboundChannelOperatorRevokedIterator struct {
	Event *IncentivizedOutboundChannelOperatorRevoked // Event containing the contract specifics and raw log

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
func (it *IncentivizedOutboundChannelOperatorRevokedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(IncentivizedOutboundChannelOperatorRevoked)
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
		it.Event = new(IncentivizedOutboundChannelOperatorRevoked)
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
func (it *IncentivizedOutboundChannelOperatorRevokedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *IncentivizedOutboundChannelOperatorRevokedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// IncentivizedOutboundChannelOperatorRevoked represents a OperatorRevoked event raised by the IncentivizedOutboundChannel contract.
type IncentivizedOutboundChannelOperatorRevoked struct {
	Operator common.Address
	User     common.Address
	Raw      types.Log // Blockchain specific contextual infos
}

// FilterOperatorRevoked is a free log retrieval operation binding the contract event 0xa8082fae8d1bd57faeb4dde45721b46afb72c45e72e5deb2a355bd997f6251a9.
//
// Solidity: event OperatorRevoked(address operator, address user)
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelFilterer) FilterOperatorRevoked(opts *bind.FilterOpts) (*IncentivizedOutboundChannelOperatorRevokedIterator, error) {

	logs, sub, err := _IncentivizedOutboundChannel.contract.FilterLogs(opts, "OperatorRevoked")
	if err != nil {
		return nil, err
	}
	return &IncentivizedOutboundChannelOperatorRevokedIterator{contract: _IncentivizedOutboundChannel.contract, event: "OperatorRevoked", logs: logs, sub: sub}, nil
}

// WatchOperatorRevoked is a free log subscription operation binding the contract event 0xa8082fae8d1bd57faeb4dde45721b46afb72c45e72e5deb2a355bd997f6251a9.
//
// Solidity: event OperatorRevoked(address operator, address user)
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelFilterer) WatchOperatorRevoked(opts *bind.WatchOpts, sink chan<- *IncentivizedOutboundChannelOperatorRevoked) (event.Subscription, error) {

	logs, sub, err := _IncentivizedOutboundChannel.contract.WatchLogs(opts, "OperatorRevoked")
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(IncentivizedOutboundChannelOperatorRevoked)
				if err := _IncentivizedOutboundChannel.contract.UnpackLog(event, "OperatorRevoked", log); err != nil {
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
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelFilterer) ParseOperatorRevoked(log types.Log) (*IncentivizedOutboundChannelOperatorRevoked, error) {
	event := new(IncentivizedOutboundChannelOperatorRevoked)
	if err := _IncentivizedOutboundChannel.contract.UnpackLog(event, "OperatorRevoked", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// IncentivizedOutboundChannelRoleAdminChangedIterator is returned from FilterRoleAdminChanged and is used to iterate over the raw logs and unpacked data for RoleAdminChanged events raised by the IncentivizedOutboundChannel contract.
type IncentivizedOutboundChannelRoleAdminChangedIterator struct {
	Event *IncentivizedOutboundChannelRoleAdminChanged // Event containing the contract specifics and raw log

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
func (it *IncentivizedOutboundChannelRoleAdminChangedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(IncentivizedOutboundChannelRoleAdminChanged)
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
		it.Event = new(IncentivizedOutboundChannelRoleAdminChanged)
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
func (it *IncentivizedOutboundChannelRoleAdminChangedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *IncentivizedOutboundChannelRoleAdminChangedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// IncentivizedOutboundChannelRoleAdminChanged represents a RoleAdminChanged event raised by the IncentivizedOutboundChannel contract.
type IncentivizedOutboundChannelRoleAdminChanged struct {
	Role              [32]byte
	PreviousAdminRole [32]byte
	NewAdminRole      [32]byte
	Raw               types.Log // Blockchain specific contextual infos
}

// FilterRoleAdminChanged is a free log retrieval operation binding the contract event 0xbd79b86ffe0ab8e8776151514217cd7cacd52c909f66475c3af44e129f0b00ff.
//
// Solidity: event RoleAdminChanged(bytes32 indexed role, bytes32 indexed previousAdminRole, bytes32 indexed newAdminRole)
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelFilterer) FilterRoleAdminChanged(opts *bind.FilterOpts, role [][32]byte, previousAdminRole [][32]byte, newAdminRole [][32]byte) (*IncentivizedOutboundChannelRoleAdminChangedIterator, error) {

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

	logs, sub, err := _IncentivizedOutboundChannel.contract.FilterLogs(opts, "RoleAdminChanged", roleRule, previousAdminRoleRule, newAdminRoleRule)
	if err != nil {
		return nil, err
	}
	return &IncentivizedOutboundChannelRoleAdminChangedIterator{contract: _IncentivizedOutboundChannel.contract, event: "RoleAdminChanged", logs: logs, sub: sub}, nil
}

// WatchRoleAdminChanged is a free log subscription operation binding the contract event 0xbd79b86ffe0ab8e8776151514217cd7cacd52c909f66475c3af44e129f0b00ff.
//
// Solidity: event RoleAdminChanged(bytes32 indexed role, bytes32 indexed previousAdminRole, bytes32 indexed newAdminRole)
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelFilterer) WatchRoleAdminChanged(opts *bind.WatchOpts, sink chan<- *IncentivizedOutboundChannelRoleAdminChanged, role [][32]byte, previousAdminRole [][32]byte, newAdminRole [][32]byte) (event.Subscription, error) {

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

	logs, sub, err := _IncentivizedOutboundChannel.contract.WatchLogs(opts, "RoleAdminChanged", roleRule, previousAdminRoleRule, newAdminRoleRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(IncentivizedOutboundChannelRoleAdminChanged)
				if err := _IncentivizedOutboundChannel.contract.UnpackLog(event, "RoleAdminChanged", log); err != nil {
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
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelFilterer) ParseRoleAdminChanged(log types.Log) (*IncentivizedOutboundChannelRoleAdminChanged, error) {
	event := new(IncentivizedOutboundChannelRoleAdminChanged)
	if err := _IncentivizedOutboundChannel.contract.UnpackLog(event, "RoleAdminChanged", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// IncentivizedOutboundChannelRoleGrantedIterator is returned from FilterRoleGranted and is used to iterate over the raw logs and unpacked data for RoleGranted events raised by the IncentivizedOutboundChannel contract.
type IncentivizedOutboundChannelRoleGrantedIterator struct {
	Event *IncentivizedOutboundChannelRoleGranted // Event containing the contract specifics and raw log

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
func (it *IncentivizedOutboundChannelRoleGrantedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(IncentivizedOutboundChannelRoleGranted)
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
		it.Event = new(IncentivizedOutboundChannelRoleGranted)
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
func (it *IncentivizedOutboundChannelRoleGrantedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *IncentivizedOutboundChannelRoleGrantedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// IncentivizedOutboundChannelRoleGranted represents a RoleGranted event raised by the IncentivizedOutboundChannel contract.
type IncentivizedOutboundChannelRoleGranted struct {
	Role    [32]byte
	Account common.Address
	Sender  common.Address
	Raw     types.Log // Blockchain specific contextual infos
}

// FilterRoleGranted is a free log retrieval operation binding the contract event 0x2f8788117e7eff1d82e926ec794901d17c78024a50270940304540a733656f0d.
//
// Solidity: event RoleGranted(bytes32 indexed role, address indexed account, address indexed sender)
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelFilterer) FilterRoleGranted(opts *bind.FilterOpts, role [][32]byte, account []common.Address, sender []common.Address) (*IncentivizedOutboundChannelRoleGrantedIterator, error) {

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

	logs, sub, err := _IncentivizedOutboundChannel.contract.FilterLogs(opts, "RoleGranted", roleRule, accountRule, senderRule)
	if err != nil {
		return nil, err
	}
	return &IncentivizedOutboundChannelRoleGrantedIterator{contract: _IncentivizedOutboundChannel.contract, event: "RoleGranted", logs: logs, sub: sub}, nil
}

// WatchRoleGranted is a free log subscription operation binding the contract event 0x2f8788117e7eff1d82e926ec794901d17c78024a50270940304540a733656f0d.
//
// Solidity: event RoleGranted(bytes32 indexed role, address indexed account, address indexed sender)
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelFilterer) WatchRoleGranted(opts *bind.WatchOpts, sink chan<- *IncentivizedOutboundChannelRoleGranted, role [][32]byte, account []common.Address, sender []common.Address) (event.Subscription, error) {

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

	logs, sub, err := _IncentivizedOutboundChannel.contract.WatchLogs(opts, "RoleGranted", roleRule, accountRule, senderRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(IncentivizedOutboundChannelRoleGranted)
				if err := _IncentivizedOutboundChannel.contract.UnpackLog(event, "RoleGranted", log); err != nil {
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
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelFilterer) ParseRoleGranted(log types.Log) (*IncentivizedOutboundChannelRoleGranted, error) {
	event := new(IncentivizedOutboundChannelRoleGranted)
	if err := _IncentivizedOutboundChannel.contract.UnpackLog(event, "RoleGranted", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// IncentivizedOutboundChannelRoleRevokedIterator is returned from FilterRoleRevoked and is used to iterate over the raw logs and unpacked data for RoleRevoked events raised by the IncentivizedOutboundChannel contract.
type IncentivizedOutboundChannelRoleRevokedIterator struct {
	Event *IncentivizedOutboundChannelRoleRevoked // Event containing the contract specifics and raw log

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
func (it *IncentivizedOutboundChannelRoleRevokedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(IncentivizedOutboundChannelRoleRevoked)
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
		it.Event = new(IncentivizedOutboundChannelRoleRevoked)
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
func (it *IncentivizedOutboundChannelRoleRevokedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *IncentivizedOutboundChannelRoleRevokedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// IncentivizedOutboundChannelRoleRevoked represents a RoleRevoked event raised by the IncentivizedOutboundChannel contract.
type IncentivizedOutboundChannelRoleRevoked struct {
	Role    [32]byte
	Account common.Address
	Sender  common.Address
	Raw     types.Log // Blockchain specific contextual infos
}

// FilterRoleRevoked is a free log retrieval operation binding the contract event 0xf6391f5c32d9c69d2a47ea670b442974b53935d1edc7fd64eb21e047a839171b.
//
// Solidity: event RoleRevoked(bytes32 indexed role, address indexed account, address indexed sender)
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelFilterer) FilterRoleRevoked(opts *bind.FilterOpts, role [][32]byte, account []common.Address, sender []common.Address) (*IncentivizedOutboundChannelRoleRevokedIterator, error) {

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

	logs, sub, err := _IncentivizedOutboundChannel.contract.FilterLogs(opts, "RoleRevoked", roleRule, accountRule, senderRule)
	if err != nil {
		return nil, err
	}
	return &IncentivizedOutboundChannelRoleRevokedIterator{contract: _IncentivizedOutboundChannel.contract, event: "RoleRevoked", logs: logs, sub: sub}, nil
}

// WatchRoleRevoked is a free log subscription operation binding the contract event 0xf6391f5c32d9c69d2a47ea670b442974b53935d1edc7fd64eb21e047a839171b.
//
// Solidity: event RoleRevoked(bytes32 indexed role, address indexed account, address indexed sender)
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelFilterer) WatchRoleRevoked(opts *bind.WatchOpts, sink chan<- *IncentivizedOutboundChannelRoleRevoked, role [][32]byte, account []common.Address, sender []common.Address) (event.Subscription, error) {

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

	logs, sub, err := _IncentivizedOutboundChannel.contract.WatchLogs(opts, "RoleRevoked", roleRule, accountRule, senderRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(IncentivizedOutboundChannelRoleRevoked)
				if err := _IncentivizedOutboundChannel.contract.UnpackLog(event, "RoleRevoked", log); err != nil {
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
func (_IncentivizedOutboundChannel *IncentivizedOutboundChannelFilterer) ParseRoleRevoked(log types.Log) (*IncentivizedOutboundChannelRoleRevoked, error) {
	event := new(IncentivizedOutboundChannelRoleRevoked)
	if err := _IncentivizedOutboundChannel.contract.UnpackLog(event, "RoleRevoked", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}
