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

// InboundChannelDispatchResult is an auto generated low-level Go binding around an user-defined struct.
type InboundChannelDispatchResult struct {
	Succeeded       bool
	ErrorReason     string
	ErrorPanicCode  *big.Int
	ErrorReturnData []byte
}

// InboundChannelHandler is an auto generated low-level Go binding around an user-defined struct.
type InboundChannelHandler struct {
	Recipient    common.Address
	GasToForward uint32
}

// InboundChannelMessage is an auto generated low-level Go binding around an user-defined struct.
type InboundChannelMessage struct {
	Origin  []byte
	Nonce   uint64
	Handler uint16
	Payload []byte
}

// InboundChannelMetaData contains all meta data concerning the InboundChannel contract.
var InboundChannelMetaData = &bind.MetaData{
	ABI: "[{\"inputs\":[{\"internalType\":\"contractIParachainClient\",\"name\":\"_parachainClient\",\"type\":\"address\"},{\"internalType\":\"contractIVault\",\"name\":\"_vault\",\"type\":\"address\"},{\"internalType\":\"uint256\",\"name\":\"_reward\",\"type\":\"uint256\"}],\"stateMutability\":\"nonpayable\",\"type\":\"constructor\"},{\"inputs\":[],\"name\":\"InvalidHandler\",\"type\":\"error\"},{\"inputs\":[],\"name\":\"InvalidNonce\",\"type\":\"error\"},{\"inputs\":[],\"name\":\"InvalidProof\",\"type\":\"error\"},{\"inputs\":[],\"name\":\"NotEnoughGas\",\"type\":\"error\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":false,\"internalType\":\"uint16\",\"name\":\"id\",\"type\":\"uint16\"},{\"components\":[{\"internalType\":\"address\",\"name\":\"recipient\",\"type\":\"address\"},{\"internalType\":\"uint32\",\"name\":\"gasToForward\",\"type\":\"uint32\"}],\"indexed\":false,\"internalType\":\"structInboundChannel.Handler\",\"name\":\"handler\",\"type\":\"tuple\"}],\"name\":\"HandlerUpdated\",\"type\":\"event\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":false,\"internalType\":\"bytes\",\"name\":\"origin\",\"type\":\"bytes\"},{\"indexed\":false,\"internalType\":\"uint64\",\"name\":\"nonce\",\"type\":\"uint64\"},{\"components\":[{\"internalType\":\"bool\",\"name\":\"succeeded\",\"type\":\"bool\"},{\"internalType\":\"string\",\"name\":\"errorReason\",\"type\":\"string\"},{\"internalType\":\"uint256\",\"name\":\"errorPanicCode\",\"type\":\"uint256\"},{\"internalType\":\"bytes\",\"name\":\"errorReturnData\",\"type\":\"bytes\"}],\"indexed\":false,\"internalType\":\"structInboundChannel.DispatchResult\",\"name\":\"result\",\"type\":\"tuple\"}],\"name\":\"MessageDispatched\",\"type\":\"event\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":false,\"internalType\":\"address\",\"name\":\"parachainClient\",\"type\":\"address\"}],\"name\":\"ParachainClientUpdated\",\"type\":\"event\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":false,\"internalType\":\"uint256\",\"name\":\"reward\",\"type\":\"uint256\"}],\"name\":\"RewardUpdated\",\"type\":\"event\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":true,\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\"},{\"indexed\":true,\"internalType\":\"bytes32\",\"name\":\"previousAdminRole\",\"type\":\"bytes32\"},{\"indexed\":true,\"internalType\":\"bytes32\",\"name\":\"newAdminRole\",\"type\":\"bytes32\"}],\"name\":\"RoleAdminChanged\",\"type\":\"event\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":true,\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\"},{\"indexed\":true,\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\"},{\"indexed\":true,\"internalType\":\"address\",\"name\":\"sender\",\"type\":\"address\"}],\"name\":\"RoleGranted\",\"type\":\"event\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":true,\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\"},{\"indexed\":true,\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\"},{\"indexed\":true,\"internalType\":\"address\",\"name\":\"sender\",\"type\":\"address\"}],\"name\":\"RoleRevoked\",\"type\":\"event\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":false,\"internalType\":\"address\",\"name\":\"vault\",\"type\":\"address\"}],\"name\":\"VaultUpdated\",\"type\":\"event\"},{\"inputs\":[],\"name\":\"ADMIN_ROLE\",\"outputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"DEFAULT_ADMIN_ROLE\",\"outputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"GAS_BUFFER\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\"}],\"name\":\"getRoleAdmin\",\"outputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\"},{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\"}],\"name\":\"grantRole\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\"},{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\"}],\"name\":\"hasRole\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"bytes\",\"name\":\"\",\"type\":\"bytes\"}],\"name\":\"nonce\",\"outputs\":[{\"internalType\":\"uint64\",\"name\":\"\",\"type\":\"uint64\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"parachainClient\",\"outputs\":[{\"internalType\":\"contractIParachainClient\",\"name\":\"\",\"type\":\"address\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\"},{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\"}],\"name\":\"renounceRole\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\"},{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\"}],\"name\":\"revokeRole\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"reward\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"components\":[{\"internalType\":\"bytes\",\"name\":\"origin\",\"type\":\"bytes\"},{\"internalType\":\"uint64\",\"name\":\"nonce\",\"type\":\"uint64\"},{\"internalType\":\"uint16\",\"name\":\"handler\",\"type\":\"uint16\"},{\"internalType\":\"bytes\",\"name\":\"payload\",\"type\":\"bytes\"}],\"internalType\":\"structInboundChannel.Message\",\"name\":\"message\",\"type\":\"tuple\"},{\"internalType\":\"bytes32[]\",\"name\":\"leafProof\",\"type\":\"bytes32[]\"},{\"internalType\":\"bytes\",\"name\":\"headerProof\",\"type\":\"bytes\"}],\"name\":\"submit\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"bytes4\",\"name\":\"interfaceId\",\"type\":\"bytes4\"}],\"name\":\"supportsInterface\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"uint16\",\"name\":\"id\",\"type\":\"uint16\"},{\"components\":[{\"internalType\":\"address\",\"name\":\"recipient\",\"type\":\"address\"},{\"internalType\":\"uint32\",\"name\":\"gasToForward\",\"type\":\"uint32\"}],\"internalType\":\"structInboundChannel.Handler\",\"name\":\"handler\",\"type\":\"tuple\"}],\"name\":\"updateHandler\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"contractIParachainClient\",\"name\":\"_parachainClient\",\"type\":\"address\"}],\"name\":\"updateParachainClient\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"_reward\",\"type\":\"uint256\"}],\"name\":\"updateReward\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"contractIVault\",\"name\":\"_vault\",\"type\":\"address\"}],\"name\":\"updateVault\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"vault\",\"outputs\":[{\"internalType\":\"contractIVault\",\"name\":\"\",\"type\":\"address\"}],\"stateMutability\":\"view\",\"type\":\"function\"}]",
}

// InboundChannelABI is the input ABI used to generate the binding from.
// Deprecated: Use InboundChannelMetaData.ABI instead.
var InboundChannelABI = InboundChannelMetaData.ABI

// InboundChannel is an auto generated Go binding around an Ethereum contract.
type InboundChannel struct {
	InboundChannelCaller     // Read-only binding to the contract
	InboundChannelTransactor // Write-only binding to the contract
	InboundChannelFilterer   // Log filterer for contract events
}

