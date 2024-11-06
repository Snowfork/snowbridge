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
	_ = abi.ConvertType
)

// Command is an auto generated low-level Go binding around an user-defined struct.
type Command struct {
	Kind    uint8
	Gas     uint64
	Payload []byte
}

// InboundMessage is an auto generated low-level Go binding around an user-defined struct.
type InboundMessage struct {
	Origin   [32]byte
	Nonce    uint64
	Commands []Command
}

// VerificationDigestItem is an auto generated low-level Go binding around an user-defined struct.
type VerificationDigestItem struct {
	Kind              *big.Int
	ConsensusEngineID [4]byte
	Data              []byte
}

// VerificationHeadProof is an auto generated low-level Go binding around an user-defined struct.
type VerificationHeadProof struct {
	Pos   *big.Int
	Width *big.Int
	Proof [][32]byte
}

// VerificationMMRLeafPartial is an auto generated low-level Go binding around an user-defined struct.
type VerificationMMRLeafPartial struct {
	Version              uint8
	ParentNumber         uint32
	ParentHash           [32]byte
	NextAuthoritySetID   uint64
	NextAuthoritySetLen  uint32
	NextAuthoritySetRoot [32]byte
}

// VerificationParachainHeader is an auto generated low-level Go binding around an user-defined struct.
type VerificationParachainHeader struct {
	ParentHash     [32]byte
	Number         *big.Int
	StateRoot      [32]byte
	ExtrinsicsRoot [32]byte
	DigestItems    []VerificationDigestItem
}

// VerificationProof is an auto generated low-level Go binding around an user-defined struct.
type VerificationProof struct {
	Header         VerificationParachainHeader
	HeadProof      VerificationHeadProof
	LeafPartial    VerificationMMRLeafPartial
	LeafProof      [][32]byte
	LeafProofOrder *big.Int
}

