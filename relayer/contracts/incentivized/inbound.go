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

// IncentivizedInboundChannelMessage is an auto generated low-level Go binding around an user-defined struct.
type IncentivizedInboundChannelMessage struct {
	Target  common.Address
	Nonce   uint64
	Fee     *big.Int
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

// IncentivizedInboundChannelABI is the input ABI used to generate the binding from.
const IncentivizedInboundChannelABI = "[{\"inputs\":[{\"internalType\":\"contractBeefyLightClient\",\"name\":\"_beefyLightClient\",\"type\":\"address\"}],\"stateMutability\":\"nonpayable\",\"type\":\"constructor\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":false,\"internalType\":\"uint64\",\"name\":\"nonce\",\"type\":\"uint64\"},{\"indexed\":false,\"internalType\":\"bool\",\"name\":\"result\",\"type\":\"bool\"}],\"name\":\"MessageDispatched\",\"type\":\"event\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":true,\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\"},{\"indexed\":true,\"internalType\":\"bytes32\",\"name\":\"previousAdminRole\",\"type\":\"bytes32\"},{\"indexed\":true,\"internalType\":\"bytes32\",\"name\":\"newAdminRole\",\"type\":\"bytes32\"}],\"name\":\"RoleAdminChanged\",\"type\":\"event\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":true,\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\"},{\"indexed\":true,\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\"},{\"indexed\":true,\"internalType\":\"address\",\"name\":\"sender\",\"type\":\"address\"}],\"name\":\"RoleGranted\",\"type\":\"event\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":true,\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\"},{\"indexed\":true,\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\"},{\"indexed\":true,\"internalType\":\"address\",\"name\":\"sender\",\"type\":\"address\"}],\"name\":\"RoleRevoked\",\"type\":\"event\"},{\"inputs\":[],\"name\":\"CONFIG_UPDATE_ROLE\",\"outputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"DEFAULT_ADMIN_ROLE\",\"outputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"GAS_BUFFER\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"MAX_GAS_PER_MESSAGE\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"beefyLightClient\",\"outputs\":[{\"internalType\":\"contractBeefyLightClient\",\"name\":\"\",\"type\":\"address\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\"}],\"name\":\"getRoleAdmin\",\"outputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\"},{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\"}],\"name\":\"grantRole\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\"},{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\"}],\"name\":\"hasRole\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"_configUpdater\",\"type\":\"address\"},{\"internalType\":\"address\",\"name\":\"_rewardSource\",\"type\":\"address\"}],\"name\":\"initialize\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"nonce\",\"outputs\":[{\"internalType\":\"uint64\",\"name\":\"\",\"type\":\"uint64\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\"},{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\"}],\"name\":\"renounceRole\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\"},{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\"}],\"name\":\"revokeRole\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[{\"components\":[{\"internalType\":\"address\",\"name\":\"target\",\"type\":\"address\"},{\"internalType\":\"uint64\",\"name\":\"nonce\",\"type\":\"uint64\"},{\"internalType\":\"uint256\",\"name\":\"fee\",\"type\":\"uint256\"},{\"internalType\":\"bytes\",\"name\":\"payload\",\"type\":\"bytes\"}],\"internalType\":\"structIncentivizedInboundChannel.Message[]\",\"name\":\"_messages\",\"type\":\"tuple[]\"},{\"components\":[{\"internalType\":\"bytes\",\"name\":\"ownParachainHeadPrefixBytes\",\"type\":\"bytes\"},{\"internalType\":\"bytes\",\"name\":\"ownParachainHeadSuffixBytes\",\"type\":\"bytes\"},{\"components\":[{\"internalType\":\"uint256\",\"name\":\"pos\",\"type\":\"uint256\"},{\"internalType\":\"uint256\",\"name\":\"width\",\"type\":\"uint256\"},{\"internalType\":\"bytes32[]\",\"name\":\"proof\",\"type\":\"bytes32[]\"}],\"internalType\":\"structParachainLightClient.ParachainHeadProof\",\"name\":\"parachainHeadProof\",\"type\":\"tuple\"}],\"internalType\":\"structParachainLightClient.ParachainVerifyInput\",\"name\":\"_parachainVerifyInput\",\"type\":\"tuple\"},{\"components\":[{\"internalType\":\"uint8\",\"name\":\"version\",\"type\":\"uint8\"},{\"internalType\":\"uint32\",\"name\":\"parentNumber\",\"type\":\"uint32\"},{\"internalType\":\"bytes32\",\"name\":\"parentHash\",\"type\":\"bytes32\"},{\"internalType\":\"uint64\",\"name\":\"nextAuthoritySetId\",\"type\":\"uint64\"},{\"internalType\":\"uint32\",\"name\":\"nextAuthoritySetLen\",\"type\":\"uint32\"},{\"internalType\":\"bytes32\",\"name\":\"nextAuthoritySetRoot\",\"type\":\"bytes32\"}],\"internalType\":\"structParachainLightClient.BeefyMMRLeafPartial\",\"name\":\"_beefyMMRLeafPartial\",\"type\":\"tuple\"},{\"components\":[{\"internalType\":\"bytes32[]\",\"name\":\"merkleProofItems\",\"type\":\"bytes32[]\"},{\"internalType\":\"uint64\",\"name\":\"merkleProofOrderBitField\",\"type\":\"uint64\"}],\"internalType\":\"structSimplifiedMMRProof\",\"name\":\"proof\",\"type\":\"tuple\"}],\"name\":\"submit\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"bytes4\",\"name\":\"interfaceId\",\"type\":\"bytes4\"}],\"name\":\"supportsInterface\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\"}],\"stateMutability\":\"view\",\"type\":\"function\"}]"

// IncentivizedInboundChannel is an auto generated Go binding around an Ethereum contract.
type IncentivizedInboundChannel struct {
	IncentivizedInboundChannelCaller     // Read-only binding to the contract
	IncentivizedInboundChannelTransactor // Write-only binding to the contract
	IncentivizedInboundChannelFilterer   // Log filterer for contract events
}

// IncentivizedInboundChannelCaller is an auto generated read-only Go binding around an Ethereum contract.
type IncentivizedInboundChannelCaller struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// IncentivizedInboundChannelTransactor is an auto generated write-only Go binding around an Ethereum contract.
type IncentivizedInboundChannelTransactor struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// IncentivizedInboundChannelFilterer is an auto generated log filtering Go binding around an Ethereum contract events.
type IncentivizedInboundChannelFilterer struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// IncentivizedInboundChannelSession is an auto generated Go binding around an Ethereum contract,
// with pre-set call and transact options.
type IncentivizedInboundChannelSession struct {
	Contract     *IncentivizedInboundChannel // Generic contract binding to set the session for
	CallOpts     bind.CallOpts               // Call options to use throughout this session
	TransactOpts bind.TransactOpts           // Transaction auth options to use throughout this session
}

// IncentivizedInboundChannelCallerSession is an auto generated read-only Go binding around an Ethereum contract,
// with pre-set call options.
type IncentivizedInboundChannelCallerSession struct {
	Contract *IncentivizedInboundChannelCaller // Generic contract caller binding to set the session for
	CallOpts bind.CallOpts                     // Call options to use throughout this session
}

// IncentivizedInboundChannelTransactorSession is an auto generated write-only Go binding around an Ethereum contract,
// with pre-set transact options.
type IncentivizedInboundChannelTransactorSession struct {
	Contract     *IncentivizedInboundChannelTransactor // Generic contract transactor binding to set the session for
	TransactOpts bind.TransactOpts                     // Transaction auth options to use throughout this session
}

// IncentivizedInboundChannelRaw is an auto generated low-level Go binding around an Ethereum contract.
type IncentivizedInboundChannelRaw struct {
	Contract *IncentivizedInboundChannel // Generic contract binding to access the raw methods on
}

// IncentivizedInboundChannelCallerRaw is an auto generated low-level read-only Go binding around an Ethereum contract.
type IncentivizedInboundChannelCallerRaw struct {
	Contract *IncentivizedInboundChannelCaller // Generic read-only contract binding to access the raw methods on
}

// IncentivizedInboundChannelTransactorRaw is an auto generated low-level write-only Go binding around an Ethereum contract.
type IncentivizedInboundChannelTransactorRaw struct {
	Contract *IncentivizedInboundChannelTransactor // Generic write-only contract binding to access the raw methods on
}

// NewIncentivizedInboundChannel creates a new instance of IncentivizedInboundChannel, bound to a specific deployed contract.
func NewIncentivizedInboundChannel(address common.Address, backend bind.ContractBackend) (*IncentivizedInboundChannel, error) {
	contract, err := bindIncentivizedInboundChannel(address, backend, backend, backend)
	if err != nil {
		return nil, err
	}
	return &IncentivizedInboundChannel{IncentivizedInboundChannelCaller: IncentivizedInboundChannelCaller{contract: contract}, IncentivizedInboundChannelTransactor: IncentivizedInboundChannelTransactor{contract: contract}, IncentivizedInboundChannelFilterer: IncentivizedInboundChannelFilterer{contract: contract}}, nil
}

// NewIncentivizedInboundChannelCaller creates a new read-only instance of IncentivizedInboundChannel, bound to a specific deployed contract.
func NewIncentivizedInboundChannelCaller(address common.Address, caller bind.ContractCaller) (*IncentivizedInboundChannelCaller, error) {
	contract, err := bindIncentivizedInboundChannel(address, caller, nil, nil)
	if err != nil {
		return nil, err
	}
	return &IncentivizedInboundChannelCaller{contract: contract}, nil
}

// NewIncentivizedInboundChannelTransactor creates a new write-only instance of IncentivizedInboundChannel, bound to a specific deployed contract.
func NewIncentivizedInboundChannelTransactor(address common.Address, transactor bind.ContractTransactor) (*IncentivizedInboundChannelTransactor, error) {
	contract, err := bindIncentivizedInboundChannel(address, nil, transactor, nil)
	if err != nil {
		return nil, err
	}
	return &IncentivizedInboundChannelTransactor{contract: contract}, nil
}

// NewIncentivizedInboundChannelFilterer creates a new log filterer instance of IncentivizedInboundChannel, bound to a specific deployed contract.
func NewIncentivizedInboundChannelFilterer(address common.Address, filterer bind.ContractFilterer) (*IncentivizedInboundChannelFilterer, error) {
	contract, err := bindIncentivizedInboundChannel(address, nil, nil, filterer)
	if err != nil {
		return nil, err
	}
	return &IncentivizedInboundChannelFilterer{contract: contract}, nil
}

// bindIncentivizedInboundChannel binds a generic wrapper to an already deployed contract.
func bindIncentivizedInboundChannel(address common.Address, caller bind.ContractCaller, transactor bind.ContractTransactor, filterer bind.ContractFilterer) (*bind.BoundContract, error) {
	parsed, err := abi.JSON(strings.NewReader(IncentivizedInboundChannelABI))
	if err != nil {
		return nil, err
	}
	return bind.NewBoundContract(address, parsed, caller, transactor, filterer), nil
}

// Call invokes the (constant) contract method with params as input values and
// sets the output to result. The result type might be a single field for simple
// returns, a slice of interfaces for anonymous returns and a struct for named
// returns.
func (_IncentivizedInboundChannel *IncentivizedInboundChannelRaw) Call(opts *bind.CallOpts, result *[]interface{}, method string, params ...interface{}) error {
	return _IncentivizedInboundChannel.Contract.IncentivizedInboundChannelCaller.contract.Call(opts, result, method, params...)
}

// Transfer initiates a plain transaction to move funds to the contract, calling
// its default method if one is available.
func (_IncentivizedInboundChannel *IncentivizedInboundChannelRaw) Transfer(opts *bind.TransactOpts) (*types.Transaction, error) {
	return _IncentivizedInboundChannel.Contract.IncentivizedInboundChannelTransactor.contract.Transfer(opts)
}

// Transact invokes the (paid) contract method with params as input values.
func (_IncentivizedInboundChannel *IncentivizedInboundChannelRaw) Transact(opts *bind.TransactOpts, method string, params ...interface{}) (*types.Transaction, error) {
	return _IncentivizedInboundChannel.Contract.IncentivizedInboundChannelTransactor.contract.Transact(opts, method, params...)
}

// Call invokes the (constant) contract method with params as input values and
// sets the output to result. The result type might be a single field for simple
// returns, a slice of interfaces for anonymous returns and a struct for named
// returns.
func (_IncentivizedInboundChannel *IncentivizedInboundChannelCallerRaw) Call(opts *bind.CallOpts, result *[]interface{}, method string, params ...interface{}) error {
	return _IncentivizedInboundChannel.Contract.contract.Call(opts, result, method, params...)
}

// Transfer initiates a plain transaction to move funds to the contract, calling
// its default method if one is available.
func (_IncentivizedInboundChannel *IncentivizedInboundChannelTransactorRaw) Transfer(opts *bind.TransactOpts) (*types.Transaction, error) {
	return _IncentivizedInboundChannel.Contract.contract.Transfer(opts)
}

// Transact invokes the (paid) contract method with params as input values.
func (_IncentivizedInboundChannel *IncentivizedInboundChannelTransactorRaw) Transact(opts *bind.TransactOpts, method string, params ...interface{}) (*types.Transaction, error) {
	return _IncentivizedInboundChannel.Contract.contract.Transact(opts, method, params...)
}

// CONFIGUPDATEROLE is a free data retrieval call binding the contract method 0xa2d6c6e5.
//
// Solidity: function CONFIG_UPDATE_ROLE() view returns(bytes32)
func (_IncentivizedInboundChannel *IncentivizedInboundChannelCaller) CONFIGUPDATEROLE(opts *bind.CallOpts) ([32]byte, error) {
	var out []interface{}
	err := _IncentivizedInboundChannel.contract.Call(opts, &out, "CONFIG_UPDATE_ROLE")

	if err != nil {
		return *new([32]byte), err
	}

	out0 := *abi.ConvertType(out[0], new([32]byte)).(*[32]byte)

	return out0, err

}

// CONFIGUPDATEROLE is a free data retrieval call binding the contract method 0xa2d6c6e5.
//
// Solidity: function CONFIG_UPDATE_ROLE() view returns(bytes32)
func (_IncentivizedInboundChannel *IncentivizedInboundChannelSession) CONFIGUPDATEROLE() ([32]byte, error) {
	return _IncentivizedInboundChannel.Contract.CONFIGUPDATEROLE(&_IncentivizedInboundChannel.CallOpts)
}

// CONFIGUPDATEROLE is a free data retrieval call binding the contract method 0xa2d6c6e5.
//
// Solidity: function CONFIG_UPDATE_ROLE() view returns(bytes32)
func (_IncentivizedInboundChannel *IncentivizedInboundChannelCallerSession) CONFIGUPDATEROLE() ([32]byte, error) {
	return _IncentivizedInboundChannel.Contract.CONFIGUPDATEROLE(&_IncentivizedInboundChannel.CallOpts)
}

// DEFAULTADMINROLE is a free data retrieval call binding the contract method 0xa217fddf.
//
// Solidity: function DEFAULT_ADMIN_ROLE() view returns(bytes32)
func (_IncentivizedInboundChannel *IncentivizedInboundChannelCaller) DEFAULTADMINROLE(opts *bind.CallOpts) ([32]byte, error) {
	var out []interface{}
	err := _IncentivizedInboundChannel.contract.Call(opts, &out, "DEFAULT_ADMIN_ROLE")

	if err != nil {
		return *new([32]byte), err
	}

	out0 := *abi.ConvertType(out[0], new([32]byte)).(*[32]byte)

	return out0, err

}

// DEFAULTADMINROLE is a free data retrieval call binding the contract method 0xa217fddf.
//
// Solidity: function DEFAULT_ADMIN_ROLE() view returns(bytes32)
func (_IncentivizedInboundChannel *IncentivizedInboundChannelSession) DEFAULTADMINROLE() ([32]byte, error) {
	return _IncentivizedInboundChannel.Contract.DEFAULTADMINROLE(&_IncentivizedInboundChannel.CallOpts)
}

// DEFAULTADMINROLE is a free data retrieval call binding the contract method 0xa217fddf.
//
// Solidity: function DEFAULT_ADMIN_ROLE() view returns(bytes32)
func (_IncentivizedInboundChannel *IncentivizedInboundChannelCallerSession) DEFAULTADMINROLE() ([32]byte, error) {
	return _IncentivizedInboundChannel.Contract.DEFAULTADMINROLE(&_IncentivizedInboundChannel.CallOpts)
}

// GASBUFFER is a free data retrieval call binding the contract method 0xc7e67360.
//
// Solidity: function GAS_BUFFER() view returns(uint256)
func (_IncentivizedInboundChannel *IncentivizedInboundChannelCaller) GASBUFFER(opts *bind.CallOpts) (*big.Int, error) {
	var out []interface{}
	err := _IncentivizedInboundChannel.contract.Call(opts, &out, "GAS_BUFFER")

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// GASBUFFER is a free data retrieval call binding the contract method 0xc7e67360.
//
// Solidity: function GAS_BUFFER() view returns(uint256)
func (_IncentivizedInboundChannel *IncentivizedInboundChannelSession) GASBUFFER() (*big.Int, error) {
	return _IncentivizedInboundChannel.Contract.GASBUFFER(&_IncentivizedInboundChannel.CallOpts)
}

// GASBUFFER is a free data retrieval call binding the contract method 0xc7e67360.
//
// Solidity: function GAS_BUFFER() view returns(uint256)
func (_IncentivizedInboundChannel *IncentivizedInboundChannelCallerSession) GASBUFFER() (*big.Int, error) {
	return _IncentivizedInboundChannel.Contract.GASBUFFER(&_IncentivizedInboundChannel.CallOpts)
}

// MAXGASPERMESSAGE is a free data retrieval call binding the contract method 0x49bee574.
//
// Solidity: function MAX_GAS_PER_MESSAGE() view returns(uint256)
func (_IncentivizedInboundChannel *IncentivizedInboundChannelCaller) MAXGASPERMESSAGE(opts *bind.CallOpts) (*big.Int, error) {
	var out []interface{}
	err := _IncentivizedInboundChannel.contract.Call(opts, &out, "MAX_GAS_PER_MESSAGE")

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// MAXGASPERMESSAGE is a free data retrieval call binding the contract method 0x49bee574.
//
// Solidity: function MAX_GAS_PER_MESSAGE() view returns(uint256)
func (_IncentivizedInboundChannel *IncentivizedInboundChannelSession) MAXGASPERMESSAGE() (*big.Int, error) {
	return _IncentivizedInboundChannel.Contract.MAXGASPERMESSAGE(&_IncentivizedInboundChannel.CallOpts)
}

// MAXGASPERMESSAGE is a free data retrieval call binding the contract method 0x49bee574.
//
// Solidity: function MAX_GAS_PER_MESSAGE() view returns(uint256)
func (_IncentivizedInboundChannel *IncentivizedInboundChannelCallerSession) MAXGASPERMESSAGE() (*big.Int, error) {
	return _IncentivizedInboundChannel.Contract.MAXGASPERMESSAGE(&_IncentivizedInboundChannel.CallOpts)
}

// BeefyLightClient is a free data retrieval call binding the contract method 0xaf41c33e.
//
// Solidity: function beefyLightClient() view returns(address)
func (_IncentivizedInboundChannel *IncentivizedInboundChannelCaller) BeefyLightClient(opts *bind.CallOpts) (common.Address, error) {
	var out []interface{}
	err := _IncentivizedInboundChannel.contract.Call(opts, &out, "beefyLightClient")

	if err != nil {
		return *new(common.Address), err
	}

	out0 := *abi.ConvertType(out[0], new(common.Address)).(*common.Address)

	return out0, err

}

// BeefyLightClient is a free data retrieval call binding the contract method 0xaf41c33e.
//
// Solidity: function beefyLightClient() view returns(address)
func (_IncentivizedInboundChannel *IncentivizedInboundChannelSession) BeefyLightClient() (common.Address, error) {
	return _IncentivizedInboundChannel.Contract.BeefyLightClient(&_IncentivizedInboundChannel.CallOpts)
}

// BeefyLightClient is a free data retrieval call binding the contract method 0xaf41c33e.
//
// Solidity: function beefyLightClient() view returns(address)
func (_IncentivizedInboundChannel *IncentivizedInboundChannelCallerSession) BeefyLightClient() (common.Address, error) {
	return _IncentivizedInboundChannel.Contract.BeefyLightClient(&_IncentivizedInboundChannel.CallOpts)
}

// GetRoleAdmin is a free data retrieval call binding the contract method 0x248a9ca3.
//
// Solidity: function getRoleAdmin(bytes32 role) view returns(bytes32)
func (_IncentivizedInboundChannel *IncentivizedInboundChannelCaller) GetRoleAdmin(opts *bind.CallOpts, role [32]byte) ([32]byte, error) {
	var out []interface{}
	err := _IncentivizedInboundChannel.contract.Call(opts, &out, "getRoleAdmin", role)

	if err != nil {
		return *new([32]byte), err
	}

	out0 := *abi.ConvertType(out[0], new([32]byte)).(*[32]byte)

	return out0, err

}

// GetRoleAdmin is a free data retrieval call binding the contract method 0x248a9ca3.
//
// Solidity: function getRoleAdmin(bytes32 role) view returns(bytes32)
func (_IncentivizedInboundChannel *IncentivizedInboundChannelSession) GetRoleAdmin(role [32]byte) ([32]byte, error) {
	return _IncentivizedInboundChannel.Contract.GetRoleAdmin(&_IncentivizedInboundChannel.CallOpts, role)
}

// GetRoleAdmin is a free data retrieval call binding the contract method 0x248a9ca3.
//
// Solidity: function getRoleAdmin(bytes32 role) view returns(bytes32)
func (_IncentivizedInboundChannel *IncentivizedInboundChannelCallerSession) GetRoleAdmin(role [32]byte) ([32]byte, error) {
	return _IncentivizedInboundChannel.Contract.GetRoleAdmin(&_IncentivizedInboundChannel.CallOpts, role)
}

// HasRole is a free data retrieval call binding the contract method 0x91d14854.
//
// Solidity: function hasRole(bytes32 role, address account) view returns(bool)
func (_IncentivizedInboundChannel *IncentivizedInboundChannelCaller) HasRole(opts *bind.CallOpts, role [32]byte, account common.Address) (bool, error) {
	var out []interface{}
	err := _IncentivizedInboundChannel.contract.Call(opts, &out, "hasRole", role, account)

	if err != nil {
		return *new(bool), err
	}

	out0 := *abi.ConvertType(out[0], new(bool)).(*bool)

	return out0, err

}

// HasRole is a free data retrieval call binding the contract method 0x91d14854.
//
// Solidity: function hasRole(bytes32 role, address account) view returns(bool)
func (_IncentivizedInboundChannel *IncentivizedInboundChannelSession) HasRole(role [32]byte, account common.Address) (bool, error) {
	return _IncentivizedInboundChannel.Contract.HasRole(&_IncentivizedInboundChannel.CallOpts, role, account)
}

// HasRole is a free data retrieval call binding the contract method 0x91d14854.
//
// Solidity: function hasRole(bytes32 role, address account) view returns(bool)
func (_IncentivizedInboundChannel *IncentivizedInboundChannelCallerSession) HasRole(role [32]byte, account common.Address) (bool, error) {
	return _IncentivizedInboundChannel.Contract.HasRole(&_IncentivizedInboundChannel.CallOpts, role, account)
}

// Nonce is a free data retrieval call binding the contract method 0xaffed0e0.
//
// Solidity: function nonce() view returns(uint64)
func (_IncentivizedInboundChannel *IncentivizedInboundChannelCaller) Nonce(opts *bind.CallOpts) (uint64, error) {
	var out []interface{}
	err := _IncentivizedInboundChannel.contract.Call(opts, &out, "nonce")

	if err != nil {
		return *new(uint64), err
	}

	out0 := *abi.ConvertType(out[0], new(uint64)).(*uint64)

	return out0, err

}

// Nonce is a free data retrieval call binding the contract method 0xaffed0e0.
//
// Solidity: function nonce() view returns(uint64)
func (_IncentivizedInboundChannel *IncentivizedInboundChannelSession) Nonce() (uint64, error) {
	return _IncentivizedInboundChannel.Contract.Nonce(&_IncentivizedInboundChannel.CallOpts)
}

// Nonce is a free data retrieval call binding the contract method 0xaffed0e0.
//
// Solidity: function nonce() view returns(uint64)
func (_IncentivizedInboundChannel *IncentivizedInboundChannelCallerSession) Nonce() (uint64, error) {
	return _IncentivizedInboundChannel.Contract.Nonce(&_IncentivizedInboundChannel.CallOpts)
}

// SupportsInterface is a free data retrieval call binding the contract method 0x01ffc9a7.
//
// Solidity: function supportsInterface(bytes4 interfaceId) view returns(bool)
func (_IncentivizedInboundChannel *IncentivizedInboundChannelCaller) SupportsInterface(opts *bind.CallOpts, interfaceId [4]byte) (bool, error) {
	var out []interface{}
	err := _IncentivizedInboundChannel.contract.Call(opts, &out, "supportsInterface", interfaceId)

	if err != nil {
		return *new(bool), err
	}

	out0 := *abi.ConvertType(out[0], new(bool)).(*bool)

	return out0, err

}

// SupportsInterface is a free data retrieval call binding the contract method 0x01ffc9a7.
//
// Solidity: function supportsInterface(bytes4 interfaceId) view returns(bool)
func (_IncentivizedInboundChannel *IncentivizedInboundChannelSession) SupportsInterface(interfaceId [4]byte) (bool, error) {
	return _IncentivizedInboundChannel.Contract.SupportsInterface(&_IncentivizedInboundChannel.CallOpts, interfaceId)
}

// SupportsInterface is a free data retrieval call binding the contract method 0x01ffc9a7.
//
// Solidity: function supportsInterface(bytes4 interfaceId) view returns(bool)
func (_IncentivizedInboundChannel *IncentivizedInboundChannelCallerSession) SupportsInterface(interfaceId [4]byte) (bool, error) {
	return _IncentivizedInboundChannel.Contract.SupportsInterface(&_IncentivizedInboundChannel.CallOpts, interfaceId)
}

// GrantRole is a paid mutator transaction binding the contract method 0x2f2ff15d.
//
// Solidity: function grantRole(bytes32 role, address account) returns()
func (_IncentivizedInboundChannel *IncentivizedInboundChannelTransactor) GrantRole(opts *bind.TransactOpts, role [32]byte, account common.Address) (*types.Transaction, error) {
	return _IncentivizedInboundChannel.contract.Transact(opts, "grantRole", role, account)
}

// GrantRole is a paid mutator transaction binding the contract method 0x2f2ff15d.
//
// Solidity: function grantRole(bytes32 role, address account) returns()
func (_IncentivizedInboundChannel *IncentivizedInboundChannelSession) GrantRole(role [32]byte, account common.Address) (*types.Transaction, error) {
	return _IncentivizedInboundChannel.Contract.GrantRole(&_IncentivizedInboundChannel.TransactOpts, role, account)
}

// GrantRole is a paid mutator transaction binding the contract method 0x2f2ff15d.
//
// Solidity: function grantRole(bytes32 role, address account) returns()
func (_IncentivizedInboundChannel *IncentivizedInboundChannelTransactorSession) GrantRole(role [32]byte, account common.Address) (*types.Transaction, error) {
	return _IncentivizedInboundChannel.Contract.GrantRole(&_IncentivizedInboundChannel.TransactOpts, role, account)
}

// Initialize is a paid mutator transaction binding the contract method 0x485cc955.
//
// Solidity: function initialize(address _configUpdater, address _rewardSource) returns()
func (_IncentivizedInboundChannel *IncentivizedInboundChannelTransactor) Initialize(opts *bind.TransactOpts, _configUpdater common.Address, _rewardSource common.Address) (*types.Transaction, error) {
	return _IncentivizedInboundChannel.contract.Transact(opts, "initialize", _configUpdater, _rewardSource)
}

// Initialize is a paid mutator transaction binding the contract method 0x485cc955.
//
// Solidity: function initialize(address _configUpdater, address _rewardSource) returns()
func (_IncentivizedInboundChannel *IncentivizedInboundChannelSession) Initialize(_configUpdater common.Address, _rewardSource common.Address) (*types.Transaction, error) {
	return _IncentivizedInboundChannel.Contract.Initialize(&_IncentivizedInboundChannel.TransactOpts, _configUpdater, _rewardSource)
}

// Initialize is a paid mutator transaction binding the contract method 0x485cc955.
//
// Solidity: function initialize(address _configUpdater, address _rewardSource) returns()
func (_IncentivizedInboundChannel *IncentivizedInboundChannelTransactorSession) Initialize(_configUpdater common.Address, _rewardSource common.Address) (*types.Transaction, error) {
	return _IncentivizedInboundChannel.Contract.Initialize(&_IncentivizedInboundChannel.TransactOpts, _configUpdater, _rewardSource)
}

// RenounceRole is a paid mutator transaction binding the contract method 0x36568abe.
//
// Solidity: function renounceRole(bytes32 role, address account) returns()
func (_IncentivizedInboundChannel *IncentivizedInboundChannelTransactor) RenounceRole(opts *bind.TransactOpts, role [32]byte, account common.Address) (*types.Transaction, error) {
	return _IncentivizedInboundChannel.contract.Transact(opts, "renounceRole", role, account)
}

// RenounceRole is a paid mutator transaction binding the contract method 0x36568abe.
//
// Solidity: function renounceRole(bytes32 role, address account) returns()
func (_IncentivizedInboundChannel *IncentivizedInboundChannelSession) RenounceRole(role [32]byte, account common.Address) (*types.Transaction, error) {
	return _IncentivizedInboundChannel.Contract.RenounceRole(&_IncentivizedInboundChannel.TransactOpts, role, account)
}

// RenounceRole is a paid mutator transaction binding the contract method 0x36568abe.
//
// Solidity: function renounceRole(bytes32 role, address account) returns()
func (_IncentivizedInboundChannel *IncentivizedInboundChannelTransactorSession) RenounceRole(role [32]byte, account common.Address) (*types.Transaction, error) {
	return _IncentivizedInboundChannel.Contract.RenounceRole(&_IncentivizedInboundChannel.TransactOpts, role, account)
}

// RevokeRole is a paid mutator transaction binding the contract method 0xd547741f.
//
// Solidity: function revokeRole(bytes32 role, address account) returns()
func (_IncentivizedInboundChannel *IncentivizedInboundChannelTransactor) RevokeRole(opts *bind.TransactOpts, role [32]byte, account common.Address) (*types.Transaction, error) {
	return _IncentivizedInboundChannel.contract.Transact(opts, "revokeRole", role, account)
}

// RevokeRole is a paid mutator transaction binding the contract method 0xd547741f.
//
// Solidity: function revokeRole(bytes32 role, address account) returns()
func (_IncentivizedInboundChannel *IncentivizedInboundChannelSession) RevokeRole(role [32]byte, account common.Address) (*types.Transaction, error) {
	return _IncentivizedInboundChannel.Contract.RevokeRole(&_IncentivizedInboundChannel.TransactOpts, role, account)
}

// RevokeRole is a paid mutator transaction binding the contract method 0xd547741f.
//
// Solidity: function revokeRole(bytes32 role, address account) returns()
func (_IncentivizedInboundChannel *IncentivizedInboundChannelTransactorSession) RevokeRole(role [32]byte, account common.Address) (*types.Transaction, error) {
	return _IncentivizedInboundChannel.Contract.RevokeRole(&_IncentivizedInboundChannel.TransactOpts, role, account)
}

// Submit is a paid mutator transaction binding the contract method 0x203d965f.
//
// Solidity: function submit((address,uint64,uint256,bytes)[] _messages, (bytes,bytes,(uint256,uint256,bytes32[])) _parachainVerifyInput, (uint8,uint32,bytes32,uint64,uint32,bytes32) _beefyMMRLeafPartial, (bytes32[],uint64) proof) returns()
func (_IncentivizedInboundChannel *IncentivizedInboundChannelTransactor) Submit(opts *bind.TransactOpts, _messages []IncentivizedInboundChannelMessage, _parachainVerifyInput ParachainLightClientParachainVerifyInput, _beefyMMRLeafPartial ParachainLightClientBeefyMMRLeafPartial, proof SimplifiedMMRProof) (*types.Transaction, error) {
	return _IncentivizedInboundChannel.contract.Transact(opts, "submit", _messages, _parachainVerifyInput, _beefyMMRLeafPartial, proof)
}

// Submit is a paid mutator transaction binding the contract method 0x203d965f.
//
// Solidity: function submit((address,uint64,uint256,bytes)[] _messages, (bytes,bytes,(uint256,uint256,bytes32[])) _parachainVerifyInput, (uint8,uint32,bytes32,uint64,uint32,bytes32) _beefyMMRLeafPartial, (bytes32[],uint64) proof) returns()
func (_IncentivizedInboundChannel *IncentivizedInboundChannelSession) Submit(_messages []IncentivizedInboundChannelMessage, _parachainVerifyInput ParachainLightClientParachainVerifyInput, _beefyMMRLeafPartial ParachainLightClientBeefyMMRLeafPartial, proof SimplifiedMMRProof) (*types.Transaction, error) {
	return _IncentivizedInboundChannel.Contract.Submit(&_IncentivizedInboundChannel.TransactOpts, _messages, _parachainVerifyInput, _beefyMMRLeafPartial, proof)
}

// Submit is a paid mutator transaction binding the contract method 0x203d965f.
//
// Solidity: function submit((address,uint64,uint256,bytes)[] _messages, (bytes,bytes,(uint256,uint256,bytes32[])) _parachainVerifyInput, (uint8,uint32,bytes32,uint64,uint32,bytes32) _beefyMMRLeafPartial, (bytes32[],uint64) proof) returns()
func (_IncentivizedInboundChannel *IncentivizedInboundChannelTransactorSession) Submit(_messages []IncentivizedInboundChannelMessage, _parachainVerifyInput ParachainLightClientParachainVerifyInput, _beefyMMRLeafPartial ParachainLightClientBeefyMMRLeafPartial, proof SimplifiedMMRProof) (*types.Transaction, error) {
	return _IncentivizedInboundChannel.Contract.Submit(&_IncentivizedInboundChannel.TransactOpts, _messages, _parachainVerifyInput, _beefyMMRLeafPartial, proof)
}

// IncentivizedInboundChannelMessageDispatchedIterator is returned from FilterMessageDispatched and is used to iterate over the raw logs and unpacked data for MessageDispatched events raised by the IncentivizedInboundChannel contract.
type IncentivizedInboundChannelMessageDispatchedIterator struct {
	Event *IncentivizedInboundChannelMessageDispatched // Event containing the contract specifics and raw log

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
func (it *IncentivizedInboundChannelMessageDispatchedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(IncentivizedInboundChannelMessageDispatched)
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
		it.Event = new(IncentivizedInboundChannelMessageDispatched)
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
func (it *IncentivizedInboundChannelMessageDispatchedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *IncentivizedInboundChannelMessageDispatchedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// IncentivizedInboundChannelMessageDispatched represents a MessageDispatched event raised by the IncentivizedInboundChannel contract.
type IncentivizedInboundChannelMessageDispatched struct {
	Nonce  uint64
	Result bool
	Raw    types.Log // Blockchain specific contextual infos
}

// FilterMessageDispatched is a free log retrieval operation binding the contract event 0x504b093d860dc827c72a879d052fd8ac6b4c2af80c5f3a634654f172690bf10a.
//
// Solidity: event MessageDispatched(uint64 nonce, bool result)
func (_IncentivizedInboundChannel *IncentivizedInboundChannelFilterer) FilterMessageDispatched(opts *bind.FilterOpts) (*IncentivizedInboundChannelMessageDispatchedIterator, error) {

	logs, sub, err := _IncentivizedInboundChannel.contract.FilterLogs(opts, "MessageDispatched")
	if err != nil {
		return nil, err
	}
	return &IncentivizedInboundChannelMessageDispatchedIterator{contract: _IncentivizedInboundChannel.contract, event: "MessageDispatched", logs: logs, sub: sub}, nil
}

// WatchMessageDispatched is a free log subscription operation binding the contract event 0x504b093d860dc827c72a879d052fd8ac6b4c2af80c5f3a634654f172690bf10a.
//
// Solidity: event MessageDispatched(uint64 nonce, bool result)
func (_IncentivizedInboundChannel *IncentivizedInboundChannelFilterer) WatchMessageDispatched(opts *bind.WatchOpts, sink chan<- *IncentivizedInboundChannelMessageDispatched) (event.Subscription, error) {

	logs, sub, err := _IncentivizedInboundChannel.contract.WatchLogs(opts, "MessageDispatched")
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(IncentivizedInboundChannelMessageDispatched)
				if err := _IncentivizedInboundChannel.contract.UnpackLog(event, "MessageDispatched", log); err != nil {
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
func (_IncentivizedInboundChannel *IncentivizedInboundChannelFilterer) ParseMessageDispatched(log types.Log) (*IncentivizedInboundChannelMessageDispatched, error) {
	event := new(IncentivizedInboundChannelMessageDispatched)
	if err := _IncentivizedInboundChannel.contract.UnpackLog(event, "MessageDispatched", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// IncentivizedInboundChannelRoleAdminChangedIterator is returned from FilterRoleAdminChanged and is used to iterate over the raw logs and unpacked data for RoleAdminChanged events raised by the IncentivizedInboundChannel contract.
type IncentivizedInboundChannelRoleAdminChangedIterator struct {
	Event *IncentivizedInboundChannelRoleAdminChanged // Event containing the contract specifics and raw log

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
func (it *IncentivizedInboundChannelRoleAdminChangedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(IncentivizedInboundChannelRoleAdminChanged)
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
		it.Event = new(IncentivizedInboundChannelRoleAdminChanged)
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
func (it *IncentivizedInboundChannelRoleAdminChangedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *IncentivizedInboundChannelRoleAdminChangedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// IncentivizedInboundChannelRoleAdminChanged represents a RoleAdminChanged event raised by the IncentivizedInboundChannel contract.
type IncentivizedInboundChannelRoleAdminChanged struct {
	Role              [32]byte
	PreviousAdminRole [32]byte
	NewAdminRole      [32]byte
	Raw               types.Log // Blockchain specific contextual infos
}

// FilterRoleAdminChanged is a free log retrieval operation binding the contract event 0xbd79b86ffe0ab8e8776151514217cd7cacd52c909f66475c3af44e129f0b00ff.
//
// Solidity: event RoleAdminChanged(bytes32 indexed role, bytes32 indexed previousAdminRole, bytes32 indexed newAdminRole)
func (_IncentivizedInboundChannel *IncentivizedInboundChannelFilterer) FilterRoleAdminChanged(opts *bind.FilterOpts, role [][32]byte, previousAdminRole [][32]byte, newAdminRole [][32]byte) (*IncentivizedInboundChannelRoleAdminChangedIterator, error) {

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

	logs, sub, err := _IncentivizedInboundChannel.contract.FilterLogs(opts, "RoleAdminChanged", roleRule, previousAdminRoleRule, newAdminRoleRule)
	if err != nil {
		return nil, err
	}
	return &IncentivizedInboundChannelRoleAdminChangedIterator{contract: _IncentivizedInboundChannel.contract, event: "RoleAdminChanged", logs: logs, sub: sub}, nil
}

// WatchRoleAdminChanged is a free log subscription operation binding the contract event 0xbd79b86ffe0ab8e8776151514217cd7cacd52c909f66475c3af44e129f0b00ff.
//
// Solidity: event RoleAdminChanged(bytes32 indexed role, bytes32 indexed previousAdminRole, bytes32 indexed newAdminRole)
func (_IncentivizedInboundChannel *IncentivizedInboundChannelFilterer) WatchRoleAdminChanged(opts *bind.WatchOpts, sink chan<- *IncentivizedInboundChannelRoleAdminChanged, role [][32]byte, previousAdminRole [][32]byte, newAdminRole [][32]byte) (event.Subscription, error) {

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

	logs, sub, err := _IncentivizedInboundChannel.contract.WatchLogs(opts, "RoleAdminChanged", roleRule, previousAdminRoleRule, newAdminRoleRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(IncentivizedInboundChannelRoleAdminChanged)
				if err := _IncentivizedInboundChannel.contract.UnpackLog(event, "RoleAdminChanged", log); err != nil {
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
func (_IncentivizedInboundChannel *IncentivizedInboundChannelFilterer) ParseRoleAdminChanged(log types.Log) (*IncentivizedInboundChannelRoleAdminChanged, error) {
	event := new(IncentivizedInboundChannelRoleAdminChanged)
	if err := _IncentivizedInboundChannel.contract.UnpackLog(event, "RoleAdminChanged", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// IncentivizedInboundChannelRoleGrantedIterator is returned from FilterRoleGranted and is used to iterate over the raw logs and unpacked data for RoleGranted events raised by the IncentivizedInboundChannel contract.
type IncentivizedInboundChannelRoleGrantedIterator struct {
	Event *IncentivizedInboundChannelRoleGranted // Event containing the contract specifics and raw log

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
func (it *IncentivizedInboundChannelRoleGrantedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(IncentivizedInboundChannelRoleGranted)
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
		it.Event = new(IncentivizedInboundChannelRoleGranted)
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
func (it *IncentivizedInboundChannelRoleGrantedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *IncentivizedInboundChannelRoleGrantedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// IncentivizedInboundChannelRoleGranted represents a RoleGranted event raised by the IncentivizedInboundChannel contract.
type IncentivizedInboundChannelRoleGranted struct {
	Role    [32]byte
	Account common.Address
	Sender  common.Address
	Raw     types.Log // Blockchain specific contextual infos
}

// FilterRoleGranted is a free log retrieval operation binding the contract event 0x2f8788117e7eff1d82e926ec794901d17c78024a50270940304540a733656f0d.
//
// Solidity: event RoleGranted(bytes32 indexed role, address indexed account, address indexed sender)
func (_IncentivizedInboundChannel *IncentivizedInboundChannelFilterer) FilterRoleGranted(opts *bind.FilterOpts, role [][32]byte, account []common.Address, sender []common.Address) (*IncentivizedInboundChannelRoleGrantedIterator, error) {

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

	logs, sub, err := _IncentivizedInboundChannel.contract.FilterLogs(opts, "RoleGranted", roleRule, accountRule, senderRule)
	if err != nil {
		return nil, err
	}
	return &IncentivizedInboundChannelRoleGrantedIterator{contract: _IncentivizedInboundChannel.contract, event: "RoleGranted", logs: logs, sub: sub}, nil
}

// WatchRoleGranted is a free log subscription operation binding the contract event 0x2f8788117e7eff1d82e926ec794901d17c78024a50270940304540a733656f0d.
//
// Solidity: event RoleGranted(bytes32 indexed role, address indexed account, address indexed sender)
func (_IncentivizedInboundChannel *IncentivizedInboundChannelFilterer) WatchRoleGranted(opts *bind.WatchOpts, sink chan<- *IncentivizedInboundChannelRoleGranted, role [][32]byte, account []common.Address, sender []common.Address) (event.Subscription, error) {

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

	logs, sub, err := _IncentivizedInboundChannel.contract.WatchLogs(opts, "RoleGranted", roleRule, accountRule, senderRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(IncentivizedInboundChannelRoleGranted)
				if err := _IncentivizedInboundChannel.contract.UnpackLog(event, "RoleGranted", log); err != nil {
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
func (_IncentivizedInboundChannel *IncentivizedInboundChannelFilterer) ParseRoleGranted(log types.Log) (*IncentivizedInboundChannelRoleGranted, error) {
	event := new(IncentivizedInboundChannelRoleGranted)
	if err := _IncentivizedInboundChannel.contract.UnpackLog(event, "RoleGranted", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// IncentivizedInboundChannelRoleRevokedIterator is returned from FilterRoleRevoked and is used to iterate over the raw logs and unpacked data for RoleRevoked events raised by the IncentivizedInboundChannel contract.
type IncentivizedInboundChannelRoleRevokedIterator struct {
	Event *IncentivizedInboundChannelRoleRevoked // Event containing the contract specifics and raw log

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
func (it *IncentivizedInboundChannelRoleRevokedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(IncentivizedInboundChannelRoleRevoked)
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
		it.Event = new(IncentivizedInboundChannelRoleRevoked)
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
func (it *IncentivizedInboundChannelRoleRevokedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *IncentivizedInboundChannelRoleRevokedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// IncentivizedInboundChannelRoleRevoked represents a RoleRevoked event raised by the IncentivizedInboundChannel contract.
type IncentivizedInboundChannelRoleRevoked struct {
	Role    [32]byte
	Account common.Address
	Sender  common.Address
	Raw     types.Log // Blockchain specific contextual infos
}

// FilterRoleRevoked is a free log retrieval operation binding the contract event 0xf6391f5c32d9c69d2a47ea670b442974b53935d1edc7fd64eb21e047a839171b.
//
// Solidity: event RoleRevoked(bytes32 indexed role, address indexed account, address indexed sender)
func (_IncentivizedInboundChannel *IncentivizedInboundChannelFilterer) FilterRoleRevoked(opts *bind.FilterOpts, role [][32]byte, account []common.Address, sender []common.Address) (*IncentivizedInboundChannelRoleRevokedIterator, error) {

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

	logs, sub, err := _IncentivizedInboundChannel.contract.FilterLogs(opts, "RoleRevoked", roleRule, accountRule, senderRule)
	if err != nil {
		return nil, err
	}
	return &IncentivizedInboundChannelRoleRevokedIterator{contract: _IncentivizedInboundChannel.contract, event: "RoleRevoked", logs: logs, sub: sub}, nil
}

// WatchRoleRevoked is a free log subscription operation binding the contract event 0xf6391f5c32d9c69d2a47ea670b442974b53935d1edc7fd64eb21e047a839171b.
//
// Solidity: event RoleRevoked(bytes32 indexed role, address indexed account, address indexed sender)
func (_IncentivizedInboundChannel *IncentivizedInboundChannelFilterer) WatchRoleRevoked(opts *bind.WatchOpts, sink chan<- *IncentivizedInboundChannelRoleRevoked, role [][32]byte, account []common.Address, sender []common.Address) (event.Subscription, error) {

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

	logs, sub, err := _IncentivizedInboundChannel.contract.WatchLogs(opts, "RoleRevoked", roleRule, accountRule, senderRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(IncentivizedInboundChannelRoleRevoked)
				if err := _IncentivizedInboundChannel.contract.UnpackLog(event, "RoleRevoked", log); err != nil {
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
func (_IncentivizedInboundChannel *IncentivizedInboundChannelFilterer) ParseRoleRevoked(log types.Log) (*IncentivizedInboundChannelRoleRevoked, error) {
	event := new(IncentivizedInboundChannelRoleRevoked)
	if err := _IncentivizedInboundChannel.contract.UnpackLog(event, "RoleRevoked", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}