// InboundChannelCaller is an auto generated read-only Go binding around an Ethereum contract.
type InboundChannelCaller struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// InboundChannelTransactor is an auto generated write-only Go binding around an Ethereum contract.
type InboundChannelTransactor struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// InboundChannelFilterer is an auto generated log filtering Go binding around an Ethereum contract events.
type InboundChannelFilterer struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// InboundChannelSession is an auto generated Go binding around an Ethereum contract,
// with pre-set call and transact options.
type InboundChannelSession struct {
	Contract     *InboundChannel   // Generic contract binding to set the session for
	CallOpts     bind.CallOpts     // Call options to use throughout this session
	TransactOpts bind.TransactOpts // Transaction auth options to use throughout this session
}

// InboundChannelCallerSession is an auto generated read-only Go binding around an Ethereum contract,
// with pre-set call options.
type InboundChannelCallerSession struct {
	Contract *InboundChannelCaller // Generic contract caller binding to set the session for
	CallOpts bind.CallOpts         // Call options to use throughout this session
}

// InboundChannelTransactorSession is an auto generated write-only Go binding around an Ethereum contract,
// with pre-set transact options.
type InboundChannelTransactorSession struct {
	Contract     *InboundChannelTransactor // Generic contract transactor binding to set the session for
	TransactOpts bind.TransactOpts         // Transaction auth options to use throughout this session
}

// InboundChannelRaw is an auto generated low-level Go binding around an Ethereum contract.
type InboundChannelRaw struct {
	Contract *InboundChannel // Generic contract binding to access the raw methods on
}

// InboundChannelCallerRaw is an auto generated low-level read-only Go binding around an Ethereum contract.
type InboundChannelCallerRaw struct {
	Contract *InboundChannelCaller // Generic read-only contract binding to access the raw methods on
}

// InboundChannelTransactorRaw is an auto generated low-level write-only Go binding around an Ethereum contract.
type InboundChannelTransactorRaw struct {
	Contract *InboundChannelTransactor // Generic write-only contract binding to access the raw methods on
}

// NewInboundChannel creates a new instance of InboundChannel, bound to a specific deployed contract.
func NewInboundChannel(address common.Address, backend bind.ContractBackend) (*InboundChannel, error) {
	contract, err := bindInboundChannel(address, backend, backend, backend)
	if err != nil {
		return nil, err
	}
	return &InboundChannel{InboundChannelCaller: InboundChannelCaller{contract: contract}, InboundChannelTransactor: InboundChannelTransactor{contract: contract}, InboundChannelFilterer: InboundChannelFilterer{contract: contract}}, nil
}

// NewInboundChannelCaller creates a new read-only instance of InboundChannel, bound to a specific deployed contract.
func NewInboundChannelCaller(address common.Address, caller bind.ContractCaller) (*InboundChannelCaller, error) {
	contract, err := bindInboundChannel(address, caller, nil, nil)
	if err != nil {
		return nil, err
	}
	return &InboundChannelCaller{contract: contract}, nil
}

// NewInboundChannelTransactor creates a new write-only instance of InboundChannel, bound to a specific deployed contract.
func NewInboundChannelTransactor(address common.Address, transactor bind.ContractTransactor) (*InboundChannelTransactor, error) {
	contract, err := bindInboundChannel(address, nil, transactor, nil)
	if err != nil {
		return nil, err
	}
	return &InboundChannelTransactor{contract: contract}, nil
}

// NewInboundChannelFilterer creates a new log filterer instance of InboundChannel, bound to a specific deployed contract.
func NewInboundChannelFilterer(address common.Address, filterer bind.ContractFilterer) (*InboundChannelFilterer, error) {
	contract, err := bindInboundChannel(address, nil, nil, filterer)
	if err != nil {
		return nil, err
	}
	return &InboundChannelFilterer{contract: contract}, nil
}

// bindInboundChannel binds a generic wrapper to an already deployed contract.
func bindInboundChannel(address common.Address, caller bind.ContractCaller, transactor bind.ContractTransactor, filterer bind.ContractFilterer) (*bind.BoundContract, error) {
	parsed, err := abi.JSON(strings.NewReader(InboundChannelABI))
	if err != nil {
		return nil, err
	}
	return bind.NewBoundContract(address, parsed, caller, transactor, filterer), nil
}

// Call invokes the (constant) contract method with params as input values and
// sets the output to result. The result type might be a single field for simple
// returns, a slice of interfaces for anonymous returns and a struct for named
// returns.
func (_InboundChannel *InboundChannelRaw) Call(opts *bind.CallOpts, result *[]interface{}, method string, params ...interface{}) error {
	return _InboundChannel.Contract.InboundChannelCaller.contract.Call(opts, result, method, params...)
}

// Transfer initiates a plain transaction to move funds to the contract, calling
// its default method if one is available.
func (_InboundChannel *InboundChannelRaw) Transfer(opts *bind.TransactOpts) (*types.Transaction, error) {
	return _InboundChannel.Contract.InboundChannelTransactor.contract.Transfer(opts)
}

// Transact invokes the (paid) contract method with params as input values.
func (_InboundChannel *InboundChannelRaw) Transact(opts *bind.TransactOpts, method string, params ...interface{}) (*types.Transaction, error) {
	return _InboundChannel.Contract.InboundChannelTransactor.contract.Transact(opts, method, params...)
}

// Call invokes the (constant) contract method with params as input values and
// sets the output to result. The result type might be a single field for simple
// returns, a slice of interfaces for anonymous returns and a struct for named
// returns.
func (_InboundChannel *InboundChannelCallerRaw) Call(opts *bind.CallOpts, result *[]interface{}, method string, params ...interface{}) error {
	return _InboundChannel.Contract.contract.Call(opts, result, method, params...)
}

// Transfer initiates a plain transaction to move funds to the contract, calling
// its default method if one is available.
func (_InboundChannel *InboundChannelTransactorRaw) Transfer(opts *bind.TransactOpts) (*types.Transaction, error) {
	return _InboundChannel.Contract.contract.Transfer(opts)
}

// Transact invokes the (paid) contract method with params as input values.
func (_InboundChannel *InboundChannelTransactorRaw) Transact(opts *bind.TransactOpts, method string, params ...interface{}) (*types.Transaction, error) {
	return _InboundChannel.Contract.contract.Transact(opts, method, params...)
}

// ADMINROLE is a free data retrieval call binding the contract method 0x75b238fc.
//
// Solidity: function ADMIN_ROLE() view returns(bytes32)
func (_InboundChannel *InboundChannelCaller) ADMINROLE(opts *bind.CallOpts) ([32]byte, error) {
	var out []interface{}
	err := _InboundChannel.contract.Call(opts, &out, "ADMIN_ROLE")

	if err != nil {
		return *new([32]byte), err
	}

	out0 := *abi.ConvertType(out[0], new([32]byte)).(*[32]byte)

	return out0, err

}

// ADMINROLE is a free data retrieval call binding the contract method 0x75b238fc.
//
// Solidity: function ADMIN_ROLE() view returns(bytes32)
func (_InboundChannel *InboundChannelSession) ADMINROLE() ([32]byte, error) {
	return _InboundChannel.Contract.ADMINROLE(&_InboundChannel.CallOpts)
}