// GatewayMetaData contains all meta data concerning the Gateway contract.
var GatewayMetaData = &bind.MetaData{
	ABI: "[{\"type\":\"function\",\"name\":\"agentOf\",\"inputs\":[{\"name\":\"agentID\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"}],\"outputs\":[{\"name\":\"\",\"type\":\"address\",\"internalType\":\"address\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"isTokenRegistered\",\"inputs\":[{\"name\":\"token\",\"type\":\"address\",\"internalType\":\"address\"}],\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"internalType\":\"bool\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"operatingMode\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"uint8\",\"internalType\":\"enumOperatingMode\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"v2_isDispatched\",\"inputs\":[{\"name\":\"nonce\",\"type\":\"uint64\",\"internalType\":\"uint64\"}],\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"internalType\":\"bool\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"v2_registerToken\",\"inputs\":[{\"name\":\"token\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"xcmFeeAHP\",\"type\":\"uint128\",\"internalType\":\"uint128\"}],\"outputs\":[],\"stateMutability\":\"payable\"},{\"type\":\"function\",\"name\":\"v2_registerTokenOnKusama\",\"inputs\":[{\"name\":\"token\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"xcmFeeAHP\",\"type\":\"uint128\",\"internalType\":\"uint128\"},{\"name\":\"xcmFeeAHK\",\"type\":\"uint128\",\"internalType\":\"uint128\"}],\"outputs\":[],\"stateMutability\":\"payable\"},{\"type\":\"function\",\"name\":\"v2_sendMessage\",\"inputs\":[{\"name\":\"xcm\",\"type\":\"bytes\",\"internalType\":\"bytes\"},{\"name\":\"assets\",\"type\":\"bytes[]\",\"internalType\":\"bytes[]\"},{\"name\":\"claimer\",\"type\":\"bytes\",\"internalType\":\"bytes\"}],\"outputs\":[],\"stateMutability\":\"payable\"},{\"type\":\"function\",\"name\":\"v2_submit\",\"inputs\":[{\"name\":\"message\",\"type\":\"tuple\",\"internalType\":\"structInboundMessage\",\"components\":[{\"name\":\"origin\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"},{\"name\":\"nonce\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"commands\",\"type\":\"tuple[]\",\"internalType\":\"structCommand[]\",\"components\":[{\"name\":\"kind\",\"type\":\"uint8\",\"internalType\":\"uint8\"},{\"name\":\"gas\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"payload\",\"type\":\"bytes\",\"internalType\":\"bytes\"}]}]},{\"name\":\"leafProof\",\"type\":\"bytes32[]\",\"internalType\":\"bytes32[]\"},{\"name\":\"headerProof\",\"type\":\"tuple\",\"internalType\":\"structVerification.Proof\",\"components\":[{\"name\":\"header\",\"type\":\"tuple\",\"internalType\":\"structVerification.ParachainHeader\",\"components\":[{\"name\":\"parentHash\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"},{\"name\":\"number\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"stateRoot\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"},{\"name\":\"extrinsicsRoot\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"},{\"name\":\"digestItems\",\"type\":\"tuple[]\",\"internalType\":\"structVerification.DigestItem[]\",\"components\":[{\"name\":\"kind\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"consensusEngineID\",\"type\":\"bytes4\",\"internalType\":\"bytes4\"},{\"name\":\"data\",\"type\":\"bytes\",\"internalType\":\"bytes\"}]}]},{\"name\":\"headProof\",\"type\":\"tuple\",\"internalType\":\"structVerification.HeadProof\",\"components\":[{\"name\":\"pos\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"width\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"proof\",\"type\":\"bytes32[]\",\"internalType\":\"bytes32[]\"}]},{\"name\":\"leafPartial\",\"type\":\"tuple\",\"internalType\":\"structVerification.MMRLeafPartial\",\"components\":[{\"name\":\"version\",\"type\":\"uint8\",\"internalType\":\"uint8\"},{\"name\":\"parentNumber\",\"type\":\"uint32\",\"internalType\":\"uint32\"},{\"name\":\"parentHash\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"},{\"name\":\"nextAuthoritySetID\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"nextAuthoritySetLen\",\"type\":\"uint32\",\"internalType\":\"uint32\"},{\"name\":\"nextAuthoritySetRoot\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"}]},{\"name\":\"leafProof\",\"type\":\"bytes32[]\",\"internalType\":\"bytes32[]\"},{\"name\":\"leafProofOrder\",\"type\":\"uint256\",\"internalType\":\"uint256\"}]},{\"name\":\"rewardAddress\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"event\",\"name\":\"InboundMessageDispatched\",\"inputs\":[{\"name\":\"nonce\",\"type\":\"uint64\",\"indexed\":true,\"internalType\":\"uint64\"},{\"name\":\"success\",\"type\":\"bool\",\"indexed\":false,\"internalType\":\"bool\"},{\"name\":\"rewardAddress\",\"type\":\"bytes32\",\"indexed\":true,\"internalType\":\"bytes32\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"OutboundMessageAccepted\",\"inputs\":[{\"name\":\"nonce\",\"type\":\"uint64\",\"indexed\":false,\"internalType\":\"uint64\"},{\"name\":\"reward\",\"type\":\"uint256\",\"indexed\":false,\"internalType\":\"uint256\"},{\"name\":\"payload\",\"type\":\"bytes\",\"indexed\":false,\"internalType\":\"bytes\"}],\"anonymous\":false},{\"type\":\"error\",\"name\":\"InvalidAsset\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"InvalidEtherValue\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"InvalidFee\",\"inputs\":[]}]",
}

// GatewayABI is the input ABI used to generate the binding from.
// Deprecated: Use GatewayMetaData.ABI instead.
var GatewayABI = GatewayMetaData.ABI

// Gateway is an auto generated Go binding around an Ethereum contract.
type Gateway struct {
	GatewayCaller     // Read-only binding to the contract
	GatewayTransactor // Write-only binding to the contract
	GatewayFilterer   // Log filterer for contract events
}

// GatewayCaller is an auto generated read-only Go binding around an Ethereum contract.
type GatewayCaller struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// GatewayTransactor is an auto generated write-only Go binding around an Ethereum contract.
type GatewayTransactor struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// GatewayFilterer is an auto generated log filtering Go binding around an Ethereum contract events.
type GatewayFilterer struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// GatewaySession is an auto generated Go binding around an Ethereum contract,
// with pre-set call and transact options.
type GatewaySession struct {
	Contract     *Gateway          // Generic contract binding to set the session for
	CallOpts     bind.CallOpts     // Call options to use throughout this session
	TransactOpts bind.TransactOpts // Transaction auth options to use throughout this session
}

