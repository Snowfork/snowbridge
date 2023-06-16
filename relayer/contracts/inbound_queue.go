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

// InboundQueueMessage is an auto generated low-level Go binding around an user-defined struct.
type InboundQueueMessage struct {
	Origin    uint32
	Nonce     uint64
	Recipient [32]byte
	Payload   []byte
}

// InboundQueueMetaData contains all meta data concerning the InboundQueue contract.
var InboundQueueMetaData = &bind.MetaData{
	ABI: "[{\"inputs\":[{\"internalType\":\"contractRegistry\",\"name\":\"registry\",\"type\":\"address\"},{\"internalType\":\"contractIParachainClient\",\"name\":\"_parachainClient\",\"type\":\"address\"},{\"internalType\":\"contractVault\",\"name\":\"_vault\",\"type\":\"address\"},{\"internalType\":\"uint256\",\"name\":\"_reward\",\"type\":\"uint256\"}],\"stateMutability\":\"nonpayable\",\"type\":\"constructor\"},{\"inputs\":[],\"name\":\"InvalidNonce\",\"type\":\"error\"},{\"inputs\":[],\"name\":\"InvalidProof\",\"type\":\"error\"},{\"inputs\":[],\"name\":\"LookupError\",\"type\":\"error\"},{\"inputs\":[],\"name\":\"NotEnoughGas\",\"type\":\"error\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":false,\"internalType\":\"uint256\",\"name\":\"gasToForward\",\"type\":\"uint256\"}],\"name\":\"GasToForwardUpdated\",\"type\":\"event\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":false,\"internalType\":\"uint16\",\"name\":\"id\",\"type\":\"uint16\"},{\"indexed\":false,\"internalType\":\"contractIRecipient\",\"name\":\"handler\",\"type\":\"address\"}],\"name\":\"HandlerUpdated\",\"type\":\"event\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":false,\"internalType\":\"bytes32\",\"name\":\"recipient\",\"type\":\"bytes32\"}],\"name\":\"InvalidRecipient\",\"type\":\"event\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":false,\"internalType\":\"ParaID\",\"name\":\"origin\",\"type\":\"uint32\"},{\"indexed\":false,\"internalType\":\"uint64\",\"name\":\"nonce\",\"type\":\"uint64\"},{\"indexed\":false,\"internalType\":\"enumInboundQueue.DispatchResult\",\"name\":\"result\",\"type\":\"uint8\"}],\"name\":\"MessageDispatched\",\"type\":\"event\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":false,\"internalType\":\"address\",\"name\":\"parachainClient\",\"type\":\"address\"}],\"name\":\"ParachainClientUpdated\",\"type\":\"event\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":false,\"internalType\":\"uint256\",\"name\":\"reward\",\"type\":\"uint256\"}],\"name\":\"RewardUpdated\",\"type\":\"event\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":true,\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\"},{\"indexed\":true,\"internalType\":\"bytes32\",\"name\":\"previousAdminRole\",\"type\":\"bytes32\"},{\"indexed\":true,\"internalType\":\"bytes32\",\"name\":\"newAdminRole\",\"type\":\"bytes32\"}],\"name\":\"RoleAdminChanged\",\"type\":\"event\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":true,\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\"},{\"indexed\":true,\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\"},{\"indexed\":true,\"internalType\":\"address\",\"name\":\"sender\",\"type\":\"address\"}],\"name\":\"RoleGranted\",\"type\":\"event\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":true,\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\"},{\"indexed\":true,\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\"},{\"indexed\":true,\"internalType\":\"address\",\"name\":\"sender\",\"type\":\"address\"}],\"name\":\"RoleRevoked\",\"type\":\"event\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":false,\"internalType\":\"address\",\"name\":\"vault\",\"type\":\"address\"}],\"name\":\"VaultUpdated\",\"type\":\"event\"},{\"inputs\":[],\"name\":\"ADMIN_ROLE\",\"outputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"DEFAULT_ADMIN_ROLE\",\"outputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"GAS_BUFFER\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"gasToForward\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\"}],\"name\":\"getRoleAdmin\",\"outputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\"},{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\"}],\"name\":\"grantRole\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\"},{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\"}],\"name\":\"hasRole\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"ParaID\",\"name\":\"origin\",\"type\":\"uint32\"}],\"name\":\"nonce\",\"outputs\":[{\"internalType\":\"uint64\",\"name\":\"\",\"type\":\"uint64\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"parachainClient\",\"outputs\":[{\"internalType\":\"contractIParachainClient\",\"name\":\"\",\"type\":\"address\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"registry\",\"outputs\":[{\"internalType\":\"contractRegistry\",\"name\":\"\",\"type\":\"address\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\"},{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\"}],\"name\":\"renounceRole\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\"},{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\"}],\"name\":\"revokeRole\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"reward\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"components\":[{\"internalType\":\"ParaID\",\"name\":\"origin\",\"type\":\"uint32\"},{\"internalType\":\"uint64\",\"name\":\"nonce\",\"type\":\"uint64\"},{\"internalType\":\"bytes32\",\"name\":\"recipient\",\"type\":\"bytes32\"},{\"internalType\":\"bytes\",\"name\":\"payload\",\"type\":\"bytes\"}],\"internalType\":\"structInboundQueue.Message\",\"name\":\"message\",\"type\":\"tuple\"},{\"internalType\":\"bytes32[]\",\"name\":\"leafProof\",\"type\":\"bytes32[]\"},{\"internalType\":\"bytes\",\"name\":\"headerProof\",\"type\":\"bytes\"}],\"name\":\"submit\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"bytes4\",\"name\":\"interfaceId\",\"type\":\"bytes4\"}],\"name\":\"supportsInterface\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"_gasToForward\",\"type\":\"uint256\"}],\"name\":\"updateGasToForward\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"_reward\",\"type\":\"uint256\"}],\"name\":\"updateReward\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"vault\",\"outputs\":[{\"internalType\":\"contractVault\",\"name\":\"\",\"type\":\"address\"}],\"stateMutability\":\"view\",\"type\":\"function\"}]",
}

// InboundQueueABI is the input ABI used to generate the binding from.
// Deprecated: Use InboundQueueMetaData.ABI instead.
var InboundQueueABI = InboundQueueMetaData.ABI

// InboundQueue is an auto generated Go binding around an Ethereum contract.
type InboundQueue struct {
	InboundQueueCaller     // Read-only binding to the contract
	InboundQueueTransactor // Write-only binding to the contract
	InboundQueueFilterer   // Log filterer for contract events
}

// InboundQueueCaller is an auto generated read-only Go binding around an Ethereum contract.
type InboundQueueCaller struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// InboundQueueTransactor is an auto generated write-only Go binding around an Ethereum contract.
type InboundQueueTransactor struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// InboundQueueFilterer is an auto generated log filtering Go binding around an Ethereum contract events.
type InboundQueueFilterer struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// InboundQueueSession is an auto generated Go binding around an Ethereum contract,
// with pre-set call and transact options.
type InboundQueueSession struct {
	Contract     *InboundQueue     // Generic contract binding to set the session for
	CallOpts     bind.CallOpts     // Call options to use throughout this session
	TransactOpts bind.TransactOpts // Transaction auth options to use throughout this session
}

// InboundQueueCallerSession is an auto generated read-only Go binding around an Ethereum contract,
// with pre-set call options.
type InboundQueueCallerSession struct {
	Contract *InboundQueueCaller // Generic contract caller binding to set the session for
	CallOpts bind.CallOpts       // Call options to use throughout this session
}

// InboundQueueTransactorSession is an auto generated write-only Go binding around an Ethereum contract,
// with pre-set transact options.
type InboundQueueTransactorSession struct {
	Contract     *InboundQueueTransactor // Generic contract transactor binding to set the session for
	TransactOpts bind.TransactOpts       // Transaction auth options to use throughout this session
}