// ADMINROLE is a free data retrieval call binding the contract method 0x75b238fc.
//
// Solidity: function ADMIN_ROLE() view returns(bytes32)
func (_InboundChannel *InboundChannelCallerSession) ADMINROLE() ([32]byte, error) {
	return _InboundChannel.Contract.ADMINROLE(&_InboundChannel.CallOpts)
}

// DEFAULTADMINROLE is a free data retrieval call binding the contract method 0xa217fddf.
//
// Solidity: function DEFAULT_ADMIN_ROLE() view returns(bytes32)
func (_InboundChannel *InboundChannelCaller) DEFAULTADMINROLE(opts *bind.CallOpts) ([32]byte, error) {
	var out []interface{}
	err := _InboundChannel.contract.Call(opts, &out, "DEFAULT_ADMIN_ROLE")

	if err != nil {
		return *new([32]byte), err
	}

	out0 := *abi.ConvertType(out[0], new([32]byte)).(*[32]byte)

	return out0, err

}

// DEFAULTADMINROLE is a free data retrieval call binding the contract method 0xa217fddf.
//
// Solidity: function DEFAULT_ADMIN_ROLE() view returns(bytes32)
func (_InboundChannel *InboundChannelSession) DEFAULTADMINROLE() ([32]byte, error) {
	return _InboundChannel.Contract.DEFAULTADMINROLE(&_InboundChannel.CallOpts)
}

// DEFAULTADMINROLE is a free data retrieval call binding the contract method 0xa217fddf.
//
// Solidity: function DEFAULT_ADMIN_ROLE() view returns(bytes32)
func (_InboundChannel *InboundChannelCallerSession) DEFAULTADMINROLE() ([32]byte, error) {
	return _InboundChannel.Contract.DEFAULTADMINROLE(&_InboundChannel.CallOpts)
}