// GatewayCallerSession is an auto generated read-only Go binding around an Ethereum contract,
// with pre-set call options.
type GatewayCallerSession struct {
	Contract *GatewayCaller // Generic contract caller binding to set the session for
	CallOpts bind.CallOpts  // Call options to use throughout this session
}

// GatewayTransactorSession is an auto generated write-only Go binding around an Ethereum contract,
// with pre-set transact options.
type GatewayTransactorSession struct {
	Contract     *GatewayTransactor // Generic contract transactor binding to set the session for
	TransactOpts bind.TransactOpts  // Transaction auth options to use throughout this session
}

// GatewayRaw is an auto generated low-level Go binding around an Ethereum contract.
type GatewayRaw struct {
	Contract *Gateway // Generic contract binding to access the raw methods on
}

// GatewayCallerRaw is an auto generated low-level read-only Go binding around an Ethereum contract.
type GatewayCallerRaw struct {
	Contract *GatewayCaller // Generic read-only contract binding to access the raw methods on
}

// GatewayTransactorRaw is an auto generated low-level write-only Go binding around an Ethereum contract.
type GatewayTransactorRaw struct {
	Contract *GatewayTransactor // Generic write-only contract binding to access the raw methods on
}

// NewGateway creates a new instance of Gateway, bound to a specific deployed contract.
func NewGateway(address common.Address, backend bind.ContractBackend) (*Gateway, error) {
	contract, err := bindGateway(address, backend, backend, backend)
	if err != nil {
		return nil, err
	}
	return &Gateway{GatewayCaller: GatewayCaller{contract: contract}, GatewayTransactor: GatewayTransactor{contract: contract}, GatewayFilterer: GatewayFilterer{contract: contract}}, nil
}

// NewGatewayCaller creates a new read-only instance of Gateway, bound to a specific deployed contract.
func NewGatewayCaller(address common.Address, caller bind.ContractCaller) (*GatewayCaller, error) {
	contract, err := bindGateway(address, caller, nil, nil)
	if err != nil {
		return nil, err
	}
	return &GatewayCaller{contract: contract}, nil
}

// NewGatewayTransactor creates a new write-only instance of Gateway, bound to a specific deployed contract.
func NewGatewayTransactor(address common.Address, transactor bind.ContractTransactor) (*GatewayTransactor, error) {
	contract, err := bindGateway(address, nil, transactor, nil)
	if err != nil {
		return nil, err
	}
	return &GatewayTransactor{contract: contract}, nil
}

// NewGatewayFilterer creates a new log filterer instance of Gateway, bound to a specific deployed contract.
func NewGatewayFilterer(address common.Address, filterer bind.ContractFilterer) (*GatewayFilterer, error) {
	contract, err := bindGateway(address, nil, nil, filterer)
	if err != nil {
		return nil, err
	}
	return &GatewayFilterer{contract: contract}, nil
}

// bindGateway binds a generic wrapper to an already deployed contract.
func bindGateway(address common.Address, caller bind.ContractCaller, transactor bind.ContractTransactor, filterer bind.ContractFilterer) (*bind.BoundContract, error) {
	parsed, err := GatewayMetaData.GetAbi()
	if err != nil {
		return nil, err
	}
	return bind.NewBoundContract(address, *parsed, caller, transactor, filterer), nil
}

// Call invokes the (constant) contract method with params as input values and
// sets the output to result. The result type might be a single field for simple
// returns, a slice of interfaces for anonymous returns and a struct for named
// returns.
func (_Gateway *GatewayRaw) Call(opts *bind.CallOpts, result *[]interface{}, method string, params ...interface{}) error {
	return _Gateway.Contract.GatewayCaller.contract.Call(opts, result, method, params...)
}