// InboundQueueRaw is an auto generated low-level Go binding around an Ethereum contract.
type InboundQueueRaw struct {
	Contract *InboundQueue // Generic contract binding to access the raw methods on
}

// InboundQueueCallerRaw is an auto generated low-level read-only Go binding around an Ethereum contract.
type InboundQueueCallerRaw struct {
	Contract *InboundQueueCaller // Generic read-only contract binding to access the raw methods on
}

// InboundQueueTransactorRaw is an auto generated low-level write-only Go binding around an Ethereum contract.
type InboundQueueTransactorRaw struct {
	Contract *InboundQueueTransactor // Generic write-only contract binding to access the raw methods on
}

// NewInboundQueue creates a new instance of InboundQueue, bound to a specific deployed contract.
func NewInboundQueue(address common.Address, backend bind.ContractBackend) (*InboundQueue, error) {
	contract, err := bindInboundQueue(address, backend, backend, backend)
	if err != nil {
		return nil, err
	}
	return &InboundQueue{InboundQueueCaller: InboundQueueCaller{contract: contract}, InboundQueueTransactor: InboundQueueTransactor{contract: contract}, InboundQueueFilterer: InboundQueueFilterer{contract: contract}}, nil
}

// NewInboundQueueCaller creates a new read-only instance of InboundQueue, bound to a specific deployed contract.
func NewInboundQueueCaller(address common.Address, caller bind.ContractCaller) (*InboundQueueCaller, error) {
	contract, err := bindInboundQueue(address, caller, nil, nil)
	if err != nil {
		return nil, err
	}
	return &InboundQueueCaller{contract: contract}, nil
}

// NewInboundQueueTransactor creates a new write-only instance of InboundQueue, bound to a specific deployed contract.
func NewInboundQueueTransactor(address common.Address, transactor bind.ContractTransactor) (*InboundQueueTransactor, error) {
	contract, err := bindInboundQueue(address, nil, transactor, nil)
	if err != nil {
		return nil, err
	}
	return &InboundQueueTransactor{contract: contract}, nil
}

// NewInboundQueueFilterer creates a new log filterer instance of InboundQueue, bound to a specific deployed contract.
func NewInboundQueueFilterer(address common.Address, filterer bind.ContractFilterer) (*InboundQueueFilterer, error) {
	contract, err := bindInboundQueue(address, nil, nil, filterer)
	if err != nil {
		return nil, err
	}
	return &InboundQueueFilterer{contract: contract}, nil
}

// bindInboundQueue binds a generic wrapper to an already deployed contract.
func bindInboundQueue(address common.Address, caller bind.ContractCaller, transactor bind.ContractTransactor, filterer bind.ContractFilterer) (*bind.BoundContract, error) {
	parsed, err := abi.JSON(strings.NewReader(InboundQueueABI))
	if err != nil {
		return nil, err
	}
	return bind.NewBoundContract(address, parsed, caller, transactor, filterer), nil
}

// Call invokes the (constant) contract method with params as input values and
// sets the output to result. The result type might be a single field for simple
// returns, a slice of interfaces for anonymous returns and a struct for named
// returns.
func (_InboundQueue *InboundQueueRaw) Call(opts *bind.CallOpts, result *[]interface{}, method string, params ...interface{}) error {
	return _InboundQueue.Contract.InboundQueueCaller.contract.Call(opts, result, method, params...)
}

// Transfer initiates a plain transaction to move funds to the contract, calling
// its default method if one is available.
func (_InboundQueue *InboundQueueRaw) Transfer(opts *bind.TransactOpts) (*types.Transaction, error) {
	return _InboundQueue.Contract.InboundQueueTransactor.contract.Transfer(opts)
}

// Transact invokes the (paid) contract method with params as input values.
func (_InboundQueue *InboundQueueRaw) Transact(opts *bind.TransactOpts, method string, params ...interface{}) (*types.Transaction, error) {
	return _InboundQueue.Contract.InboundQueueTransactor.contract.Transact(opts, method, params...)
}

// Call invokes the (constant) contract method with params as input values and
// sets the output to result. The result type might be a single field for simple
// returns, a slice of interfaces for anonymous returns and a struct for named
// returns.
func (_InboundQueue *InboundQueueCallerRaw) Call(opts *bind.CallOpts, result *[]interface{}, method string, params ...interface{}) error {
	return _InboundQueue.Contract.contract.Call(opts, result, method, params...)
}

// Transfer initiates a plain transaction to move funds to the contract, calling
// its default method if one is available.
func (_InboundQueue *InboundQueueTransactorRaw) Transfer(opts *bind.TransactOpts) (*types.Transaction, error) {
	return _InboundQueue.Contract.contract.Transfer(opts)
}

// Transact invokes the (paid) contract method with params as input values.
func (_InboundQueue *InboundQueueTransactorRaw) Transact(opts *bind.TransactOpts, method string, params ...interface{}) (*types.Transaction, error) {
	return _InboundQueue.Contract.contract.Transact(opts, method, params...)
}

// ADMINROLE is a free data retrieval call binding the contract method 0x75b238fc.
//
// Solidity: function ADMIN_ROLE() view returns(bytes32)
func (_InboundQueue *InboundQueueCaller) ADMINROLE(opts *bind.CallOpts) ([32]byte, error) {
	var out []interface{}
	err := _InboundQueue.contract.Call(opts, &out, "ADMIN_ROLE")

	if err != nil {
		return *new([32]byte), err
	}

	out0 := *abi.ConvertType(out[0], new([32]byte)).(*[32]byte)

	return out0, err

}

// ADMINROLE is a free data retrieval call binding the contract method 0x75b238fc.
//
// Solidity: function ADMIN_ROLE() view returns(bytes32)
func (_InboundQueue *InboundQueueSession) ADMINROLE() ([32]byte, error) {
	return _InboundQueue.Contract.ADMINROLE(&_InboundQueue.CallOpts)
}

// ADMINROLE is a free data retrieval call binding the contract method 0x75b238fc.
//
// Solidity: function ADMIN_ROLE() view returns(bytes32)
func (_InboundQueue *InboundQueueCallerSession) ADMINROLE() ([32]byte, error) {
	return _InboundQueue.Contract.ADMINROLE(&_InboundQueue.CallOpts)
}

// DEFAULTADMINROLE is a free data retrieval call binding the contract method 0xa217fddf.
//
// Solidity: function DEFAULT_ADMIN_ROLE() view returns(bytes32)
func (_InboundQueue *InboundQueueCaller) DEFAULTADMINROLE(opts *bind.CallOpts) ([32]byte, error) {
	var out []interface{}
	err := _InboundQueue.contract.Call(opts, &out, "DEFAULT_ADMIN_ROLE")

	if err != nil {
		return *new([32]byte), err
	}

	out0 := *abi.ConvertType(out[0], new([32]byte)).(*[32]byte)

	return out0, err

}

// DEFAULTADMINROLE is a free data retrieval call binding the contract method 0xa217fddf.
//
// Solidity: function DEFAULT_ADMIN_ROLE() view returns(bytes32)
func (_InboundQueue *InboundQueueSession) DEFAULTADMINROLE() ([32]byte, error) {
	return _InboundQueue.Contract.DEFAULTADMINROLE(&_InboundQueue.CallOpts)
}

// DEFAULTADMINROLE is a free data retrieval call binding the contract method 0xa217fddf.
//
// Solidity: function DEFAULT_ADMIN_ROLE() view returns(bytes32)
func (_InboundQueue *InboundQueueCallerSession) DEFAULTADMINROLE() ([32]byte, error) {
	return _InboundQueue.Contract.DEFAULTADMINROLE(&_InboundQueue.CallOpts)
}