// GASBUFFER is a free data retrieval call binding the contract method 0xc7e67360.
//
// Solidity: function GAS_BUFFER() view returns(uint256)
func (_InboundChannel *InboundChannelCaller) GASBUFFER(opts *bind.CallOpts) (*big.Int, error) {
	var out []interface{}
	err := _InboundChannel.contract.Call(opts, &out, "GAS_BUFFER")

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// GASBUFFER is a free data retrieval call binding the contract method 0xc7e67360.
//
// Solidity: function GAS_BUFFER() view returns(uint256)
func (_InboundChannel *InboundChannelSession) GASBUFFER() (*big.Int, error) {
	return _InboundChannel.Contract.GASBUFFER(&_InboundChannel.CallOpts)
}

// GASBUFFER is a free data retrieval call binding the contract method 0xc7e67360.
//
// Solidity: function GAS_BUFFER() view returns(uint256)
func (_InboundChannel *InboundChannelCallerSession) GASBUFFER() (*big.Int, error) {
	return _InboundChannel.Contract.GASBUFFER(&_InboundChannel.CallOpts)
}

// GetRoleAdmin is a free data retrieval call binding the contract method 0x248a9ca3.
//
// Solidity: function getRoleAdmin(bytes32 role) view returns(bytes32)
func (_InboundChannel *InboundChannelCaller) GetRoleAdmin(opts *bind.CallOpts, role [32]byte) ([32]byte, error) {
	var out []interface{}
	err := _InboundChannel.contract.Call(opts, &out, "getRoleAdmin", role)

	if err != nil {
		return *new([32]byte), err
	}

	out0 := *abi.ConvertType(out[0], new([32]byte)).(*[32]byte)

	return out0, err

}

// GetRoleAdmin is a free data retrieval call binding the contract method 0x248a9ca3.
//
// Solidity: function getRoleAdmin(bytes32 role) view returns(bytes32)
func (_InboundChannel *InboundChannelSession) GetRoleAdmin(role [32]byte) ([32]byte, error) {
	return _InboundChannel.Contract.GetRoleAdmin(&_InboundChannel.CallOpts, role)
}

// GetRoleAdmin is a free data retrieval call binding the contract method 0x248a9ca3.
//
// Solidity: function getRoleAdmin(bytes32 role) view returns(bytes32)
func (_InboundChannel *InboundChannelCallerSession) GetRoleAdmin(role [32]byte) ([32]byte, error) {
	return _InboundChannel.Contract.GetRoleAdmin(&_InboundChannel.CallOpts, role)
}

// HasRole is a free data retrieval call binding the contract method 0x91d14854.
//
// Solidity: function hasRole(bytes32 role, address account) view returns(bool)
func (_InboundChannel *InboundChannelCaller) HasRole(opts *bind.CallOpts, role [32]byte, account common.Address) (bool, error) {
	var out []interface{}
	err := _InboundChannel.contract.Call(opts, &out, "hasRole", role, account)

	if err != nil {
		return *new(bool), err
	}

	out0 := *abi.ConvertType(out[0], new(bool)).(*bool)

	return out0, err

}

// HasRole is a free data retrieval call binding the contract method 0x91d14854.
//
// Solidity: function hasRole(bytes32 role, address account) view returns(bool)
func (_InboundChannel *InboundChannelSession) HasRole(role [32]byte, account common.Address) (bool, error) {
	return _InboundChannel.Contract.HasRole(&_InboundChannel.CallOpts, role, account)
}

// HasRole is a free data retrieval call binding the contract method 0x91d14854.
//
// Solidity: function hasRole(bytes32 role, address account) view returns(bool)
func (_InboundChannel *InboundChannelCallerSession) HasRole(role [32]byte, account common.Address) (bool, error) {
	return _InboundChannel.Contract.HasRole(&_InboundChannel.CallOpts, role, account)
}

// Nonce is a free data retrieval call binding the contract method 0x4e765004.
//
// Solidity: function nonce(bytes ) view returns(uint64)
func (_InboundChannel *InboundChannelCaller) Nonce(opts *bind.CallOpts, arg0 []byte) (uint64, error) {
	var out []interface{}
	err := _InboundChannel.contract.Call(opts, &out, "nonce", arg0)

	if err != nil {
		return *new(uint64), err
	}

	out0 := *abi.ConvertType(out[0], new(uint64)).(*uint64)

	return out0, err

}

// Nonce is a free data retrieval call binding the contract method 0x4e765004.
//
// Solidity: function nonce(bytes ) view returns(uint64)
func (_InboundChannel *InboundChannelSession) Nonce(arg0 []byte) (uint64, error) {
	return _InboundChannel.Contract.Nonce(&_InboundChannel.CallOpts, arg0)
}

// Nonce is a free data retrieval call binding the contract method 0x4e765004.
//
// Solidity: function nonce(bytes ) view returns(uint64)
func (_InboundChannel *InboundChannelCallerSession) Nonce(arg0 []byte) (uint64, error) {
	return _InboundChannel.Contract.Nonce(&_InboundChannel.CallOpts, arg0)
}

// ParachainClient is a free data retrieval call binding the contract method 0x1674f9b7.
//
// Solidity: function parachainClient() view returns(address)
func (_InboundChannel *InboundChannelCaller) ParachainClient(opts *bind.CallOpts) (common.Address, error) {
	var out []interface{}
	err := _InboundChannel.contract.Call(opts, &out, "parachainClient")

	if err != nil {
		return *new(common.Address), err
	}

	out0 := *abi.ConvertType(out[0], new(common.Address)).(*common.Address)

	return out0, err

}

// ParachainClient is a free data retrieval call binding the contract method 0x1674f9b7.
//
// Solidity: function parachainClient() view returns(address)
func (_InboundChannel *InboundChannelSession) ParachainClient() (common.Address, error) {
	return _InboundChannel.Contract.ParachainClient(&_InboundChannel.CallOpts)
}

// ParachainClient is a free data retrieval call binding the contract method 0x1674f9b7.
//
// Solidity: function parachainClient() view returns(address)
func (_InboundChannel *InboundChannelCallerSession) ParachainClient() (common.Address, error) {
	return _InboundChannel.Contract.ParachainClient(&_InboundChannel.CallOpts)
}

// Reward is a free data retrieval call binding the contract method 0x228cb733.
//
// Solidity: function reward() view returns(uint256)
func (_InboundChannel *InboundChannelCaller) Reward(opts *bind.CallOpts) (*big.Int, error) {
	var out []interface{}
	err := _InboundChannel.contract.Call(opts, &out, "reward")

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// Reward is a free data retrieval call binding the contract method 0x228cb733.
//
// Solidity: function reward() view returns(uint256)
func (_InboundChannel *InboundChannelSession) Reward() (*big.Int, error) {
	return _InboundChannel.Contract.Reward(&_InboundChannel.CallOpts)
}

// Reward is a free data retrieval call binding the contract method 0x228cb733.
//
// Solidity: function reward() view returns(uint256)
func (_InboundChannel *InboundChannelCallerSession) Reward() (*big.Int, error) {
	return _InboundChannel.Contract.Reward(&_InboundChannel.CallOpts)
}

// SupportsInterface is a free data retrieval call binding the contract method 0x01ffc9a7.
//
// Solidity: function supportsInterface(bytes4 interfaceId) view returns(bool)
func (_InboundChannel *InboundChannelCaller) SupportsInterface(opts *bind.CallOpts, interfaceId [4]byte) (bool, error) {
	var out []interface{}
	err := _InboundChannel.contract.Call(opts, &out, "supportsInterface", interfaceId)

	if err != nil {
		return *new(bool), err
	}

	out0 := *abi.ConvertType(out[0], new(bool)).(*bool)

	return out0, err

}

// SupportsInterface is a free data retrieval call binding the contract method 0x01ffc9a7.
//
// Solidity: function supportsInterface(bytes4 interfaceId) view returns(bool)
func (_InboundChannel *InboundChannelSession) SupportsInterface(interfaceId [4]byte) (bool, error) {
	return _InboundChannel.Contract.SupportsInterface(&_InboundChannel.CallOpts, interfaceId)
}

// SupportsInterface is a free data retrieval call binding the contract method 0x01ffc9a7.
//
// Solidity: function supportsInterface(bytes4 interfaceId) view returns(bool)
func (_InboundChannel *InboundChannelCallerSession) SupportsInterface(interfaceId [4]byte) (bool, error) {
	return _InboundChannel.Contract.SupportsInterface(&_InboundChannel.CallOpts, interfaceId)
}

// Vault is a free data retrieval call binding the contract method 0xfbfa77cf.
//
// Solidity: function vault() view returns(address)
func (_InboundChannel *InboundChannelCaller) Vault(opts *bind.CallOpts) (common.Address, error) {
	var out []interface{}
	err := _InboundChannel.contract.Call(opts, &out, "vault")

	if err != nil {
		return *new(common.Address), err
	}

	out0 := *abi.ConvertType(out[0], new(common.Address)).(*common.Address)

	return out0, err

}

// Vault is a free data retrieval call binding the contract method 0xfbfa77cf.
//
// Solidity: function vault() view returns(address)
func (_InboundChannel *InboundChannelSession) Vault() (common.Address, error) {
	return _InboundChannel.Contract.Vault(&_InboundChannel.CallOpts)
}

// Vault is a free data retrieval call binding the contract method 0xfbfa77cf.
//
// Solidity: function vault() view returns(address)
func (_InboundChannel *InboundChannelCallerSession) Vault() (common.Address, error) {
	return _InboundChannel.Contract.Vault(&_InboundChannel.CallOpts)
}

// GrantRole is a paid mutator transaction binding the contract method 0x2f2ff15d.
//
// Solidity: function grantRole(bytes32 role, address account) returns()
func (_InboundChannel *InboundChannelTransactor) GrantRole(opts *bind.TransactOpts, role [32]byte, account common.Address) (*types.Transaction, error) {
	return _InboundChannel.contract.Transact(opts, "grantRole", role, account)
}

// GrantRole is a paid mutator transaction binding the contract method 0x2f2ff15d.
//
// Solidity: function grantRole(bytes32 role, address account) returns()
func (_InboundChannel *InboundChannelSession) GrantRole(role [32]byte, account common.Address) (*types.Transaction, error) {
	return _InboundChannel.Contract.GrantRole(&_InboundChannel.TransactOpts, role, account)
}

// GrantRole is a paid mutator transaction binding the contract method 0x2f2ff15d.
//
// Solidity: function grantRole(bytes32 role, address account) returns()
func (_InboundChannel *InboundChannelTransactorSession) GrantRole(role [32]byte, account common.Address) (*types.Transaction, error) {
	return _InboundChannel.Contract.GrantRole(&_InboundChannel.TransactOpts, role, account)
}

// RenounceRole is a paid mutator transaction binding the contract method 0x36568abe.
//
// Solidity: function renounceRole(bytes32 role, address account) returns()
func (_InboundChannel *InboundChannelTransactor) RenounceRole(opts *bind.TransactOpts, role [32]byte, account common.Address) (*types.Transaction, error) {
	return _InboundChannel.contract.Transact(opts, "renounceRole", role, account)
}

// RenounceRole is a paid mutator transaction binding the contract method 0x36568abe.
//
// Solidity: function renounceRole(bytes32 role, address account) returns()
func (_InboundChannel *InboundChannelSession) RenounceRole(role [32]byte, account common.Address) (*types.Transaction, error) {
	return _InboundChannel.Contract.RenounceRole(&_InboundChannel.TransactOpts, role, account)
}

// RenounceRole is a paid mutator transaction binding the contract method 0x36568abe.
//
// Solidity: function renounceRole(bytes32 role, address account) returns()
func (_InboundChannel *InboundChannelTransactorSession) RenounceRole(role [32]byte, account common.Address) (*types.Transaction, error) {
	return _InboundChannel.Contract.RenounceRole(&_InboundChannel.TransactOpts, role, account)
}

// RevokeRole is a paid mutator transaction binding the contract method 0xd547741f.
//
// Solidity: function revokeRole(bytes32 role, address account) returns()
func (_InboundChannel *InboundChannelTransactor) RevokeRole(opts *bind.TransactOpts, role [32]byte, account common.Address) (*types.Transaction, error) {
	return _InboundChannel.contract.Transact(opts, "revokeRole", role, account)
}

// RevokeRole is a paid mutator transaction binding the contract method 0xd547741f.
//
// Solidity: function revokeRole(bytes32 role, address account) returns()
func (_InboundChannel *InboundChannelSession) RevokeRole(role [32]byte, account common.Address) (*types.Transaction, error) {
	return _InboundChannel.Contract.RevokeRole(&_InboundChannel.TransactOpts, role, account)
}

// RevokeRole is a paid mutator transaction binding the contract method 0xd547741f.
//
// Solidity: function revokeRole(bytes32 role, address account) returns()
func (_InboundChannel *InboundChannelTransactorSession) RevokeRole(role [32]byte, account common.Address) (*types.Transaction, error) {
	return _InboundChannel.Contract.RevokeRole(&_InboundChannel.TransactOpts, role, account)
}

// Submit is a paid mutator transaction binding the contract method 0x824960a2.
//
// Solidity: function submit((bytes,uint64,uint16,bytes) message, bytes32[] leafProof, bytes headerProof) returns()
func (_InboundChannel *InboundChannelTransactor) Submit(opts *bind.TransactOpts, message InboundChannelMessage, leafProof [][32]byte, headerProof []byte) (*types.Transaction, error) {
	return _InboundChannel.contract.Transact(opts, "submit", message, leafProof, headerProof)
}

// Submit is a paid mutator transaction binding the contract method 0x824960a2.
//
// Solidity: function submit((bytes,uint64,uint16,bytes) message, bytes32[] leafProof, bytes headerProof) returns()
func (_InboundChannel *InboundChannelSession) Submit(message InboundChannelMessage, leafProof [][32]byte, headerProof []byte) (*types.Transaction, error) {
	return _InboundChannel.Contract.Submit(&_InboundChannel.TransactOpts, message, leafProof, headerProof)
}

// Submit is a paid mutator transaction binding the contract method 0x824960a2.
//
// Solidity: function submit((bytes,uint64,uint16,bytes) message, bytes32[] leafProof, bytes headerProof) returns()
func (_InboundChannel *InboundChannelTransactorSession) Submit(message InboundChannelMessage, leafProof [][32]byte, headerProof []byte) (*types.Transaction, error) {
	return _InboundChannel.Contract.Submit(&_InboundChannel.TransactOpts, message, leafProof, headerProof)
}

// UpdateHandler is a paid mutator transaction binding the contract method 0x094440f3.
//
// Solidity: function updateHandler(uint16 id, (address,uint32) handler) returns()
func (_InboundChannel *InboundChannelTransactor) UpdateHandler(opts *bind.TransactOpts, id uint16, handler InboundChannelHandler) (*types.Transaction, error) {
	return _InboundChannel.contract.Transact(opts, "updateHandler", id, handler)
}

// UpdateHandler is a paid mutator transaction binding the contract method 0x094440f3.
//
// Solidity: function updateHandler(uint16 id, (address,uint32) handler) returns()
func (_InboundChannel *InboundChannelSession) UpdateHandler(id uint16, handler InboundChannelHandler) (*types.Transaction, error) {
	return _InboundChannel.Contract.UpdateHandler(&_InboundChannel.TransactOpts, id, handler)
}

// UpdateHandler is a paid mutator transaction binding the contract method 0x094440f3.
//
// Solidity: function updateHandler(uint16 id, (address,uint32) handler) returns()
func (_InboundChannel *InboundChannelTransactorSession) UpdateHandler(id uint16, handler InboundChannelHandler) (*types.Transaction, error) {
	return _InboundChannel.Contract.UpdateHandler(&_InboundChannel.TransactOpts, id, handler)
}

// UpdateParachainClient is a paid mutator transaction binding the contract method 0x33b9d6ee.
//
// Solidity: function updateParachainClient(address _parachainClient) returns()
func (_InboundChannel *InboundChannelTransactor) UpdateParachainClient(opts *bind.TransactOpts, _parachainClient common.Address) (*types.Transaction, error) {
	return _InboundChannel.contract.Transact(opts, "updateParachainClient", _parachainClient)
}

// UpdateParachainClient is a paid mutator transaction binding the contract method 0x33b9d6ee.
//
// Solidity: function updateParachainClient(address _parachainClient) returns()
func (_InboundChannel *InboundChannelSession) UpdateParachainClient(_parachainClient common.Address) (*types.Transaction, error) {
	return _InboundChannel.Contract.UpdateParachainClient(&_InboundChannel.TransactOpts, _parachainClient)
}

// UpdateParachainClient is a paid mutator transaction binding the contract method 0x33b9d6ee.
//
// Solidity: function updateParachainClient(address _parachainClient) returns()
func (_InboundChannel *InboundChannelTransactorSession) UpdateParachainClient(_parachainClient common.Address) (*types.Transaction, error) {
	return _InboundChannel.Contract.UpdateParachainClient(&_InboundChannel.TransactOpts, _parachainClient)
}

// UpdateReward is a paid mutator transaction binding the contract method 0x425c8abd.
//
// Solidity: function updateReward(uint256 _reward) returns()
func (_InboundChannel *InboundChannelTransactor) UpdateReward(opts *bind.TransactOpts, _reward *big.Int) (*types.Transaction, error) {
	return _InboundChannel.contract.Transact(opts, "updateReward", _reward)
}

// UpdateReward is a paid mutator transaction binding the contract method 0x425c8abd.
//
// Solidity: function updateReward(uint256 _reward) returns()
func (_InboundChannel *InboundChannelSession) UpdateReward(_reward *big.Int) (*types.Transaction, error) {
	return _InboundChannel.Contract.UpdateReward(&_InboundChannel.TransactOpts, _reward)
}

// UpdateReward is a paid mutator transaction binding the contract method 0x425c8abd.
//
// Solidity: function updateReward(uint256 _reward) returns()
func (_InboundChannel *InboundChannelTransactorSession) UpdateReward(_reward *big.Int) (*types.Transaction, error) {
	return _InboundChannel.Contract.UpdateReward(&_InboundChannel.TransactOpts, _reward)
}

// UpdateVault is a paid mutator transaction binding the contract method 0xe7563f3f.
//
// Solidity: function updateVault(address _vault) returns()
func (_InboundChannel *InboundChannelTransactor) UpdateVault(opts *bind.TransactOpts, _vault common.Address) (*types.Transaction, error) {
	return _InboundChannel.contract.Transact(opts, "updateVault", _vault)
}

// UpdateVault is a paid mutator transaction binding the contract method 0xe7563f3f.
//
// Solidity: function updateVault(address _vault) returns()
func (_InboundChannel *InboundChannelSession) UpdateVault(_vault common.Address) (*types.Transaction, error) {
	return _InboundChannel.Contract.UpdateVault(&_InboundChannel.TransactOpts, _vault)
}

// UpdateVault is a paid mutator transaction binding the contract method 0xe7563f3f.
//
// Solidity: function updateVault(address _vault) returns()
func (_InboundChannel *InboundChannelTransactorSession) UpdateVault(_vault common.Address) (*types.Transaction, error) {
	return _InboundChannel.Contract.UpdateVault(&_InboundChannel.TransactOpts, _vault)
}

// InboundChannelHandlerUpdatedIterator is returned from FilterHandlerUpdated and is used to iterate over the raw logs and unpacked data for HandlerUpdated events raised by the InboundChannel contract.
type InboundChannelHandlerUpdatedIterator struct {
	Event *InboundChannelHandlerUpdated // Event containing the contract specifics and raw log

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
func (it *InboundChannelHandlerUpdatedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(InboundChannelHandlerUpdated)
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
		it.Event = new(InboundChannelHandlerUpdated)
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
func (it *InboundChannelHandlerUpdatedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *InboundChannelHandlerUpdatedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// InboundChannelHandlerUpdated represents a HandlerUpdated event raised by the InboundChannel contract.
type InboundChannelHandlerUpdated struct {
	Id      uint16
	Handler InboundChannelHandler
	Raw     types.Log // Blockchain specific contextual infos
}

// FilterHandlerUpdated is a free log retrieval operation binding the contract event 0x6c2fc0cb577f29b9193278b7dacc4fb7e2e96a881a16c58e9fb8b4a60ad44c02.
//
// Solidity: event HandlerUpdated(uint16 id, (address,uint32) handler)
func (_InboundChannel *InboundChannelFilterer) FilterHandlerUpdated(opts *bind.FilterOpts) (*InboundChannelHandlerUpdatedIterator, error) {

	logs, sub, err := _InboundChannel.contract.FilterLogs(opts, "HandlerUpdated")
	if err != nil {
		return nil, err
	}
	return &InboundChannelHandlerUpdatedIterator{contract: _InboundChannel.contract, event: "HandlerUpdated", logs: logs, sub: sub}, nil
}

// WatchHandlerUpdated is a free log subscription operation binding the contract event 0x6c2fc0cb577f29b9193278b7dacc4fb7e2e96a881a16c58e9fb8b4a60ad44c02.
//
// Solidity: event HandlerUpdated(uint16 id, (address,uint32) handler)
func (_InboundChannel *InboundChannelFilterer) WatchHandlerUpdated(opts *bind.WatchOpts, sink chan<- *InboundChannelHandlerUpdated) (event.Subscription, error) {

	logs, sub, err := _InboundChannel.contract.WatchLogs(opts, "HandlerUpdated")
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(InboundChannelHandlerUpdated)
				if err := _InboundChannel.contract.UnpackLog(event, "HandlerUpdated", log); err != nil {
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

// ParseHandlerUpdated is a log parse operation binding the contract event 0x6c2fc0cb577f29b9193278b7dacc4fb7e2e96a881a16c58e9fb8b4a60ad44c02.
//
// Solidity: event HandlerUpdated(uint16 id, (address,uint32) handler)
func (_InboundChannel *InboundChannelFilterer) ParseHandlerUpdated(log types.Log) (*InboundChannelHandlerUpdated, error) {
	event := new(InboundChannelHandlerUpdated)
	if err := _InboundChannel.contract.UnpackLog(event, "HandlerUpdated", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// InboundChannelMessageDispatchedIterator is returned from FilterMessageDispatched and is used to iterate over the raw logs and unpacked data for MessageDispatched events raised by the InboundChannel contract.
type InboundChannelMessageDispatchedIterator struct {
	Event *InboundChannelMessageDispatched // Event containing the contract specifics and raw log

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
func (it *InboundChannelMessageDispatchedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(InboundChannelMessageDispatched)
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
		it.Event = new(InboundChannelMessageDispatched)
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
func (it *InboundChannelMessageDispatchedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *InboundChannelMessageDispatchedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// InboundChannelMessageDispatched represents a MessageDispatched event raised by the InboundChannel contract.
type InboundChannelMessageDispatched struct {
	Origin []byte
	Nonce  uint64
	Result InboundChannelDispatchResult
	Raw    types.Log // Blockchain specific contextual infos
}

// FilterMessageDispatched is a free log retrieval operation binding the contract event 0x3aaa5c2dcc1be357b3aac5b7a4dbb2ecdae5b819f7fc5a48fea27b42aa705b79.
//
// Solidity: event MessageDispatched(bytes origin, uint64 nonce, (bool,string,uint256,bytes) result)
func (_InboundChannel *InboundChannelFilterer) FilterMessageDispatched(opts *bind.FilterOpts) (*InboundChannelMessageDispatchedIterator, error) {

	logs, sub, err := _InboundChannel.contract.FilterLogs(opts, "MessageDispatched")
	if err != nil {
		return nil, err
	}
	return &InboundChannelMessageDispatchedIterator{contract: _InboundChannel.contract, event: "MessageDispatched", logs: logs, sub: sub}, nil
}

// WatchMessageDispatched is a free log subscription operation binding the contract event 0x3aaa5c2dcc1be357b3aac5b7a4dbb2ecdae5b819f7fc5a48fea27b42aa705b79.
//
// Solidity: event MessageDispatched(bytes origin, uint64 nonce, (bool,string,uint256,bytes) result)
func (_InboundChannel *InboundChannelFilterer) WatchMessageDispatched(opts *bind.WatchOpts, sink chan<- *InboundChannelMessageDispatched) (event.Subscription, error) {

	logs, sub, err := _InboundChannel.contract.WatchLogs(opts, "MessageDispatched")
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(InboundChannelMessageDispatched)
				if err := _InboundChannel.contract.UnpackLog(event, "MessageDispatched", log); err != nil {
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

// ParseMessageDispatched is a log parse operation binding the contract event 0x3aaa5c2dcc1be357b3aac5b7a4dbb2ecdae5b819f7fc5a48fea27b42aa705b79.
//
// Solidity: event MessageDispatched(bytes origin, uint64 nonce, (bool,string,uint256,bytes) result)
func (_InboundChannel *InboundChannelFilterer) ParseMessageDispatched(log types.Log) (*InboundChannelMessageDispatched, error) {
	event := new(InboundChannelMessageDispatched)
	if err := _InboundChannel.contract.UnpackLog(event, "MessageDispatched", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// InboundChannelParachainClientUpdatedIterator is returned from FilterParachainClientUpdated and is used to iterate over the raw logs and unpacked data for ParachainClientUpdated events raised by the InboundChannel contract.
type InboundChannelParachainClientUpdatedIterator struct {
	Event *InboundChannelParachainClientUpdated // Event containing the contract specifics and raw log

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
func (it *InboundChannelParachainClientUpdatedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(InboundChannelParachainClientUpdated)
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
		it.Event = new(InboundChannelParachainClientUpdated)
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
func (it *InboundChannelParachainClientUpdatedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *InboundChannelParachainClientUpdatedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// InboundChannelParachainClientUpdated represents a ParachainClientUpdated event raised by the InboundChannel contract.
type InboundChannelParachainClientUpdated struct {
	ParachainClient common.Address
	Raw             types.Log // Blockchain specific contextual infos
}

// FilterParachainClientUpdated is a free log retrieval operation binding the contract event 0x2eb3efb9388b586c251856bc1b31af67a4015796cf665e600df15d2a42e2ba41.
//
// Solidity: event ParachainClientUpdated(address parachainClient)
func (_InboundChannel *InboundChannelFilterer) FilterParachainClientUpdated(opts *bind.FilterOpts) (*InboundChannelParachainClientUpdatedIterator, error) {

	logs, sub, err := _InboundChannel.contract.FilterLogs(opts, "ParachainClientUpdated")
	if err != nil {
		return nil, err
	}
	return &InboundChannelParachainClientUpdatedIterator{contract: _InboundChannel.contract, event: "ParachainClientUpdated", logs: logs, sub: sub}, nil
}

// WatchParachainClientUpdated is a free log subscription operation binding the contract event 0x2eb3efb9388b586c251856bc1b31af67a4015796cf665e600df15d2a42e2ba41.
//
// Solidity: event ParachainClientUpdated(address parachainClient)
func (_InboundChannel *InboundChannelFilterer) WatchParachainClientUpdated(opts *bind.WatchOpts, sink chan<- *InboundChannelParachainClientUpdated) (event.Subscription, error) {

	logs, sub, err := _InboundChannel.contract.WatchLogs(opts, "ParachainClientUpdated")
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(InboundChannelParachainClientUpdated)
				if err := _InboundChannel.contract.UnpackLog(event, "ParachainClientUpdated", log); err != nil {
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

// ParseParachainClientUpdated is a log parse operation binding the contract event 0x2eb3efb9388b586c251856bc1b31af67a4015796cf665e600df15d2a42e2ba41.
//
// Solidity: event ParachainClientUpdated(address parachainClient)
func (_InboundChannel *InboundChannelFilterer) ParseParachainClientUpdated(log types.Log) (*InboundChannelParachainClientUpdated, error) {
	event := new(InboundChannelParachainClientUpdated)
	if err := _InboundChannel.contract.UnpackLog(event, "ParachainClientUpdated", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// InboundChannelRewardUpdatedIterator is returned from FilterRewardUpdated and is used to iterate over the raw logs and unpacked data for RewardUpdated events raised by the InboundChannel contract.
type InboundChannelRewardUpdatedIterator struct {
	Event *InboundChannelRewardUpdated // Event containing the contract specifics and raw log

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
func (it *InboundChannelRewardUpdatedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(InboundChannelRewardUpdated)
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
		it.Event = new(InboundChannelRewardUpdated)
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
func (it *InboundChannelRewardUpdatedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *InboundChannelRewardUpdatedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// InboundChannelRewardUpdated represents a RewardUpdated event raised by the InboundChannel contract.
type InboundChannelRewardUpdated struct {
	Reward *big.Int
	Raw    types.Log // Blockchain specific contextual infos
}

// FilterRewardUpdated is a free log retrieval operation binding the contract event 0xcb94909754d27c309adf4167150f1f7aa04de40b6a0e6bb98b2ae80a2bf438f6.
//
// Solidity: event RewardUpdated(uint256 reward)
func (_InboundChannel *InboundChannelFilterer) FilterRewardUpdated(opts *bind.FilterOpts) (*InboundChannelRewardUpdatedIterator, error) {

	logs, sub, err := _InboundChannel.contract.FilterLogs(opts, "RewardUpdated")
	if err != nil {
		return nil, err
	}
	return &InboundChannelRewardUpdatedIterator{contract: _InboundChannel.contract, event: "RewardUpdated", logs: logs, sub: sub}, nil
}

// WatchRewardUpdated is a free log subscription operation binding the contract event 0xcb94909754d27c309adf4167150f1f7aa04de40b6a0e6bb98b2ae80a2bf438f6.
//
// Solidity: event RewardUpdated(uint256 reward)
func (_InboundChannel *InboundChannelFilterer) WatchRewardUpdated(opts *bind.WatchOpts, sink chan<- *InboundChannelRewardUpdated) (event.Subscription, error) {

	logs, sub, err := _InboundChannel.contract.WatchLogs(opts, "RewardUpdated")
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(InboundChannelRewardUpdated)
				if err := _InboundChannel.contract.UnpackLog(event, "RewardUpdated", log); err != nil {
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

// ParseRewardUpdated is a log parse operation binding the contract event 0xcb94909754d27c309adf4167150f1f7aa04de40b6a0e6bb98b2ae80a2bf438f6.
//
// Solidity: event RewardUpdated(uint256 reward)
func (_InboundChannel *InboundChannelFilterer) ParseRewardUpdated(log types.Log) (*InboundChannelRewardUpdated, error) {
	event := new(InboundChannelRewardUpdated)
	if err := _InboundChannel.contract.UnpackLog(event, "RewardUpdated", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// InboundChannelRoleAdminChangedIterator is returned from FilterRoleAdminChanged and is used to iterate over the raw logs and unpacked data for RoleAdminChanged events raised by the InboundChannel contract.
type InboundChannelRoleAdminChangedIterator struct {
	Event *InboundChannelRoleAdminChanged // Event containing the contract specifics and raw log

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
func (it *InboundChannelRoleAdminChangedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(InboundChannelRoleAdminChanged)
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
		it.Event = new(InboundChannelRoleAdminChanged)
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
func (it *InboundChannelRoleAdminChangedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *InboundChannelRoleAdminChangedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// InboundChannelRoleAdminChanged represents a RoleAdminChanged event raised by the InboundChannel contract.
type InboundChannelRoleAdminChanged struct {
	Role              [32]byte
	PreviousAdminRole [32]byte
	NewAdminRole      [32]byte
	Raw               types.Log // Blockchain specific contextual infos
}

// FilterRoleAdminChanged is a free log retrieval operation binding the contract event 0xbd79b86ffe0ab8e8776151514217cd7cacd52c909f66475c3af44e129f0b00ff.
//
// Solidity: event RoleAdminChanged(bytes32 indexed role, bytes32 indexed previousAdminRole, bytes32 indexed newAdminRole)
func (_InboundChannel *InboundChannelFilterer) FilterRoleAdminChanged(opts *bind.FilterOpts, role [][32]byte, previousAdminRole [][32]byte, newAdminRole [][32]byte) (*InboundChannelRoleAdminChangedIterator, error) {

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

	logs, sub, err := _InboundChannel.contract.FilterLogs(opts, "RoleAdminChanged", roleRule, previousAdminRoleRule, newAdminRoleRule)
	if err != nil {
		return nil, err
	}
	return &InboundChannelRoleAdminChangedIterator{contract: _InboundChannel.contract, event: "RoleAdminChanged", logs: logs, sub: sub}, nil
}

// WatchRoleAdminChanged is a free log subscription operation binding the contract event 0xbd79b86ffe0ab8e8776151514217cd7cacd52c909f66475c3af44e129f0b00ff.
//
// Solidity: event RoleAdminChanged(bytes32 indexed role, bytes32 indexed previousAdminRole, bytes32 indexed newAdminRole)
func (_InboundChannel *InboundChannelFilterer) WatchRoleAdminChanged(opts *bind.WatchOpts, sink chan<- *InboundChannelRoleAdminChanged, role [][32]byte, previousAdminRole [][32]byte, newAdminRole [][32]byte) (event.Subscription, error) {

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

	logs, sub, err := _InboundChannel.contract.WatchLogs(opts, "RoleAdminChanged", roleRule, previousAdminRoleRule, newAdminRoleRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(InboundChannelRoleAdminChanged)
				if err := _InboundChannel.contract.UnpackLog(event, "RoleAdminChanged", log); err != nil {
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
func (_InboundChannel *InboundChannelFilterer) ParseRoleAdminChanged(log types.Log) (*InboundChannelRoleAdminChanged, error) {
	event := new(InboundChannelRoleAdminChanged)
	if err := _InboundChannel.contract.UnpackLog(event, "RoleAdminChanged", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// InboundChannelRoleGrantedIterator is returned from FilterRoleGranted and is used to iterate over the raw logs and unpacked data for RoleGranted events raised by the InboundChannel contract.
type InboundChannelRoleGrantedIterator struct {
	Event *InboundChannelRoleGranted // Event containing the contract specifics and raw log

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
func (it *InboundChannelRoleGrantedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(InboundChannelRoleGranted)
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
		it.Event = new(InboundChannelRoleGranted)
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
func (it *InboundChannelRoleGrantedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *InboundChannelRoleGrantedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// InboundChannelRoleGranted represents a RoleGranted event raised by the InboundChannel contract.
type InboundChannelRoleGranted struct {
	Role    [32]byte
	Account common.Address
	Sender  common.Address
	Raw     types.Log // Blockchain specific contextual infos
}

// FilterRoleGranted is a free log retrieval operation binding the contract event 0x2f8788117e7eff1d82e926ec794901d17c78024a50270940304540a733656f0d.
//
// Solidity: event RoleGranted(bytes32 indexed role, address indexed account, address indexed sender)
func (_InboundChannel *InboundChannelFilterer) FilterRoleGranted(opts *bind.FilterOpts, role [][32]byte, account []common.Address, sender []common.Address) (*InboundChannelRoleGrantedIterator, error) {

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

	logs, sub, err := _InboundChannel.contract.FilterLogs(opts, "RoleGranted", roleRule, accountRule, senderRule)
	if err != nil {
		return nil, err
	}
	return &InboundChannelRoleGrantedIterator{contract: _InboundChannel.contract, event: "RoleGranted", logs: logs, sub: sub}, nil
}

// WatchRoleGranted is a free log subscription operation binding the contract event 0x2f8788117e7eff1d82e926ec794901d17c78024a50270940304540a733656f0d.
//
// Solidity: event RoleGranted(bytes32 indexed role, address indexed account, address indexed sender)
func (_InboundChannel *InboundChannelFilterer) WatchRoleGranted(opts *bind.WatchOpts, sink chan<- *InboundChannelRoleGranted, role [][32]byte, account []common.Address, sender []common.Address) (event.Subscription, error) {

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

	logs, sub, err := _InboundChannel.contract.WatchLogs(opts, "RoleGranted", roleRule, accountRule, senderRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(InboundChannelRoleGranted)
				if err := _InboundChannel.contract.UnpackLog(event, "RoleGranted", log); err != nil {
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
func (_InboundChannel *InboundChannelFilterer) ParseRoleGranted(log types.Log) (*InboundChannelRoleGranted, error) {
	event := new(InboundChannelRoleGranted)
	if err := _InboundChannel.contract.UnpackLog(event, "RoleGranted", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// InboundChannelRoleRevokedIterator is returned from FilterRoleRevoked and is used to iterate over the raw logs and unpacked data for RoleRevoked events raised by the InboundChannel contract.
type InboundChannelRoleRevokedIterator struct {
	Event *InboundChannelRoleRevoked // Event containing the contract specifics and raw log

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
func (it *InboundChannelRoleRevokedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(InboundChannelRoleRevoked)
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
		it.Event = new(InboundChannelRoleRevoked)
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
func (it *InboundChannelRoleRevokedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *InboundChannelRoleRevokedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// InboundChannelRoleRevoked represents a RoleRevoked event raised by the InboundChannel contract.
type InboundChannelRoleRevoked struct {
	Role    [32]byte
	Account common.Address
	Sender  common.Address
	Raw     types.Log // Blockchain specific contextual infos
}

// FilterRoleRevoked is a free log retrieval operation binding the contract event 0xf6391f5c32d9c69d2a47ea670b442974b53935d1edc7fd64eb21e047a839171b.
//
// Solidity: event RoleRevoked(bytes32 indexed role, address indexed account, address indexed sender)
func (_InboundChannel *InboundChannelFilterer) FilterRoleRevoked(opts *bind.FilterOpts, role [][32]byte, account []common.Address, sender []common.Address) (*InboundChannelRoleRevokedIterator, error) {

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

	logs, sub, err := _InboundChannel.contract.FilterLogs(opts, "RoleRevoked", roleRule, accountRule, senderRule)
	if err != nil {
		return nil, err
	}
	return &InboundChannelRoleRevokedIterator{contract: _InboundChannel.contract, event: "RoleRevoked", logs: logs, sub: sub}, nil
}

// WatchRoleRevoked is a free log subscription operation binding the contract event 0xf6391f5c32d9c69d2a47ea670b442974b53935d1edc7fd64eb21e047a839171b.
//
// Solidity: event RoleRevoked(bytes32 indexed role, address indexed account, address indexed sender)
func (_InboundChannel *InboundChannelFilterer) WatchRoleRevoked(opts *bind.WatchOpts, sink chan<- *InboundChannelRoleRevoked, role [][32]byte, account []common.Address, sender []common.Address) (event.Subscription, error) {

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

	logs, sub, err := _InboundChannel.contract.WatchLogs(opts, "RoleRevoked", roleRule, accountRule, senderRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(InboundChannelRoleRevoked)
				if err := _InboundChannel.contract.UnpackLog(event, "RoleRevoked", log); err != nil {
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
func (_InboundChannel *InboundChannelFilterer) ParseRoleRevoked(log types.Log) (*InboundChannelRoleRevoked, error) {
	event := new(InboundChannelRoleRevoked)
	if err := _InboundChannel.contract.UnpackLog(event, "RoleRevoked", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// InboundChannelVaultUpdatedIterator is returned from FilterVaultUpdated and is used to iterate over the raw logs and unpacked data for VaultUpdated events raised by the InboundChannel contract.
type InboundChannelVaultUpdatedIterator struct {
	Event *InboundChannelVaultUpdated // Event containing the contract specifics and raw log

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
func (it *InboundChannelVaultUpdatedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(InboundChannelVaultUpdated)
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
		it.Event = new(InboundChannelVaultUpdated)
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
func (it *InboundChannelVaultUpdatedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *InboundChannelVaultUpdatedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// InboundChannelVaultUpdated represents a VaultUpdated event raised by the InboundChannel contract.
type InboundChannelVaultUpdated struct {
	Vault common.Address
	Raw   types.Log // Blockchain specific contextual infos
}

// FilterVaultUpdated is a free log retrieval operation binding the contract event 0x161584aed96e7f34998117c9ad67e2d21ff46d2a42775c22b11ed282f3c7b2cd.
//
// Solidity: event VaultUpdated(address vault)
func (_InboundChannel *InboundChannelFilterer) FilterVaultUpdated(opts *bind.FilterOpts) (*InboundChannelVaultUpdatedIterator, error) {

	logs, sub, err := _InboundChannel.contract.FilterLogs(opts, "VaultUpdated")
	if err != nil {
		return nil, err
	}
	return &InboundChannelVaultUpdatedIterator{contract: _InboundChannel.contract, event: "VaultUpdated", logs: logs, sub: sub}, nil
}

// WatchVaultUpdated is a free log subscription operation binding the contract event 0x161584aed96e7f34998117c9ad67e2d21ff46d2a42775c22b11ed282f3c7b2cd.
//
// Solidity: event VaultUpdated(address vault)
func (_InboundChannel *InboundChannelFilterer) WatchVaultUpdated(opts *bind.WatchOpts, sink chan<- *InboundChannelVaultUpdated) (event.Subscription, error) {

	logs, sub, err := _InboundChannel.contract.WatchLogs(opts, "VaultUpdated")
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(InboundChannelVaultUpdated)
				if err := _InboundChannel.contract.UnpackLog(event, "VaultUpdated", log); err != nil {
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

// ParseVaultUpdated is a log parse operation binding the contract event 0x161584aed96e7f34998117c9ad67e2d21ff46d2a42775c22b11ed282f3c7b2cd.
//
// Solidity: event VaultUpdated(address vault)
func (_InboundChannel *InboundChannelFilterer) ParseVaultUpdated(log types.Log) (*InboundChannelVaultUpdated, error) {
	event := new(InboundChannelVaultUpdated)
	if err := _InboundChannel.contract.UnpackLog(event, "VaultUpdated", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}