// Transfer initiates a plain transaction to move funds to the contract, calling
// its default method if one is available.
func (_Gateway *GatewayRaw) Transfer(opts *bind.TransactOpts) (*types.Transaction, error) {
	return _Gateway.Contract.GatewayTransactor.contract.Transfer(opts)
}

// Transact invokes the (paid) contract method with params as input values.
func (_Gateway *GatewayRaw) Transact(opts *bind.TransactOpts, method string, params ...interface{}) (*types.Transaction, error) {
	return _Gateway.Contract.GatewayTransactor.contract.Transact(opts, method, params...)
}

// Call invokes the (constant) contract method with params as input values and
// sets the output to result. The result type might be a single field for simple
// returns, a slice of interfaces for anonymous returns and a struct for named
// returns.
func (_Gateway *GatewayCallerRaw) Call(opts *bind.CallOpts, result *[]interface{}, method string, params ...interface{}) error {
	return _Gateway.Contract.contract.Call(opts, result, method, params...)
}

// Transfer initiates a plain transaction to move funds to the contract, calling
// its default method if one is available.
func (_Gateway *GatewayTransactorRaw) Transfer(opts *bind.TransactOpts) (*types.Transaction, error) {
	return _Gateway.Contract.contract.Transfer(opts)
}

// Transact invokes the (paid) contract method with params as input values.
func (_Gateway *GatewayTransactorRaw) Transact(opts *bind.TransactOpts, method string, params ...interface{}) (*types.Transaction, error) {
	return _Gateway.Contract.contract.Transact(opts, method, params...)
}

// AgentOf is a free data retrieval call binding the contract method 0x5e6dae26.
//
// Solidity: function agentOf(bytes32 agentID) view returns(address)
func (_Gateway *GatewayCaller) AgentOf(opts *bind.CallOpts, agentID [32]byte) (common.Address, error) {
	var out []interface{}
	err := _Gateway.contract.Call(opts, &out, "agentOf", agentID)

	if err != nil {
		return *new(common.Address), err
	}

	out0 := *abi.ConvertType(out[0], new(common.Address)).(*common.Address)

	return out0, err

}

// AgentOf is a free data retrieval call binding the contract method 0x5e6dae26.
//
// Solidity: function agentOf(bytes32 agentID) view returns(address)
func (_Gateway *GatewaySession) AgentOf(agentID [32]byte) (common.Address, error) {
	return _Gateway.Contract.AgentOf(&_Gateway.CallOpts, agentID)
}

// AgentOf is a free data retrieval call binding the contract method 0x5e6dae26.
//
// Solidity: function agentOf(bytes32 agentID) view returns(address)
func (_Gateway *GatewayCallerSession) AgentOf(agentID [32]byte) (common.Address, error) {
	return _Gateway.Contract.AgentOf(&_Gateway.CallOpts, agentID)
}

// IsTokenRegistered is a free data retrieval call binding the contract method 0x26aa101f.
//
// Solidity: function isTokenRegistered(address token) view returns(bool)
func (_Gateway *GatewayCaller) IsTokenRegistered(opts *bind.CallOpts, token common.Address) (bool, error) {
	var out []interface{}
	err := _Gateway.contract.Call(opts, &out, "isTokenRegistered", token)

	if err != nil {
		return *new(bool), err
	}

	out0 := *abi.ConvertType(out[0], new(bool)).(*bool)

	return out0, err

}

// IsTokenRegistered is a free data retrieval call binding the contract method 0x26aa101f.
//
// Solidity: function isTokenRegistered(address token) view returns(bool)
func (_Gateway *GatewaySession) IsTokenRegistered(token common.Address) (bool, error) {
	return _Gateway.Contract.IsTokenRegistered(&_Gateway.CallOpts, token)
}

// IsTokenRegistered is a free data retrieval call binding the contract method 0x26aa101f.
//
// Solidity: function isTokenRegistered(address token) view returns(bool)
func (_Gateway *GatewayCallerSession) IsTokenRegistered(token common.Address) (bool, error) {
	return _Gateway.Contract.IsTokenRegistered(&_Gateway.CallOpts, token)
}