// GASBUFFER is a free data retrieval call binding the contract method 0xc7e67360.
//
// Solidity: function GAS_BUFFER() view returns(uint256)
func (_InboundQueue *InboundQueueCaller) GASBUFFER(opts *bind.CallOpts) (*big.Int, error) {
	var out []interface{}
	err := _InboundQueue.contract.Call(opts, &out, "GAS_BUFFER")

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// GASBUFFER is a free data retrieval call binding the contract method 0xc7e67360.
//
// Solidity: function GAS_BUFFER() view returns(uint256)
func (_InboundQueue *InboundQueueSession) GASBUFFER() (*big.Int, error) {
	return _InboundQueue.Contract.GASBUFFER(&_InboundQueue.CallOpts)
}

// GASBUFFER is a free data retrieval call binding the contract method 0xc7e67360.
//
// Solidity: function GAS_BUFFER() view returns(uint256)
func (_InboundQueue *InboundQueueCallerSession) GASBUFFER() (*big.Int, error) {
	return _InboundQueue.Contract.GASBUFFER(&_InboundQueue.CallOpts)
}

// GasToForward is a free data retrieval call binding the contract method 0x3b834210.
//
// Solidity: function gasToForward() view returns(uint256)
func (_InboundQueue *InboundQueueCaller) GasToForward(opts *bind.CallOpts) (*big.Int, error) {
	var out []interface{}
	err := _InboundQueue.contract.Call(opts, &out, "gasToForward")

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// GasToForward is a free data retrieval call binding the contract method 0x3b834210.
//
// Solidity: function gasToForward() view returns(uint256)
func (_InboundQueue *InboundQueueSession) GasToForward() (*big.Int, error) {
	return _InboundQueue.Contract.GasToForward(&_InboundQueue.CallOpts)
}

// GasToForward is a free data retrieval call binding the contract method 0x3b834210.
//
// Solidity: function gasToForward() view returns(uint256)
func (_InboundQueue *InboundQueueCallerSession) GasToForward() (*big.Int, error) {
	return _InboundQueue.Contract.GasToForward(&_InboundQueue.CallOpts)
}

// GetRoleAdmin is a free data retrieval call binding the contract method 0x248a9ca3.
//
// Solidity: function getRoleAdmin(bytes32 role) view returns(bytes32)
func (_InboundQueue *InboundQueueCaller) GetRoleAdmin(opts *bind.CallOpts, role [32]byte) ([32]byte, error) {
	var out []interface{}
	err := _InboundQueue.contract.Call(opts, &out, "getRoleAdmin", role)

	if err != nil {
		return *new([32]byte), err
	}

	out0 := *abi.ConvertType(out[0], new([32]byte)).(*[32]byte)

	return out0, err

}

// GetRoleAdmin is a free data retrieval call binding the contract method 0x248a9ca3.
//
// Solidity: function getRoleAdmin(bytes32 role) view returns(bytes32)
func (_InboundQueue *InboundQueueSession) GetRoleAdmin(role [32]byte) ([32]byte, error) {
	return _InboundQueue.Contract.GetRoleAdmin(&_InboundQueue.CallOpts, role)
}

// GetRoleAdmin is a free data retrieval call binding the contract method 0x248a9ca3.
//
// Solidity: function getRoleAdmin(bytes32 role) view returns(bytes32)
func (_InboundQueue *InboundQueueCallerSession) GetRoleAdmin(role [32]byte) ([32]byte, error) {
	return _InboundQueue.Contract.GetRoleAdmin(&_InboundQueue.CallOpts, role)
}

// HasRole is a free data retrieval call binding the contract method 0x91d14854.
//
// Solidity: function hasRole(bytes32 role, address account) view returns(bool)
func (_InboundQueue *InboundQueueCaller) HasRole(opts *bind.CallOpts, role [32]byte, account common.Address) (bool, error) {
	var out []interface{}
	err := _InboundQueue.contract.Call(opts, &out, "hasRole", role, account)

	if err != nil {
		return *new(bool), err
	}

	out0 := *abi.ConvertType(out[0], new(bool)).(*bool)

	return out0, err

}

// HasRole is a free data retrieval call binding the contract method 0x91d14854.
//
// Solidity: function hasRole(bytes32 role, address account) view returns(bool)
func (_InboundQueue *InboundQueueSession) HasRole(role [32]byte, account common.Address) (bool, error) {
	return _InboundQueue.Contract.HasRole(&_InboundQueue.CallOpts, role, account)
}

// HasRole is a free data retrieval call binding the contract method 0x91d14854.
//
// Solidity: function hasRole(bytes32 role, address account) view returns(bool)
func (_InboundQueue *InboundQueueCallerSession) HasRole(role [32]byte, account common.Address) (bool, error) {
	return _InboundQueue.Contract.HasRole(&_InboundQueue.CallOpts, role, account)
}

// Nonce is a free data retrieval call binding the contract method 0x141c4985.
//
// Solidity: function nonce(uint32 origin) view returns(uint64)
func (_InboundQueue *InboundQueueCaller) Nonce(opts *bind.CallOpts, origin uint32) (uint64, error) {
	var out []interface{}
	err := _InboundQueue.contract.Call(opts, &out, "nonce", origin)

	if err != nil {
		return *new(uint64), err
	}

	out0 := *abi.ConvertType(out[0], new(uint64)).(*uint64)

	return out0, err

}

// Nonce is a free data retrieval call binding the contract method 0x141c4985.
//
// Solidity: function nonce(uint32 origin) view returns(uint64)
func (_InboundQueue *InboundQueueSession) Nonce(origin uint32) (uint64, error) {
	return _InboundQueue.Contract.Nonce(&_InboundQueue.CallOpts, origin)
}

// Nonce is a free data retrieval call binding the contract method 0x141c4985.
//
// Solidity: function nonce(uint32 origin) view returns(uint64)
func (_InboundQueue *InboundQueueCallerSession) Nonce(origin uint32) (uint64, error) {
	return _InboundQueue.Contract.Nonce(&_InboundQueue.CallOpts, origin)
}

// ParachainClient is a free data retrieval call binding the contract method 0x1674f9b7.
//
// Solidity: function parachainClient() view returns(address)
func (_InboundQueue *InboundQueueCaller) ParachainClient(opts *bind.CallOpts) (common.Address, error) {
	var out []interface{}
	err := _InboundQueue.contract.Call(opts, &out, "parachainClient")

	if err != nil {
		return *new(common.Address), err
	}

	out0 := *abi.ConvertType(out[0], new(common.Address)).(*common.Address)

	return out0, err

}

// ParachainClient is a free data retrieval call binding the contract method 0x1674f9b7.
//
// Solidity: function parachainClient() view returns(address)
func (_InboundQueue *InboundQueueSession) ParachainClient() (common.Address, error) {
	return _InboundQueue.Contract.ParachainClient(&_InboundQueue.CallOpts)
}

// ParachainClient is a free data retrieval call binding the contract method 0x1674f9b7.
//
// Solidity: function parachainClient() view returns(address)
func (_InboundQueue *InboundQueueCallerSession) ParachainClient() (common.Address, error) {
	return _InboundQueue.Contract.ParachainClient(&_InboundQueue.CallOpts)
}

// Registry is a free data retrieval call binding the contract method 0x7b103999.
//
// Solidity: function registry() view returns(address)
func (_InboundQueue *InboundQueueCaller) Registry(opts *bind.CallOpts) (common.Address, error) {
	var out []interface{}
	err := _InboundQueue.contract.Call(opts, &out, "registry")

	if err != nil {
		return *new(common.Address), err
	}

	out0 := *abi.ConvertType(out[0], new(common.Address)).(*common.Address)

	return out0, err

}

// Registry is a free data retrieval call binding the contract method 0x7b103999.
//
// Solidity: function registry() view returns(address)
func (_InboundQueue *InboundQueueSession) Registry() (common.Address, error) {
	return _InboundQueue.Contract.Registry(&_InboundQueue.CallOpts)
}

// Registry is a free data retrieval call binding the contract method 0x7b103999.
//
// Solidity: function registry() view returns(address)
func (_InboundQueue *InboundQueueCallerSession) Registry() (common.Address, error) {
	return _InboundQueue.Contract.Registry(&_InboundQueue.CallOpts)
}

// Reward is a free data retrieval call binding the contract method 0x228cb733.
//
// Solidity: function reward() view returns(uint256)
func (_InboundQueue *InboundQueueCaller) Reward(opts *bind.CallOpts) (*big.Int, error) {
	var out []interface{}
	err := _InboundQueue.contract.Call(opts, &out, "reward")

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// Reward is a free data retrieval call binding the contract method 0x228cb733.
//
// Solidity: function reward() view returns(uint256)
func (_InboundQueue *InboundQueueSession) Reward() (*big.Int, error) {
	return _InboundQueue.Contract.Reward(&_InboundQueue.CallOpts)
}

// Reward is a free data retrieval call binding the contract method 0x228cb733.
//
// Solidity: function reward() view returns(uint256)
func (_InboundQueue *InboundQueueCallerSession) Reward() (*big.Int, error) {
	return _InboundQueue.Contract.Reward(&_InboundQueue.CallOpts)
}

// SupportsInterface is a free data retrieval call binding the contract method 0x01ffc9a7.
//
// Solidity: function supportsInterface(bytes4 interfaceId) view returns(bool)
func (_InboundQueue *InboundQueueCaller) SupportsInterface(opts *bind.CallOpts, interfaceId [4]byte) (bool, error) {
	var out []interface{}
	err := _InboundQueue.contract.Call(opts, &out, "supportsInterface", interfaceId)

	if err != nil {
		return *new(bool), err
	}

	out0 := *abi.ConvertType(out[0], new(bool)).(*bool)

	return out0, err

}

// SupportsInterface is a free data retrieval call binding the contract method 0x01ffc9a7.
//
// Solidity: function supportsInterface(bytes4 interfaceId) view returns(bool)
func (_InboundQueue *InboundQueueSession) SupportsInterface(interfaceId [4]byte) (bool, error) {
	return _InboundQueue.Contract.SupportsInterface(&_InboundQueue.CallOpts, interfaceId)
}

// SupportsInterface is a free data retrieval call binding the contract method 0x01ffc9a7.
//
// Solidity: function supportsInterface(bytes4 interfaceId) view returns(bool)
func (_InboundQueue *InboundQueueCallerSession) SupportsInterface(interfaceId [4]byte) (bool, error) {
	return _InboundQueue.Contract.SupportsInterface(&_InboundQueue.CallOpts, interfaceId)
}

// Vault is a free data retrieval call binding the contract method 0xfbfa77cf.
//
// Solidity: function vault() view returns(address)
func (_InboundQueue *InboundQueueCaller) Vault(opts *bind.CallOpts) (common.Address, error) {
	var out []interface{}
	err := _InboundQueue.contract.Call(opts, &out, "vault")

	if err != nil {
		return *new(common.Address), err
	}

	out0 := *abi.ConvertType(out[0], new(common.Address)).(*common.Address)

	return out0, err

}

// Vault is a free data retrieval call binding the contract method 0xfbfa77cf.
//
// Solidity: function vault() view returns(address)
func (_InboundQueue *InboundQueueSession) Vault() (common.Address, error) {
	return _InboundQueue.Contract.Vault(&_InboundQueue.CallOpts)
}

// Vault is a free data retrieval call binding the contract method 0xfbfa77cf.
//
// Solidity: function vault() view returns(address)
func (_InboundQueue *InboundQueueCallerSession) Vault() (common.Address, error) {
	return _InboundQueue.Contract.Vault(&_InboundQueue.CallOpts)
}

// GrantRole is a paid mutator transaction binding the contract method 0x2f2ff15d.
//
// Solidity: function grantRole(bytes32 role, address account) returns()
func (_InboundQueue *InboundQueueTransactor) GrantRole(opts *bind.TransactOpts, role [32]byte, account common.Address) (*types.Transaction, error) {
	return _InboundQueue.contract.Transact(opts, "grantRole", role, account)
}

// GrantRole is a paid mutator transaction binding the contract method 0x2f2ff15d.
//
// Solidity: function grantRole(bytes32 role, address account) returns()
func (_InboundQueue *InboundQueueSession) GrantRole(role [32]byte, account common.Address) (*types.Transaction, error) {
	return _InboundQueue.Contract.GrantRole(&_InboundQueue.TransactOpts, role, account)
}

// GrantRole is a paid mutator transaction binding the contract method 0x2f2ff15d.
//
// Solidity: function grantRole(bytes32 role, address account) returns()
func (_InboundQueue *InboundQueueTransactorSession) GrantRole(role [32]byte, account common.Address) (*types.Transaction, error) {
	return _InboundQueue.Contract.GrantRole(&_InboundQueue.TransactOpts, role, account)
}

// RenounceRole is a paid mutator transaction binding the contract method 0x36568abe.
//
// Solidity: function renounceRole(bytes32 role, address account) returns()
func (_InboundQueue *InboundQueueTransactor) RenounceRole(opts *bind.TransactOpts, role [32]byte, account common.Address) (*types.Transaction, error) {
	return _InboundQueue.contract.Transact(opts, "renounceRole", role, account)
}

// RenounceRole is a paid mutator transaction binding the contract method 0x36568abe.
//
// Solidity: function renounceRole(bytes32 role, address account) returns()
func (_InboundQueue *InboundQueueSession) RenounceRole(role [32]byte, account common.Address) (*types.Transaction, error) {
	return _InboundQueue.Contract.RenounceRole(&_InboundQueue.TransactOpts, role, account)
}

// RenounceRole is a paid mutator transaction binding the contract method 0x36568abe.
//
// Solidity: function renounceRole(bytes32 role, address account) returns()
func (_InboundQueue *InboundQueueTransactorSession) RenounceRole(role [32]byte, account common.Address) (*types.Transaction, error) {
	return _InboundQueue.Contract.RenounceRole(&_InboundQueue.TransactOpts, role, account)
}

// RevokeRole is a paid mutator transaction binding the contract method 0xd547741f.
//
// Solidity: function revokeRole(bytes32 role, address account) returns()
func (_InboundQueue *InboundQueueTransactor) RevokeRole(opts *bind.TransactOpts, role [32]byte, account common.Address) (*types.Transaction, error) {
	return _InboundQueue.contract.Transact(opts, "revokeRole", role, account)
}

// RevokeRole is a paid mutator transaction binding the contract method 0xd547741f.
//
// Solidity: function revokeRole(bytes32 role, address account) returns()
func (_InboundQueue *InboundQueueSession) RevokeRole(role [32]byte, account common.Address) (*types.Transaction, error) {
	return _InboundQueue.Contract.RevokeRole(&_InboundQueue.TransactOpts, role, account)
}

// RevokeRole is a paid mutator transaction binding the contract method 0xd547741f.
//
// Solidity: function revokeRole(bytes32 role, address account) returns()
func (_InboundQueue *InboundQueueTransactorSession) RevokeRole(role [32]byte, account common.Address) (*types.Transaction, error) {
	return _InboundQueue.Contract.RevokeRole(&_InboundQueue.TransactOpts, role, account)
}

// Submit is a paid mutator transaction binding the contract method 0x90d7fbe9.
//
// Solidity: function submit((uint32,uint64,bytes32,bytes) message, bytes32[] leafProof, bytes headerProof) returns()
func (_InboundQueue *InboundQueueTransactor) Submit(opts *bind.TransactOpts, message InboundQueueMessage, leafProof [][32]byte, headerProof []byte) (*types.Transaction, error) {
	return _InboundQueue.contract.Transact(opts, "submit", message, leafProof, headerProof)
}

// Submit is a paid mutator transaction binding the contract method 0x90d7fbe9.
//
// Solidity: function submit((uint32,uint64,bytes32,bytes) message, bytes32[] leafProof, bytes headerProof) returns()
func (_InboundQueue *InboundQueueSession) Submit(message InboundQueueMessage, leafProof [][32]byte, headerProof []byte) (*types.Transaction, error) {
	return _InboundQueue.Contract.Submit(&_InboundQueue.TransactOpts, message, leafProof, headerProof)
}

// Submit is a paid mutator transaction binding the contract method 0x90d7fbe9.
//
// Solidity: function submit((uint32,uint64,bytes32,bytes) message, bytes32[] leafProof, bytes headerProof) returns()
func (_InboundQueue *InboundQueueTransactorSession) Submit(message InboundQueueMessage, leafProof [][32]byte, headerProof []byte) (*types.Transaction, error) {
	return _InboundQueue.Contract.Submit(&_InboundQueue.TransactOpts, message, leafProof, headerProof)
}

// UpdateGasToForward is a paid mutator transaction binding the contract method 0xf4d9d4c4.
//
// Solidity: function updateGasToForward(uint256 _gasToForward) returns()
func (_InboundQueue *InboundQueueTransactor) UpdateGasToForward(opts *bind.TransactOpts, _gasToForward *big.Int) (*types.Transaction, error) {
	return _InboundQueue.contract.Transact(opts, "updateGasToForward", _gasToForward)
}

// UpdateGasToForward is a paid mutator transaction binding the contract method 0xf4d9d4c4.
//
// Solidity: function updateGasToForward(uint256 _gasToForward) returns()
func (_InboundQueue *InboundQueueSession) UpdateGasToForward(_gasToForward *big.Int) (*types.Transaction, error) {
	return _InboundQueue.Contract.UpdateGasToForward(&_InboundQueue.TransactOpts, _gasToForward)
}

// UpdateGasToForward is a paid mutator transaction binding the contract method 0xf4d9d4c4.
//
// Solidity: function updateGasToForward(uint256 _gasToForward) returns()
func (_InboundQueue *InboundQueueTransactorSession) UpdateGasToForward(_gasToForward *big.Int) (*types.Transaction, error) {
	return _InboundQueue.Contract.UpdateGasToForward(&_InboundQueue.TransactOpts, _gasToForward)
}

// UpdateReward is a paid mutator transaction binding the contract method 0x425c8abd.
//
// Solidity: function updateReward(uint256 _reward) returns()
func (_InboundQueue *InboundQueueTransactor) UpdateReward(opts *bind.TransactOpts, _reward *big.Int) (*types.Transaction, error) {
	return _InboundQueue.contract.Transact(opts, "updateReward", _reward)
}

// UpdateReward is a paid mutator transaction binding the contract method 0x425c8abd.
//
// Solidity: function updateReward(uint256 _reward) returns()
func (_InboundQueue *InboundQueueSession) UpdateReward(_reward *big.Int) (*types.Transaction, error) {
	return _InboundQueue.Contract.UpdateReward(&_InboundQueue.TransactOpts, _reward)
}

// UpdateReward is a paid mutator transaction binding the contract method 0x425c8abd.
//
// Solidity: function updateReward(uint256 _reward) returns()
func (_InboundQueue *InboundQueueTransactorSession) UpdateReward(_reward *big.Int) (*types.Transaction, error) {
	return _InboundQueue.Contract.UpdateReward(&_InboundQueue.TransactOpts, _reward)
}

// InboundQueueGasToForwardUpdatedIterator is returned from FilterGasToForwardUpdated and is used to iterate over the raw logs and unpacked data for GasToForwardUpdated events raised by the InboundQueue contract.
type InboundQueueGasToForwardUpdatedIterator struct {
	Event *InboundQueueGasToForwardUpdated // Event containing the contract specifics and raw log

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
func (it *InboundQueueGasToForwardUpdatedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(InboundQueueGasToForwardUpdated)
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
		it.Event = new(InboundQueueGasToForwardUpdated)
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
func (it *InboundQueueGasToForwardUpdatedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *InboundQueueGasToForwardUpdatedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// InboundQueueGasToForwardUpdated represents a GasToForwardUpdated event raised by the InboundQueue contract.
type InboundQueueGasToForwardUpdated struct {
	GasToForward *big.Int
	Raw          types.Log // Blockchain specific contextual infos
}

// FilterGasToForwardUpdated is a free log retrieval operation binding the contract event 0x100f863b8d0f17ca3e98e1ee23dc6feb2a31f24f0836e89e21bcb9f0bfc2d742.
//
// Solidity: event GasToForwardUpdated(uint256 gasToForward)
func (_InboundQueue *InboundQueueFilterer) FilterGasToForwardUpdated(opts *bind.FilterOpts) (*InboundQueueGasToForwardUpdatedIterator, error) {

	logs, sub, err := _InboundQueue.contract.FilterLogs(opts, "GasToForwardUpdated")
	if err != nil {
		return nil, err
	}
	return &InboundQueueGasToForwardUpdatedIterator{contract: _InboundQueue.contract, event: "GasToForwardUpdated", logs: logs, sub: sub}, nil
}

// WatchGasToForwardUpdated is a free log subscription operation binding the contract event 0x100f863b8d0f17ca3e98e1ee23dc6feb2a31f24f0836e89e21bcb9f0bfc2d742.
//
// Solidity: event GasToForwardUpdated(uint256 gasToForward)
func (_InboundQueue *InboundQueueFilterer) WatchGasToForwardUpdated(opts *bind.WatchOpts, sink chan<- *InboundQueueGasToForwardUpdated) (event.Subscription, error) {

	logs, sub, err := _InboundQueue.contract.WatchLogs(opts, "GasToForwardUpdated")
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(InboundQueueGasToForwardUpdated)
				if err := _InboundQueue.contract.UnpackLog(event, "GasToForwardUpdated", log); err != nil {
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

// ParseGasToForwardUpdated is a log parse operation binding the contract event 0x100f863b8d0f17ca3e98e1ee23dc6feb2a31f24f0836e89e21bcb9f0bfc2d742.
//
// Solidity: event GasToForwardUpdated(uint256 gasToForward)
func (_InboundQueue *InboundQueueFilterer) ParseGasToForwardUpdated(log types.Log) (*InboundQueueGasToForwardUpdated, error) {
	event := new(InboundQueueGasToForwardUpdated)
	if err := _InboundQueue.contract.UnpackLog(event, "GasToForwardUpdated", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// InboundQueueHandlerUpdatedIterator is returned from FilterHandlerUpdated and is used to iterate over the raw logs and unpacked data for HandlerUpdated events raised by the InboundQueue contract.
type InboundQueueHandlerUpdatedIterator struct {
	Event *InboundQueueHandlerUpdated // Event containing the contract specifics and raw log

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
func (it *InboundQueueHandlerUpdatedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(InboundQueueHandlerUpdated)
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
		it.Event = new(InboundQueueHandlerUpdated)
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
func (it *InboundQueueHandlerUpdatedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *InboundQueueHandlerUpdatedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// InboundQueueHandlerUpdated represents a HandlerUpdated event raised by the InboundQueue contract.
type InboundQueueHandlerUpdated struct {
	Id      uint16
	Handler common.Address
	Raw     types.Log // Blockchain specific contextual infos
}

// FilterHandlerUpdated is a free log retrieval operation binding the contract event 0x3ed918ab4515fda74eadc324a45e89778c4e8bfc6391f78db7beac41b768df74.
//
// Solidity: event HandlerUpdated(uint16 id, address handler)
func (_InboundQueue *InboundQueueFilterer) FilterHandlerUpdated(opts *bind.FilterOpts) (*InboundQueueHandlerUpdatedIterator, error) {

	logs, sub, err := _InboundQueue.contract.FilterLogs(opts, "HandlerUpdated")
	if err != nil {
		return nil, err
	}
	return &InboundQueueHandlerUpdatedIterator{contract: _InboundQueue.contract, event: "HandlerUpdated", logs: logs, sub: sub}, nil
}

// WatchHandlerUpdated is a free log subscription operation binding the contract event 0x3ed918ab4515fda74eadc324a45e89778c4e8bfc6391f78db7beac41b768df74.
//
// Solidity: event HandlerUpdated(uint16 id, address handler)
func (_InboundQueue *InboundQueueFilterer) WatchHandlerUpdated(opts *bind.WatchOpts, sink chan<- *InboundQueueHandlerUpdated) (event.Subscription, error) {

	logs, sub, err := _InboundQueue.contract.WatchLogs(opts, "HandlerUpdated")
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(InboundQueueHandlerUpdated)
				if err := _InboundQueue.contract.UnpackLog(event, "HandlerUpdated", log); err != nil {
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

// ParseHandlerUpdated is a log parse operation binding the contract event 0x3ed918ab4515fda74eadc324a45e89778c4e8bfc6391f78db7beac41b768df74.
//
// Solidity: event HandlerUpdated(uint16 id, address handler)
func (_InboundQueue *InboundQueueFilterer) ParseHandlerUpdated(log types.Log) (*InboundQueueHandlerUpdated, error) {
	event := new(InboundQueueHandlerUpdated)
	if err := _InboundQueue.contract.UnpackLog(event, "HandlerUpdated", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// InboundQueueInvalidRecipientIterator is returned from FilterInvalidRecipient and is used to iterate over the raw logs and unpacked data for InvalidRecipient events raised by the InboundQueue contract.
type InboundQueueInvalidRecipientIterator struct {
	Event *InboundQueueInvalidRecipient // Event containing the contract specifics and raw log

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
func (it *InboundQueueInvalidRecipientIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(InboundQueueInvalidRecipient)
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
		it.Event = new(InboundQueueInvalidRecipient)
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
func (it *InboundQueueInvalidRecipientIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *InboundQueueInvalidRecipientIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// InboundQueueInvalidRecipient represents a InvalidRecipient event raised by the InboundQueue contract.
type InboundQueueInvalidRecipient struct {
	Recipient [32]byte
	Raw       types.Log // Blockchain specific contextual infos
}

// FilterInvalidRecipient is a free log retrieval operation binding the contract event 0xbf0b3f6242271405146290163e141ff674b9d85a2a16815a195bb05e3d57c835.
//
// Solidity: event InvalidRecipient(bytes32 recipient)
func (_InboundQueue *InboundQueueFilterer) FilterInvalidRecipient(opts *bind.FilterOpts) (*InboundQueueInvalidRecipientIterator, error) {

	logs, sub, err := _InboundQueue.contract.FilterLogs(opts, "InvalidRecipient")
	if err != nil {
		return nil, err
	}
	return &InboundQueueInvalidRecipientIterator{contract: _InboundQueue.contract, event: "InvalidRecipient", logs: logs, sub: sub}, nil
}

// WatchInvalidRecipient is a free log subscription operation binding the contract event 0xbf0b3f6242271405146290163e141ff674b9d85a2a16815a195bb05e3d57c835.
//
// Solidity: event InvalidRecipient(bytes32 recipient)
func (_InboundQueue *InboundQueueFilterer) WatchInvalidRecipient(opts *bind.WatchOpts, sink chan<- *InboundQueueInvalidRecipient) (event.Subscription, error) {

	logs, sub, err := _InboundQueue.contract.WatchLogs(opts, "InvalidRecipient")
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(InboundQueueInvalidRecipient)
				if err := _InboundQueue.contract.UnpackLog(event, "InvalidRecipient", log); err != nil {
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

// ParseInvalidRecipient is a log parse operation binding the contract event 0xbf0b3f6242271405146290163e141ff674b9d85a2a16815a195bb05e3d57c835.
//
// Solidity: event InvalidRecipient(bytes32 recipient)
func (_InboundQueue *InboundQueueFilterer) ParseInvalidRecipient(log types.Log) (*InboundQueueInvalidRecipient, error) {
	event := new(InboundQueueInvalidRecipient)
	if err := _InboundQueue.contract.UnpackLog(event, "InvalidRecipient", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// InboundQueueMessageDispatchedIterator is returned from FilterMessageDispatched and is used to iterate over the raw logs and unpacked data for MessageDispatched events raised by the InboundQueue contract.
type InboundQueueMessageDispatchedIterator struct {
	Event *InboundQueueMessageDispatched // Event containing the contract specifics and raw log

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
func (it *InboundQueueMessageDispatchedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(InboundQueueMessageDispatched)
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
		it.Event = new(InboundQueueMessageDispatched)
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
func (it *InboundQueueMessageDispatchedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *InboundQueueMessageDispatchedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// InboundQueueMessageDispatched represents a MessageDispatched event raised by the InboundQueue contract.
type InboundQueueMessageDispatched struct {
	Origin uint32
	Nonce  uint64
	Result uint8
	Raw    types.Log // Blockchain specific contextual infos
}

// FilterMessageDispatched is a free log retrieval operation binding the contract event 0x3daaaf6b5c13966eb060b53daff310d82d35bdd2e539be4dc92dfe1310ee170d.
//
// Solidity: event MessageDispatched(uint32 origin, uint64 nonce, uint8 result)
func (_InboundQueue *InboundQueueFilterer) FilterMessageDispatched(opts *bind.FilterOpts) (*InboundQueueMessageDispatchedIterator, error) {

	logs, sub, err := _InboundQueue.contract.FilterLogs(opts, "MessageDispatched")
	if err != nil {
		return nil, err
	}
	return &InboundQueueMessageDispatchedIterator{contract: _InboundQueue.contract, event: "MessageDispatched", logs: logs, sub: sub}, nil
}

// WatchMessageDispatched is a free log subscription operation binding the contract event 0x3daaaf6b5c13966eb060b53daff310d82d35bdd2e539be4dc92dfe1310ee170d.
//
// Solidity: event MessageDispatched(uint32 origin, uint64 nonce, uint8 result)
func (_InboundQueue *InboundQueueFilterer) WatchMessageDispatched(opts *bind.WatchOpts, sink chan<- *InboundQueueMessageDispatched) (event.Subscription, error) {

	logs, sub, err := _InboundQueue.contract.WatchLogs(opts, "MessageDispatched")
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(InboundQueueMessageDispatched)
				if err := _InboundQueue.contract.UnpackLog(event, "MessageDispatched", log); err != nil {
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

// ParseMessageDispatched is a log parse operation binding the contract event 0x3daaaf6b5c13966eb060b53daff310d82d35bdd2e539be4dc92dfe1310ee170d.
//
// Solidity: event MessageDispatched(uint32 origin, uint64 nonce, uint8 result)
func (_InboundQueue *InboundQueueFilterer) ParseMessageDispatched(log types.Log) (*InboundQueueMessageDispatched, error) {
	event := new(InboundQueueMessageDispatched)
	if err := _InboundQueue.contract.UnpackLog(event, "MessageDispatched", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// InboundQueueParachainClientUpdatedIterator is returned from FilterParachainClientUpdated and is used to iterate over the raw logs and unpacked data for ParachainClientUpdated events raised by the InboundQueue contract.
type InboundQueueParachainClientUpdatedIterator struct {
	Event *InboundQueueParachainClientUpdated // Event containing the contract specifics and raw log

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
func (it *InboundQueueParachainClientUpdatedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(InboundQueueParachainClientUpdated)
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
		it.Event = new(InboundQueueParachainClientUpdated)
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
func (it *InboundQueueParachainClientUpdatedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *InboundQueueParachainClientUpdatedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// InboundQueueParachainClientUpdated represents a ParachainClientUpdated event raised by the InboundQueue contract.
type InboundQueueParachainClientUpdated struct {
	ParachainClient common.Address
	Raw             types.Log // Blockchain specific contextual infos
}

// FilterParachainClientUpdated is a free log retrieval operation binding the contract event 0x2eb3efb9388b586c251856bc1b31af67a4015796cf665e600df15d2a42e2ba41.
//
// Solidity: event ParachainClientUpdated(address parachainClient)
func (_InboundQueue *InboundQueueFilterer) FilterParachainClientUpdated(opts *bind.FilterOpts) (*InboundQueueParachainClientUpdatedIterator, error) {

	logs, sub, err := _InboundQueue.contract.FilterLogs(opts, "ParachainClientUpdated")
	if err != nil {
		return nil, err
	}
	return &InboundQueueParachainClientUpdatedIterator{contract: _InboundQueue.contract, event: "ParachainClientUpdated", logs: logs, sub: sub}, nil
}

// WatchParachainClientUpdated is a free log subscription operation binding the contract event 0x2eb3efb9388b586c251856bc1b31af67a4015796cf665e600df15d2a42e2ba41.
//
// Solidity: event ParachainClientUpdated(address parachainClient)
func (_InboundQueue *InboundQueueFilterer) WatchParachainClientUpdated(opts *bind.WatchOpts, sink chan<- *InboundQueueParachainClientUpdated) (event.Subscription, error) {

	logs, sub, err := _InboundQueue.contract.WatchLogs(opts, "ParachainClientUpdated")
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(InboundQueueParachainClientUpdated)
				if err := _InboundQueue.contract.UnpackLog(event, "ParachainClientUpdated", log); err != nil {
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
func (_InboundQueue *InboundQueueFilterer) ParseParachainClientUpdated(log types.Log) (*InboundQueueParachainClientUpdated, error) {
	event := new(InboundQueueParachainClientUpdated)
	if err := _InboundQueue.contract.UnpackLog(event, "ParachainClientUpdated", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// InboundQueueRewardUpdatedIterator is returned from FilterRewardUpdated and is used to iterate over the raw logs and unpacked data for RewardUpdated events raised by the InboundQueue contract.
type InboundQueueRewardUpdatedIterator struct {
	Event *InboundQueueRewardUpdated // Event containing the contract specifics and raw log

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
func (it *InboundQueueRewardUpdatedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(InboundQueueRewardUpdated)
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
		it.Event = new(InboundQueueRewardUpdated)
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
func (it *InboundQueueRewardUpdatedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *InboundQueueRewardUpdatedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// InboundQueueRewardUpdated represents a RewardUpdated event raised by the InboundQueue contract.
type InboundQueueRewardUpdated struct {
	Reward *big.Int
	Raw    types.Log // Blockchain specific contextual infos
}

// FilterRewardUpdated is a free log retrieval operation binding the contract event 0xcb94909754d27c309adf4167150f1f7aa04de40b6a0e6bb98b2ae80a2bf438f6.
//
// Solidity: event RewardUpdated(uint256 reward)
func (_InboundQueue *InboundQueueFilterer) FilterRewardUpdated(opts *bind.FilterOpts) (*InboundQueueRewardUpdatedIterator, error) {

	logs, sub, err := _InboundQueue.contract.FilterLogs(opts, "RewardUpdated")
	if err != nil {
		return nil, err
	}
	return &InboundQueueRewardUpdatedIterator{contract: _InboundQueue.contract, event: "RewardUpdated", logs: logs, sub: sub}, nil
}

// WatchRewardUpdated is a free log subscription operation binding the contract event 0xcb94909754d27c309adf4167150f1f7aa04de40b6a0e6bb98b2ae80a2bf438f6.
//
// Solidity: event RewardUpdated(uint256 reward)
func (_InboundQueue *InboundQueueFilterer) WatchRewardUpdated(opts *bind.WatchOpts, sink chan<- *InboundQueueRewardUpdated) (event.Subscription, error) {

	logs, sub, err := _InboundQueue.contract.WatchLogs(opts, "RewardUpdated")
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(InboundQueueRewardUpdated)
				if err := _InboundQueue.contract.UnpackLog(event, "RewardUpdated", log); err != nil {
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
func (_InboundQueue *InboundQueueFilterer) ParseRewardUpdated(log types.Log) (*InboundQueueRewardUpdated, error) {
	event := new(InboundQueueRewardUpdated)
	if err := _InboundQueue.contract.UnpackLog(event, "RewardUpdated", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// InboundQueueRoleAdminChangedIterator is returned from FilterRoleAdminChanged and is used to iterate over the raw logs and unpacked data for RoleAdminChanged events raised by the InboundQueue contract.
type InboundQueueRoleAdminChangedIterator struct {
	Event *InboundQueueRoleAdminChanged // Event containing the contract specifics and raw log

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
func (it *InboundQueueRoleAdminChangedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(InboundQueueRoleAdminChanged)
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
		it.Event = new(InboundQueueRoleAdminChanged)
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
func (it *InboundQueueRoleAdminChangedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *InboundQueueRoleAdminChangedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// InboundQueueRoleAdminChanged represents a RoleAdminChanged event raised by the InboundQueue contract.
type InboundQueueRoleAdminChanged struct {
	Role              [32]byte
	PreviousAdminRole [32]byte
	NewAdminRole      [32]byte
	Raw               types.Log // Blockchain specific contextual infos
}

// FilterRoleAdminChanged is a free log retrieval operation binding the contract event 0xbd79b86ffe0ab8e8776151514217cd7cacd52c909f66475c3af44e129f0b00ff.
//
// Solidity: event RoleAdminChanged(bytes32 indexed role, bytes32 indexed previousAdminRole, bytes32 indexed newAdminRole)
func (_InboundQueue *InboundQueueFilterer) FilterRoleAdminChanged(opts *bind.FilterOpts, role [][32]byte, previousAdminRole [][32]byte, newAdminRole [][32]byte) (*InboundQueueRoleAdminChangedIterator, error) {

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

	logs, sub, err := _InboundQueue.contract.FilterLogs(opts, "RoleAdminChanged", roleRule, previousAdminRoleRule, newAdminRoleRule)
	if err != nil {
		return nil, err
	}
	return &InboundQueueRoleAdminChangedIterator{contract: _InboundQueue.contract, event: "RoleAdminChanged", logs: logs, sub: sub}, nil
}

// WatchRoleAdminChanged is a free log subscription operation binding the contract event 0xbd79b86ffe0ab8e8776151514217cd7cacd52c909f66475c3af44e129f0b00ff.
//
// Solidity: event RoleAdminChanged(bytes32 indexed role, bytes32 indexed previousAdminRole, bytes32 indexed newAdminRole)
func (_InboundQueue *InboundQueueFilterer) WatchRoleAdminChanged(opts *bind.WatchOpts, sink chan<- *InboundQueueRoleAdminChanged, role [][32]byte, previousAdminRole [][32]byte, newAdminRole [][32]byte) (event.Subscription, error) {

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

	logs, sub, err := _InboundQueue.contract.WatchLogs(opts, "RoleAdminChanged", roleRule, previousAdminRoleRule, newAdminRoleRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(InboundQueueRoleAdminChanged)
				if err := _InboundQueue.contract.UnpackLog(event, "RoleAdminChanged", log); err != nil {
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
func (_InboundQueue *InboundQueueFilterer) ParseRoleAdminChanged(log types.Log) (*InboundQueueRoleAdminChanged, error) {
	event := new(InboundQueueRoleAdminChanged)
	if err := _InboundQueue.contract.UnpackLog(event, "RoleAdminChanged", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// InboundQueueRoleGrantedIterator is returned from FilterRoleGranted and is used to iterate over the raw logs and unpacked data for RoleGranted events raised by the InboundQueue contract.
type InboundQueueRoleGrantedIterator struct {
	Event *InboundQueueRoleGranted // Event containing the contract specifics and raw log

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
func (it *InboundQueueRoleGrantedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(InboundQueueRoleGranted)
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
		it.Event = new(InboundQueueRoleGranted)
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
func (it *InboundQueueRoleGrantedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *InboundQueueRoleGrantedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// InboundQueueRoleGranted represents a RoleGranted event raised by the InboundQueue contract.
type InboundQueueRoleGranted struct {
	Role    [32]byte
	Account common.Address
	Sender  common.Address
	Raw     types.Log // Blockchain specific contextual infos
}

// FilterRoleGranted is a free log retrieval operation binding the contract event 0x2f8788117e7eff1d82e926ec794901d17c78024a50270940304540a733656f0d.
//
// Solidity: event RoleGranted(bytes32 indexed role, address indexed account, address indexed sender)
func (_InboundQueue *InboundQueueFilterer) FilterRoleGranted(opts *bind.FilterOpts, role [][32]byte, account []common.Address, sender []common.Address) (*InboundQueueRoleGrantedIterator, error) {

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

	logs, sub, err := _InboundQueue.contract.FilterLogs(opts, "RoleGranted", roleRule, accountRule, senderRule)
	if err != nil {
		return nil, err
	}
	return &InboundQueueRoleGrantedIterator{contract: _InboundQueue.contract, event: "RoleGranted", logs: logs, sub: sub}, nil
}

// WatchRoleGranted is a free log subscription operation binding the contract event 0x2f8788117e7eff1d82e926ec794901d17c78024a50270940304540a733656f0d.
//
// Solidity: event RoleGranted(bytes32 indexed role, address indexed account, address indexed sender)
func (_InboundQueue *InboundQueueFilterer) WatchRoleGranted(opts *bind.WatchOpts, sink chan<- *InboundQueueRoleGranted, role [][32]byte, account []common.Address, sender []common.Address) (event.Subscription, error) {

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

	logs, sub, err := _InboundQueue.contract.WatchLogs(opts, "RoleGranted", roleRule, accountRule, senderRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(InboundQueueRoleGranted)
				if err := _InboundQueue.contract.UnpackLog(event, "RoleGranted", log); err != nil {
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
func (_InboundQueue *InboundQueueFilterer) ParseRoleGranted(log types.Log) (*InboundQueueRoleGranted, error) {
	event := new(InboundQueueRoleGranted)
	if err := _InboundQueue.contract.UnpackLog(event, "RoleGranted", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// InboundQueueRoleRevokedIterator is returned from FilterRoleRevoked and is used to iterate over the raw logs and unpacked data for RoleRevoked events raised by the InboundQueue contract.
type InboundQueueRoleRevokedIterator struct {
	Event *InboundQueueRoleRevoked // Event containing the contract specifics and raw log

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
func (it *InboundQueueRoleRevokedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(InboundQueueRoleRevoked)
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
		it.Event = new(InboundQueueRoleRevoked)
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
func (it *InboundQueueRoleRevokedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *InboundQueueRoleRevokedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// InboundQueueRoleRevoked represents a RoleRevoked event raised by the InboundQueue contract.
type InboundQueueRoleRevoked struct {
	Role    [32]byte
	Account common.Address
	Sender  common.Address
	Raw     types.Log // Blockchain specific contextual infos
}

// FilterRoleRevoked is a free log retrieval operation binding the contract event 0xf6391f5c32d9c69d2a47ea670b442974b53935d1edc7fd64eb21e047a839171b.
//
// Solidity: event RoleRevoked(bytes32 indexed role, address indexed account, address indexed sender)
func (_InboundQueue *InboundQueueFilterer) FilterRoleRevoked(opts *bind.FilterOpts, role [][32]byte, account []common.Address, sender []common.Address) (*InboundQueueRoleRevokedIterator, error) {

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

	logs, sub, err := _InboundQueue.contract.FilterLogs(opts, "RoleRevoked", roleRule, accountRule, senderRule)
	if err != nil {
		return nil, err
	}
	return &InboundQueueRoleRevokedIterator{contract: _InboundQueue.contract, event: "RoleRevoked", logs: logs, sub: sub}, nil
}

// WatchRoleRevoked is a free log subscription operation binding the contract event 0xf6391f5c32d9c69d2a47ea670b442974b53935d1edc7fd64eb21e047a839171b.
//
// Solidity: event RoleRevoked(bytes32 indexed role, address indexed account, address indexed sender)
func (_InboundQueue *InboundQueueFilterer) WatchRoleRevoked(opts *bind.WatchOpts, sink chan<- *InboundQueueRoleRevoked, role [][32]byte, account []common.Address, sender []common.Address) (event.Subscription, error) {

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

	logs, sub, err := _InboundQueue.contract.WatchLogs(opts, "RoleRevoked", roleRule, accountRule, senderRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(InboundQueueRoleRevoked)
				if err := _InboundQueue.contract.UnpackLog(event, "RoleRevoked", log); err != nil {
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
func (_InboundQueue *InboundQueueFilterer) ParseRoleRevoked(log types.Log) (*InboundQueueRoleRevoked, error) {
	event := new(InboundQueueRoleRevoked)
	if err := _InboundQueue.contract.UnpackLog(event, "RoleRevoked", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// InboundQueueVaultUpdatedIterator is returned from FilterVaultUpdated and is used to iterate over the raw logs and unpacked data for VaultUpdated events raised by the InboundQueue contract.
type InboundQueueVaultUpdatedIterator struct {
	Event *InboundQueueVaultUpdated // Event containing the contract specifics and raw log

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
func (it *InboundQueueVaultUpdatedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(InboundQueueVaultUpdated)
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
		it.Event = new(InboundQueueVaultUpdated)
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
func (it *InboundQueueVaultUpdatedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *InboundQueueVaultUpdatedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// InboundQueueVaultUpdated represents a VaultUpdated event raised by the InboundQueue contract.
type InboundQueueVaultUpdated struct {
	Vault common.Address
	Raw   types.Log // Blockchain specific contextual infos
}

// FilterVaultUpdated is a free log retrieval operation binding the contract event 0x161584aed96e7f34998117c9ad67e2d21ff46d2a42775c22b11ed282f3c7b2cd.
//
// Solidity: event VaultUpdated(address vault)
func (_InboundQueue *InboundQueueFilterer) FilterVaultUpdated(opts *bind.FilterOpts) (*InboundQueueVaultUpdatedIterator, error) {

	logs, sub, err := _InboundQueue.contract.FilterLogs(opts, "VaultUpdated")
	if err != nil {
		return nil, err
	}
	return &InboundQueueVaultUpdatedIterator{contract: _InboundQueue.contract, event: "VaultUpdated", logs: logs, sub: sub}, nil
}

// WatchVaultUpdated is a free log subscription operation binding the contract event 0x161584aed96e7f34998117c9ad67e2d21ff46d2a42775c22b11ed282f3c7b2cd.
//
// Solidity: event VaultUpdated(address vault)
func (_InboundQueue *InboundQueueFilterer) WatchVaultUpdated(opts *bind.WatchOpts, sink chan<- *InboundQueueVaultUpdated) (event.Subscription, error) {

	logs, sub, err := _InboundQueue.contract.WatchLogs(opts, "VaultUpdated")
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(InboundQueueVaultUpdated)
				if err := _InboundQueue.contract.UnpackLog(event, "VaultUpdated", log); err != nil {
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
func (_InboundQueue *InboundQueueFilterer) ParseVaultUpdated(log types.Log) (*InboundQueueVaultUpdated, error) {
	event := new(InboundQueueVaultUpdated)
	if err := _InboundQueue.contract.UnpackLog(event, "VaultUpdated", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}