// OperatingMode is a free data retrieval call binding the contract method 0x38004f69.
//
// Solidity: function operatingMode() view returns(uint8)
func (_Gateway *GatewayCaller) OperatingMode(opts *bind.CallOpts) (uint8, error) {
	var out []interface{}
	err := _Gateway.contract.Call(opts, &out, "operatingMode")

	if err != nil {
		return *new(uint8), err
	}

	out0 := *abi.ConvertType(out[0], new(uint8)).(*uint8)

	return out0, err

}

// OperatingMode is a free data retrieval call binding the contract method 0x38004f69.
//
// Solidity: function operatingMode() view returns(uint8)
func (_Gateway *GatewaySession) OperatingMode() (uint8, error) {
	return _Gateway.Contract.OperatingMode(&_Gateway.CallOpts)
}

// OperatingMode is a free data retrieval call binding the contract method 0x38004f69.
//
// Solidity: function operatingMode() view returns(uint8)
func (_Gateway *GatewayCallerSession) OperatingMode() (uint8, error) {
	return _Gateway.Contract.OperatingMode(&_Gateway.CallOpts)
}

// V2IsDispatched is a free data retrieval call binding the contract method 0xc66414c5.
//
// Solidity: function v2_isDispatched(uint64 nonce) view returns(bool)
func (_Gateway *GatewayCaller) V2IsDispatched(opts *bind.CallOpts, nonce uint64) (bool, error) {
	var out []interface{}
	err := _Gateway.contract.Call(opts, &out, "v2_isDispatched", nonce)

	if err != nil {
		return *new(bool), err
	}

	out0 := *abi.ConvertType(out[0], new(bool)).(*bool)

	return out0, err

}

// V2IsDispatched is a free data retrieval call binding the contract method 0xc66414c5.
//
// Solidity: function v2_isDispatched(uint64 nonce) view returns(bool)
func (_Gateway *GatewaySession) V2IsDispatched(nonce uint64) (bool, error) {
	return _Gateway.Contract.V2IsDispatched(&_Gateway.CallOpts, nonce)
}

// V2IsDispatched is a free data retrieval call binding the contract method 0xc66414c5.
//
// Solidity: function v2_isDispatched(uint64 nonce) view returns(bool)
func (_Gateway *GatewayCallerSession) V2IsDispatched(nonce uint64) (bool, error) {
	return _Gateway.Contract.V2IsDispatched(&_Gateway.CallOpts, nonce)
}

// V2RegisterToken is a paid mutator transaction binding the contract method 0xd0b8c486.
//
// Solidity: function v2_registerToken(address token, uint128 xcmFeeAHP) payable returns()
func (_Gateway *GatewayTransactor) V2RegisterToken(opts *bind.TransactOpts, token common.Address, xcmFeeAHP *big.Int) (*types.Transaction, error) {
	return _Gateway.contract.Transact(opts, "v2_registerToken", token, xcmFeeAHP)
}

// V2RegisterToken is a paid mutator transaction binding the contract method 0xd0b8c486.
//
// Solidity: function v2_registerToken(address token, uint128 xcmFeeAHP) payable returns()
func (_Gateway *GatewaySession) V2RegisterToken(token common.Address, xcmFeeAHP *big.Int) (*types.Transaction, error) {
	return _Gateway.Contract.V2RegisterToken(&_Gateway.TransactOpts, token, xcmFeeAHP)
}

// V2RegisterToken is a paid mutator transaction binding the contract method 0xd0b8c486.
//
// Solidity: function v2_registerToken(address token, uint128 xcmFeeAHP) payable returns()
func (_Gateway *GatewayTransactorSession) V2RegisterToken(token common.Address, xcmFeeAHP *big.Int) (*types.Transaction, error) {
	return _Gateway.Contract.V2RegisterToken(&_Gateway.TransactOpts, token, xcmFeeAHP)
}

// V2RegisterTokenOnKusama is a paid mutator transaction binding the contract method 0x49c481ba.
//
// Solidity: function v2_registerTokenOnKusama(address token, uint128 xcmFeeAHP, uint128 xcmFeeAHK) payable returns()
func (_Gateway *GatewayTransactor) V2RegisterTokenOnKusama(opts *bind.TransactOpts, token common.Address, xcmFeeAHP *big.Int, xcmFeeAHK *big.Int) (*types.Transaction, error) {
	return _Gateway.contract.Transact(opts, "v2_registerTokenOnKusama", token, xcmFeeAHP, xcmFeeAHK)
}

// V2RegisterTokenOnKusama is a paid mutator transaction binding the contract method 0x49c481ba.
//
// Solidity: function v2_registerTokenOnKusama(address token, uint128 xcmFeeAHP, uint128 xcmFeeAHK) payable returns()
func (_Gateway *GatewaySession) V2RegisterTokenOnKusama(token common.Address, xcmFeeAHP *big.Int, xcmFeeAHK *big.Int) (*types.Transaction, error) {
	return _Gateway.Contract.V2RegisterTokenOnKusama(&_Gateway.TransactOpts, token, xcmFeeAHP, xcmFeeAHK)
}

// V2RegisterTokenOnKusama is a paid mutator transaction binding the contract method 0x49c481ba.
//
// Solidity: function v2_registerTokenOnKusama(address token, uint128 xcmFeeAHP, uint128 xcmFeeAHK) payable returns()
func (_Gateway *GatewayTransactorSession) V2RegisterTokenOnKusama(token common.Address, xcmFeeAHP *big.Int, xcmFeeAHK *big.Int) (*types.Transaction, error) {
	return _Gateway.Contract.V2RegisterTokenOnKusama(&_Gateway.TransactOpts, token, xcmFeeAHP, xcmFeeAHK)
}

// V2SendMessage is a paid mutator transaction binding the contract method 0xb7c02d39.
//
// Solidity: function v2_sendMessage(bytes xcm, bytes[] assets, bytes claimer) payable returns()
func (_Gateway *GatewayTransactor) V2SendMessage(opts *bind.TransactOpts, xcm []byte, assets [][]byte, claimer []byte) (*types.Transaction, error) {
	return _Gateway.contract.Transact(opts, "v2_sendMessage", xcm, assets, claimer)
}

// V2SendMessage is a paid mutator transaction binding the contract method 0xb7c02d39.
//
// Solidity: function v2_sendMessage(bytes xcm, bytes[] assets, bytes claimer) payable returns()
func (_Gateway *GatewaySession) V2SendMessage(xcm []byte, assets [][]byte, claimer []byte) (*types.Transaction, error) {
	return _Gateway.Contract.V2SendMessage(&_Gateway.TransactOpts, xcm, assets, claimer)
}

// V2SendMessage is a paid mutator transaction binding the contract method 0xb7c02d39.
//
// Solidity: function v2_sendMessage(bytes xcm, bytes[] assets, bytes claimer) payable returns()
func (_Gateway *GatewayTransactorSession) V2SendMessage(xcm []byte, assets [][]byte, claimer []byte) (*types.Transaction, error) {
	return _Gateway.Contract.V2SendMessage(&_Gateway.TransactOpts, xcm, assets, claimer)
}

// V2Submit is a paid mutator transaction binding the contract method 0x9a13f0e7.
//
// Solidity: function v2_submit((bytes32,uint64,(uint8,uint64,bytes)[]) message, bytes32[] leafProof, ((bytes32,uint256,bytes32,bytes32,(uint256,bytes4,bytes)[]),(uint256,uint256,bytes32[]),(uint8,uint32,bytes32,uint64,uint32,bytes32),bytes32[],uint256) headerProof, bytes32 rewardAddress) returns()
func (_Gateway *GatewayTransactor) V2Submit(opts *bind.TransactOpts, message InboundMessage, leafProof [][32]byte, headerProof VerificationProof, rewardAddress [32]byte) (*types.Transaction, error) {
	return _Gateway.contract.Transact(opts, "v2_submit", message, leafProof, headerProof, rewardAddress)
}

// V2Submit is a paid mutator transaction binding the contract method 0x9a13f0e7.
//
// Solidity: function v2_submit((bytes32,uint64,(uint8,uint64,bytes)[]) message, bytes32[] leafProof, ((bytes32,uint256,bytes32,bytes32,(uint256,bytes4,bytes)[]),(uint256,uint256,bytes32[]),(uint8,uint32,bytes32,uint64,uint32,bytes32),bytes32[],uint256) headerProof, bytes32 rewardAddress) returns()
func (_Gateway *GatewaySession) V2Submit(message InboundMessage, leafProof [][32]byte, headerProof VerificationProof, rewardAddress [32]byte) (*types.Transaction, error) {
	return _Gateway.Contract.V2Submit(&_Gateway.TransactOpts, message, leafProof, headerProof, rewardAddress)
}

// V2Submit is a paid mutator transaction binding the contract method 0x9a13f0e7.
//
// Solidity: function v2_submit((bytes32,uint64,(uint8,uint64,bytes)[]) message, bytes32[] leafProof, ((bytes32,uint256,bytes32,bytes32,(uint256,bytes4,bytes)[]),(uint256,uint256,bytes32[]),(uint8,uint32,bytes32,uint64,uint32,bytes32),bytes32[],uint256) headerProof, bytes32 rewardAddress) returns()
func (_Gateway *GatewayTransactorSession) V2Submit(message InboundMessage, leafProof [][32]byte, headerProof VerificationProof, rewardAddress [32]byte) (*types.Transaction, error) {
	return _Gateway.Contract.V2Submit(&_Gateway.TransactOpts, message, leafProof, headerProof, rewardAddress)
}

// GatewayInboundMessageDispatchedIterator is returned from FilterInboundMessageDispatched and is used to iterate over the raw logs and unpacked data for InboundMessageDispatched events raised by the Gateway contract.
type GatewayInboundMessageDispatchedIterator struct {
	Event *GatewayInboundMessageDispatched // Event containing the contract specifics and raw log

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
func (it *GatewayInboundMessageDispatchedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(GatewayInboundMessageDispatched)
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
		it.Event = new(GatewayInboundMessageDispatched)
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
func (it *GatewayInboundMessageDispatchedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *GatewayInboundMessageDispatchedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// GatewayInboundMessageDispatched represents a InboundMessageDispatched event raised by the Gateway contract.
type GatewayInboundMessageDispatched struct {
	Nonce         uint64
	Success       bool
	RewardAddress [32]byte
	Raw           types.Log // Blockchain specific contextual infos
}

// FilterInboundMessageDispatched is a free log retrieval operation binding the contract event 0x755d3b4d173427dc415f2c82a71641bfdbc1e8f79e36a2bd0d480237e94a159b.
//
// Solidity: event InboundMessageDispatched(uint64 indexed nonce, bool success, bytes32 indexed rewardAddress)
func (_Gateway *GatewayFilterer) FilterInboundMessageDispatched(opts *bind.FilterOpts, nonce []uint64, rewardAddress [][32]byte) (*GatewayInboundMessageDispatchedIterator, error) {

	var nonceRule []interface{}
	for _, nonceItem := range nonce {
		nonceRule = append(nonceRule, nonceItem)
	}

	var rewardAddressRule []interface{}
	for _, rewardAddressItem := range rewardAddress {
		rewardAddressRule = append(rewardAddressRule, rewardAddressItem)
	}

	logs, sub, err := _Gateway.contract.FilterLogs(opts, "InboundMessageDispatched", nonceRule, rewardAddressRule)
	if err != nil {
		return nil, err
	}
	return &GatewayInboundMessageDispatchedIterator{contract: _Gateway.contract, event: "InboundMessageDispatched", logs: logs, sub: sub}, nil
}

// WatchInboundMessageDispatched is a free log subscription operation binding the contract event 0x755d3b4d173427dc415f2c82a71641bfdbc1e8f79e36a2bd0d480237e94a159b.
//
// Solidity: event InboundMessageDispatched(uint64 indexed nonce, bool success, bytes32 indexed rewardAddress)
func (_Gateway *GatewayFilterer) WatchInboundMessageDispatched(opts *bind.WatchOpts, sink chan<- *GatewayInboundMessageDispatched, nonce []uint64, rewardAddress [][32]byte) (event.Subscription, error) {

	var nonceRule []interface{}
	for _, nonceItem := range nonce {
		nonceRule = append(nonceRule, nonceItem)
	}

	var rewardAddressRule []interface{}
	for _, rewardAddressItem := range rewardAddress {
		rewardAddressRule = append(rewardAddressRule, rewardAddressItem)
	}

	logs, sub, err := _Gateway.contract.WatchLogs(opts, "InboundMessageDispatched", nonceRule, rewardAddressRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(GatewayInboundMessageDispatched)
				if err := _Gateway.contract.UnpackLog(event, "InboundMessageDispatched", log); err != nil {
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

// ParseInboundMessageDispatched is a log parse operation binding the contract event 0x755d3b4d173427dc415f2c82a71641bfdbc1e8f79e36a2bd0d480237e94a159b.
//
// Solidity: event InboundMessageDispatched(uint64 indexed nonce, bool success, bytes32 indexed rewardAddress)
func (_Gateway *GatewayFilterer) ParseInboundMessageDispatched(log types.Log) (*GatewayInboundMessageDispatched, error) {
	event := new(GatewayInboundMessageDispatched)
	if err := _Gateway.contract.UnpackLog(event, "InboundMessageDispatched", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// GatewayOutboundMessageAcceptedIterator is returned from FilterOutboundMessageAccepted and is used to iterate over the raw logs and unpacked data for OutboundMessageAccepted events raised by the Gateway contract.
type GatewayOutboundMessageAcceptedIterator struct {
	Event *GatewayOutboundMessageAccepted // Event containing the contract specifics and raw log

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
func (it *GatewayOutboundMessageAcceptedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(GatewayOutboundMessageAccepted)
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
		it.Event = new(GatewayOutboundMessageAccepted)
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
func (it *GatewayOutboundMessageAcceptedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *GatewayOutboundMessageAcceptedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// GatewayOutboundMessageAccepted represents a OutboundMessageAccepted event raised by the Gateway contract.
type GatewayOutboundMessageAccepted struct {
	Nonce   uint64
	Reward  *big.Int
	Payload []byte
	Raw     types.Log // Blockchain specific contextual infos
}

// FilterOutboundMessageAccepted is a free log retrieval operation binding the contract event 0xf2297c2b692d37814ed50b9ff0f52bf87ab5d1692a651f2a9ee8a872fdea1dda.
//
// Solidity: event OutboundMessageAccepted(uint64 nonce, uint256 reward, bytes payload)
func (_Gateway *GatewayFilterer) FilterOutboundMessageAccepted(opts *bind.FilterOpts) (*GatewayOutboundMessageAcceptedIterator, error) {

	logs, sub, err := _Gateway.contract.FilterLogs(opts, "OutboundMessageAccepted")
	if err != nil {
		return nil, err
	}
	return &GatewayOutboundMessageAcceptedIterator{contract: _Gateway.contract, event: "OutboundMessageAccepted", logs: logs, sub: sub}, nil
}

// WatchOutboundMessageAccepted is a free log subscription operation binding the contract event 0xf2297c2b692d37814ed50b9ff0f52bf87ab5d1692a651f2a9ee8a872fdea1dda.
//
// Solidity: event OutboundMessageAccepted(uint64 nonce, uint256 reward, bytes payload)
func (_Gateway *GatewayFilterer) WatchOutboundMessageAccepted(opts *bind.WatchOpts, sink chan<- *GatewayOutboundMessageAccepted) (event.Subscription, error) {

	logs, sub, err := _Gateway.contract.WatchLogs(opts, "OutboundMessageAccepted")
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(GatewayOutboundMessageAccepted)
				if err := _Gateway.contract.UnpackLog(event, "OutboundMessageAccepted", log); err != nil {
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

// ParseOutboundMessageAccepted is a log parse operation binding the contract event 0xf2297c2b692d37814ed50b9ff0f52bf87ab5d1692a651f2a9ee8a872fdea1dda.
//
// Solidity: event OutboundMessageAccepted(uint64 nonce, uint256 reward, bytes payload)
func (_Gateway *GatewayFilterer) ParseOutboundMessageAccepted(log types.Log) (*GatewayOutboundMessageAccepted, error) {
	event := new(GatewayOutboundMessageAccepted)
	if err := _Gateway.contract.UnpackLog(event, "OutboundMessageAccepted", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}
