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

// IBeefyClientCommitment is an auto generated low-level Go binding around an user-defined struct.
type IBeefyClientCommitment struct {
	BlockNumber    uint32
	ValidatorSetID uint64
	Payload        []IBeefyClientPayloadItem
}

// IBeefyClientMMRLeaf is an auto generated low-level Go binding around an user-defined struct.
type IBeefyClientMMRLeaf struct {
	Version              uint8
	ParentNumber         uint32
	ParentHash           [32]byte
	NextAuthoritySetID   uint64
	NextAuthoritySetLen  uint32
	NextAuthoritySetRoot [32]byte
	ParachainHeadsRoot   [32]byte
}

// IBeefyClientPayloadItem is an auto generated low-level Go binding around an user-defined struct.
type IBeefyClientPayloadItem struct {
	PayloadID [2]byte
	Data      []byte
}

// IBeefyClientValidatorProof is an auto generated low-level Go binding around an user-defined struct.
type IBeefyClientValidatorProof struct {
	V       uint8
	R       [32]byte
	S       [32]byte
	Index   *big.Int
	Account common.Address
	Proof   [][32]byte
}

// BeefyClientWrapperMetaData contains all meta data concerning the BeefyClientWrapper contract.
var BeefyClientWrapperMetaData = &bind.MetaData{
	ABI: "[{\"type\":\"constructor\",\"inputs\":[{\"name\":\"_gateway\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"_owner\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"_maxGasPrice\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"_maxRefundAmount\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"_refundTarget\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"nonpayable\"},{\"type\":\"receive\",\"stateMutability\":\"payable\"},{\"type\":\"function\",\"name\":\"clearTicket\",\"inputs\":[{\"name\":\"commitmentHash\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"commitPrevRandao\",\"inputs\":[{\"name\":\"commitmentHash\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"creditedCost\",\"inputs\":[{\"name\":\"\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"}],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"gateway\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"address\",\"internalType\":\"address\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"highestPendingBlock\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"highestPendingBlockTimestamp\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"maxGasPrice\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"maxRefundAmount\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"owner\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"address\",\"internalType\":\"address\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"refundTarget\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"setMaxGasPrice\",\"inputs\":[{\"name\":\"_maxGasPrice\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"setMaxRefundAmount\",\"inputs\":[{\"name\":\"_maxRefundAmount\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"setRefundTarget\",\"inputs\":[{\"name\":\"_refundTarget\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"submitFiatShamir\",\"inputs\":[{\"name\":\"commitment\",\"type\":\"tuple\",\"internalType\":\"structIBeefyClient.Commitment\",\"components\":[{\"name\":\"blockNumber\",\"type\":\"uint32\",\"internalType\":\"uint32\"},{\"name\":\"validatorSetID\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"payload\",\"type\":\"tuple[]\",\"internalType\":\"structIBeefyClient.PayloadItem[]\",\"components\":[{\"name\":\"payloadID\",\"type\":\"bytes2\",\"internalType\":\"bytes2\"},{\"name\":\"data\",\"type\":\"bytes\",\"internalType\":\"bytes\"}]}]},{\"name\":\"bitfield\",\"type\":\"uint256[]\",\"internalType\":\"uint256[]\"},{\"name\":\"proofs\",\"type\":\"tuple[]\",\"internalType\":\"structIBeefyClient.ValidatorProof[]\",\"components\":[{\"name\":\"v\",\"type\":\"uint8\",\"internalType\":\"uint8\"},{\"name\":\"r\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"},{\"name\":\"s\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"},{\"name\":\"index\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"account\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"proof\",\"type\":\"bytes32[]\",\"internalType\":\"bytes32[]\"}]},{\"name\":\"leaf\",\"type\":\"tuple\",\"internalType\":\"structIBeefyClient.MMRLeaf\",\"components\":[{\"name\":\"version\",\"type\":\"uint8\",\"internalType\":\"uint8\"},{\"name\":\"parentNumber\",\"type\":\"uint32\",\"internalType\":\"uint32\"},{\"name\":\"parentHash\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"},{\"name\":\"nextAuthoritySetID\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"nextAuthoritySetLen\",\"type\":\"uint32\",\"internalType\":\"uint32\"},{\"name\":\"nextAuthoritySetRoot\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"},{\"name\":\"parachainHeadsRoot\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"}]},{\"name\":\"leafProof\",\"type\":\"bytes32[]\",\"internalType\":\"bytes32[]\"},{\"name\":\"leafProofOrder\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"submitFinal\",\"inputs\":[{\"name\":\"commitment\",\"type\":\"tuple\",\"internalType\":\"structIBeefyClient.Commitment\",\"components\":[{\"name\":\"blockNumber\",\"type\":\"uint32\",\"internalType\":\"uint32\"},{\"name\":\"validatorSetID\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"payload\",\"type\":\"tuple[]\",\"internalType\":\"structIBeefyClient.PayloadItem[]\",\"components\":[{\"name\":\"payloadID\",\"type\":\"bytes2\",\"internalType\":\"bytes2\"},{\"name\":\"data\",\"type\":\"bytes\",\"internalType\":\"bytes\"}]}]},{\"name\":\"bitfield\",\"type\":\"uint256[]\",\"internalType\":\"uint256[]\"},{\"name\":\"proofs\",\"type\":\"tuple[]\",\"internalType\":\"structIBeefyClient.ValidatorProof[]\",\"components\":[{\"name\":\"v\",\"type\":\"uint8\",\"internalType\":\"uint8\"},{\"name\":\"r\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"},{\"name\":\"s\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"},{\"name\":\"index\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"account\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"proof\",\"type\":\"bytes32[]\",\"internalType\":\"bytes32[]\"}]},{\"name\":\"leaf\",\"type\":\"tuple\",\"internalType\":\"structIBeefyClient.MMRLeaf\",\"components\":[{\"name\":\"version\",\"type\":\"uint8\",\"internalType\":\"uint8\"},{\"name\":\"parentNumber\",\"type\":\"uint32\",\"internalType\":\"uint32\"},{\"name\":\"parentHash\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"},{\"name\":\"nextAuthoritySetID\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"nextAuthoritySetLen\",\"type\":\"uint32\",\"internalType\":\"uint32\"},{\"name\":\"nextAuthoritySetRoot\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"},{\"name\":\"parachainHeadsRoot\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"}]},{\"name\":\"leafProof\",\"type\":\"bytes32[]\",\"internalType\":\"bytes32[]\"},{\"name\":\"leafProofOrder\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"submitInitial\",\"inputs\":[{\"name\":\"commitment\",\"type\":\"tuple\",\"internalType\":\"structIBeefyClient.Commitment\",\"components\":[{\"name\":\"blockNumber\",\"type\":\"uint32\",\"internalType\":\"uint32\"},{\"name\":\"validatorSetID\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"payload\",\"type\":\"tuple[]\",\"internalType\":\"structIBeefyClient.PayloadItem[]\",\"components\":[{\"name\":\"payloadID\",\"type\":\"bytes2\",\"internalType\":\"bytes2\"},{\"name\":\"data\",\"type\":\"bytes\",\"internalType\":\"bytes\"}]}]},{\"name\":\"bitfield\",\"type\":\"uint256[]\",\"internalType\":\"uint256[]\"},{\"name\":\"proof\",\"type\":\"tuple\",\"internalType\":\"structIBeefyClient.ValidatorProof\",\"components\":[{\"name\":\"v\",\"type\":\"uint8\",\"internalType\":\"uint8\"},{\"name\":\"r\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"},{\"name\":\"s\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"},{\"name\":\"index\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"account\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"proof\",\"type\":\"bytes32[]\",\"internalType\":\"bytes32[]\"}]}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"ticketOwner\",\"inputs\":[{\"name\":\"\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"}],\"outputs\":[{\"name\":\"\",\"type\":\"address\",\"internalType\":\"address\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"transferOwnership\",\"inputs\":[{\"name\":\"newOwner\",\"type\":\"address\",\"internalType\":\"address\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"withdrawFunds\",\"inputs\":[{\"name\":\"recipient\",\"type\":\"address\",\"internalType\":\"addresspayable\"},{\"name\":\"amount\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"event\",\"name\":\"CostCredited\",\"inputs\":[{\"name\":\"relayer\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"commitmentHash\",\"type\":\"bytes32\",\"indexed\":true,\"internalType\":\"bytes32\"},{\"name\":\"cost\",\"type\":\"uint256\",\"indexed\":false,\"internalType\":\"uint256\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"FundsDeposited\",\"inputs\":[{\"name\":\"depositor\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"amount\",\"type\":\"uint256\",\"indexed\":false,\"internalType\":\"uint256\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"FundsWithdrawn\",\"inputs\":[{\"name\":\"recipient\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"amount\",\"type\":\"uint256\",\"indexed\":false,\"internalType\":\"uint256\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"SubmissionRefunded\",\"inputs\":[{\"name\":\"relayer\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"progress\",\"type\":\"uint256\",\"indexed\":false,\"internalType\":\"uint256\"},{\"name\":\"refundAmount\",\"type\":\"uint256\",\"indexed\":false,\"internalType\":\"uint256\"}],\"anonymous\":false},{\"type\":\"error\",\"name\":\"InvalidAddress\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"NotTicketOwner\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"TicketAlreadyOwned\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"TransferFailed\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"Unauthorized\",\"inputs\":[]}]",
}

// BeefyClientWrapperABI is the input ABI used to generate the binding from.
// Deprecated: Use BeefyClientWrapperMetaData.ABI instead.
var BeefyClientWrapperABI = BeefyClientWrapperMetaData.ABI

// BeefyClientWrapper is an auto generated Go binding around an Ethereum contract.
type BeefyClientWrapper struct {
	BeefyClientWrapperCaller     // Read-only binding to the contract
	BeefyClientWrapperTransactor // Write-only binding to the contract
	BeefyClientWrapperFilterer   // Log filterer for contract events
}

// BeefyClientWrapperCaller is an auto generated read-only Go binding around an Ethereum contract.
type BeefyClientWrapperCaller struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// BeefyClientWrapperTransactor is an auto generated write-only Go binding around an Ethereum contract.
type BeefyClientWrapperTransactor struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// BeefyClientWrapperFilterer is an auto generated log filtering Go binding around an Ethereum contract events.
type BeefyClientWrapperFilterer struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// BeefyClientWrapperSession is an auto generated Go binding around an Ethereum contract,
// with pre-set call and transact options.
type BeefyClientWrapperSession struct {
	Contract     *BeefyClientWrapper // Generic contract binding to set the session for
	CallOpts     bind.CallOpts       // Call options to use throughout this session
	TransactOpts bind.TransactOpts   // Transaction auth options to use throughout this session
}

// BeefyClientWrapperCallerSession is an auto generated read-only Go binding around an Ethereum contract,
// with pre-set call options.
type BeefyClientWrapperCallerSession struct {
	Contract *BeefyClientWrapperCaller // Generic contract caller binding to set the session for
	CallOpts bind.CallOpts             // Call options to use throughout this session
}

// BeefyClientWrapperTransactorSession is an auto generated write-only Go binding around an Ethereum contract,
// with pre-set transact options.
type BeefyClientWrapperTransactorSession struct {
	Contract     *BeefyClientWrapperTransactor // Generic contract transactor binding to set the session for
	TransactOpts bind.TransactOpts             // Transaction auth options to use throughout this session
}

// BeefyClientWrapperRaw is an auto generated low-level Go binding around an Ethereum contract.
type BeefyClientWrapperRaw struct {
	Contract *BeefyClientWrapper // Generic contract binding to access the raw methods on
}

// BeefyClientWrapperCallerRaw is an auto generated low-level read-only Go binding around an Ethereum contract.
type BeefyClientWrapperCallerRaw struct {
	Contract *BeefyClientWrapperCaller // Generic read-only contract binding to access the raw methods on
}

// BeefyClientWrapperTransactorRaw is an auto generated low-level write-only Go binding around an Ethereum contract.
type BeefyClientWrapperTransactorRaw struct {
	Contract *BeefyClientWrapperTransactor // Generic write-only contract binding to access the raw methods on
}

// NewBeefyClientWrapper creates a new instance of BeefyClientWrapper, bound to a specific deployed contract.
func NewBeefyClientWrapper(address common.Address, backend bind.ContractBackend) (*BeefyClientWrapper, error) {
	contract, err := bindBeefyClientWrapper(address, backend, backend, backend)
	if err != nil {
		return nil, err
	}
	return &BeefyClientWrapper{BeefyClientWrapperCaller: BeefyClientWrapperCaller{contract: contract}, BeefyClientWrapperTransactor: BeefyClientWrapperTransactor{contract: contract}, BeefyClientWrapperFilterer: BeefyClientWrapperFilterer{contract: contract}}, nil
}

// NewBeefyClientWrapperCaller creates a new read-only instance of BeefyClientWrapper, bound to a specific deployed contract.
func NewBeefyClientWrapperCaller(address common.Address, caller bind.ContractCaller) (*BeefyClientWrapperCaller, error) {
	contract, err := bindBeefyClientWrapper(address, caller, nil, nil)
	if err != nil {
		return nil, err
	}
	return &BeefyClientWrapperCaller{contract: contract}, nil
}

// NewBeefyClientWrapperTransactor creates a new write-only instance of BeefyClientWrapper, bound to a specific deployed contract.
func NewBeefyClientWrapperTransactor(address common.Address, transactor bind.ContractTransactor) (*BeefyClientWrapperTransactor, error) {
	contract, err := bindBeefyClientWrapper(address, nil, transactor, nil)
	if err != nil {
		return nil, err
	}
	return &BeefyClientWrapperTransactor{contract: contract}, nil
}

// NewBeefyClientWrapperFilterer creates a new log filterer instance of BeefyClientWrapper, bound to a specific deployed contract.
func NewBeefyClientWrapperFilterer(address common.Address, filterer bind.ContractFilterer) (*BeefyClientWrapperFilterer, error) {
	contract, err := bindBeefyClientWrapper(address, nil, nil, filterer)
	if err != nil {
		return nil, err
	}
	return &BeefyClientWrapperFilterer{contract: contract}, nil
}

// bindBeefyClientWrapper binds a generic wrapper to an already deployed contract.
func bindBeefyClientWrapper(address common.Address, caller bind.ContractCaller, transactor bind.ContractTransactor, filterer bind.ContractFilterer) (*bind.BoundContract, error) {
	parsed, err := BeefyClientWrapperMetaData.GetAbi()
	if err != nil {
		return nil, err
	}
	return bind.NewBoundContract(address, *parsed, caller, transactor, filterer), nil
}

// Call invokes the (constant) contract method with params as input values and
// sets the output to result. The result type might be a single field for simple
// returns, a slice of interfaces for anonymous returns and a struct for named
// returns.
func (_BeefyClientWrapper *BeefyClientWrapperRaw) Call(opts *bind.CallOpts, result *[]interface{}, method string, params ...interface{}) error {
	return _BeefyClientWrapper.Contract.BeefyClientWrapperCaller.contract.Call(opts, result, method, params...)
}

// Transfer initiates a plain transaction to move funds to the contract, calling
// its default method if one is available.
func (_BeefyClientWrapper *BeefyClientWrapperRaw) Transfer(opts *bind.TransactOpts) (*types.Transaction, error) {
	return _BeefyClientWrapper.Contract.BeefyClientWrapperTransactor.contract.Transfer(opts)
}

// Transact invokes the (paid) contract method with params as input values.
func (_BeefyClientWrapper *BeefyClientWrapperRaw) Transact(opts *bind.TransactOpts, method string, params ...interface{}) (*types.Transaction, error) {
	return _BeefyClientWrapper.Contract.BeefyClientWrapperTransactor.contract.Transact(opts, method, params...)
}

// Call invokes the (constant) contract method with params as input values and
// sets the output to result. The result type might be a single field for simple
// returns, a slice of interfaces for anonymous returns and a struct for named
// returns.
func (_BeefyClientWrapper *BeefyClientWrapperCallerRaw) Call(opts *bind.CallOpts, result *[]interface{}, method string, params ...interface{}) error {
	return _BeefyClientWrapper.Contract.contract.Call(opts, result, method, params...)
}

// Transfer initiates a plain transaction to move funds to the contract, calling
// its default method if one is available.
func (_BeefyClientWrapper *BeefyClientWrapperTransactorRaw) Transfer(opts *bind.TransactOpts) (*types.Transaction, error) {
	return _BeefyClientWrapper.Contract.contract.Transfer(opts)
}

// Transact invokes the (paid) contract method with params as input values.
func (_BeefyClientWrapper *BeefyClientWrapperTransactorRaw) Transact(opts *bind.TransactOpts, method string, params ...interface{}) (*types.Transaction, error) {
	return _BeefyClientWrapper.Contract.contract.Transact(opts, method, params...)
}

// CreditedCost is a free data retrieval call binding the contract method 0x029f2f34.
//
// Solidity: function creditedCost(bytes32 ) view returns(uint256)
func (_BeefyClientWrapper *BeefyClientWrapperCaller) CreditedCost(opts *bind.CallOpts, arg0 [32]byte) (*big.Int, error) {
	var out []interface{}
	err := _BeefyClientWrapper.contract.Call(opts, &out, "creditedCost", arg0)

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// CreditedCost is a free data retrieval call binding the contract method 0x029f2f34.
//
// Solidity: function creditedCost(bytes32 ) view returns(uint256)
func (_BeefyClientWrapper *BeefyClientWrapperSession) CreditedCost(arg0 [32]byte) (*big.Int, error) {
	return _BeefyClientWrapper.Contract.CreditedCost(&_BeefyClientWrapper.CallOpts, arg0)
}

// CreditedCost is a free data retrieval call binding the contract method 0x029f2f34.
//
// Solidity: function creditedCost(bytes32 ) view returns(uint256)
func (_BeefyClientWrapper *BeefyClientWrapperCallerSession) CreditedCost(arg0 [32]byte) (*big.Int, error) {
	return _BeefyClientWrapper.Contract.CreditedCost(&_BeefyClientWrapper.CallOpts, arg0)
}

// Gateway is a free data retrieval call binding the contract method 0x116191b6.
//
// Solidity: function gateway() view returns(address)
func (_BeefyClientWrapper *BeefyClientWrapperCaller) Gateway(opts *bind.CallOpts) (common.Address, error) {
	var out []interface{}
	err := _BeefyClientWrapper.contract.Call(opts, &out, "gateway")

	if err != nil {
		return *new(common.Address), err
	}

	out0 := *abi.ConvertType(out[0], new(common.Address)).(*common.Address)

	return out0, err

}

// Gateway is a free data retrieval call binding the contract method 0x116191b6.
//
// Solidity: function gateway() view returns(address)
func (_BeefyClientWrapper *BeefyClientWrapperSession) Gateway() (common.Address, error) {
	return _BeefyClientWrapper.Contract.Gateway(&_BeefyClientWrapper.CallOpts)
}

// Gateway is a free data retrieval call binding the contract method 0x116191b6.
//
// Solidity: function gateway() view returns(address)
func (_BeefyClientWrapper *BeefyClientWrapperCallerSession) Gateway() (common.Address, error) {
	return _BeefyClientWrapper.Contract.Gateway(&_BeefyClientWrapper.CallOpts)
}

// HighestPendingBlock is a free data retrieval call binding the contract method 0x0dc2be13.
//
// Solidity: function highestPendingBlock() view returns(uint256)
func (_BeefyClientWrapper *BeefyClientWrapperCaller) HighestPendingBlock(opts *bind.CallOpts) (*big.Int, error) {
	var out []interface{}
	err := _BeefyClientWrapper.contract.Call(opts, &out, "highestPendingBlock")

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// HighestPendingBlock is a free data retrieval call binding the contract method 0x0dc2be13.
//
// Solidity: function highestPendingBlock() view returns(uint256)
func (_BeefyClientWrapper *BeefyClientWrapperSession) HighestPendingBlock() (*big.Int, error) {
	return _BeefyClientWrapper.Contract.HighestPendingBlock(&_BeefyClientWrapper.CallOpts)
}

// HighestPendingBlock is a free data retrieval call binding the contract method 0x0dc2be13.
//
// Solidity: function highestPendingBlock() view returns(uint256)
func (_BeefyClientWrapper *BeefyClientWrapperCallerSession) HighestPendingBlock() (*big.Int, error) {
	return _BeefyClientWrapper.Contract.HighestPendingBlock(&_BeefyClientWrapper.CallOpts)
}

// HighestPendingBlockTimestamp is a free data retrieval call binding the contract method 0x33e2c682.
//
// Solidity: function highestPendingBlockTimestamp() view returns(uint256)
func (_BeefyClientWrapper *BeefyClientWrapperCaller) HighestPendingBlockTimestamp(opts *bind.CallOpts) (*big.Int, error) {
	var out []interface{}
	err := _BeefyClientWrapper.contract.Call(opts, &out, "highestPendingBlockTimestamp")

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// HighestPendingBlockTimestamp is a free data retrieval call binding the contract method 0x33e2c682.
//
// Solidity: function highestPendingBlockTimestamp() view returns(uint256)
func (_BeefyClientWrapper *BeefyClientWrapperSession) HighestPendingBlockTimestamp() (*big.Int, error) {
	return _BeefyClientWrapper.Contract.HighestPendingBlockTimestamp(&_BeefyClientWrapper.CallOpts)
}

// HighestPendingBlockTimestamp is a free data retrieval call binding the contract method 0x33e2c682.
//
// Solidity: function highestPendingBlockTimestamp() view returns(uint256)
func (_BeefyClientWrapper *BeefyClientWrapperCallerSession) HighestPendingBlockTimestamp() (*big.Int, error) {
	return _BeefyClientWrapper.Contract.HighestPendingBlockTimestamp(&_BeefyClientWrapper.CallOpts)
}

// MaxGasPrice is a free data retrieval call binding the contract method 0x3de39c11.
//
// Solidity: function maxGasPrice() view returns(uint256)
func (_BeefyClientWrapper *BeefyClientWrapperCaller) MaxGasPrice(opts *bind.CallOpts) (*big.Int, error) {
	var out []interface{}
	err := _BeefyClientWrapper.contract.Call(opts, &out, "maxGasPrice")

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// MaxGasPrice is a free data retrieval call binding the contract method 0x3de39c11.
//
// Solidity: function maxGasPrice() view returns(uint256)
func (_BeefyClientWrapper *BeefyClientWrapperSession) MaxGasPrice() (*big.Int, error) {
	return _BeefyClientWrapper.Contract.MaxGasPrice(&_BeefyClientWrapper.CallOpts)
}

// MaxGasPrice is a free data retrieval call binding the contract method 0x3de39c11.
//
// Solidity: function maxGasPrice() view returns(uint256)
func (_BeefyClientWrapper *BeefyClientWrapperCallerSession) MaxGasPrice() (*big.Int, error) {
	return _BeefyClientWrapper.Contract.MaxGasPrice(&_BeefyClientWrapper.CallOpts)
}

// MaxRefundAmount is a free data retrieval call binding the contract method 0x28bbc5c1.
//
// Solidity: function maxRefundAmount() view returns(uint256)
func (_BeefyClientWrapper *BeefyClientWrapperCaller) MaxRefundAmount(opts *bind.CallOpts) (*big.Int, error) {
	var out []interface{}
	err := _BeefyClientWrapper.contract.Call(opts, &out, "maxRefundAmount")

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// MaxRefundAmount is a free data retrieval call binding the contract method 0x28bbc5c1.
//
// Solidity: function maxRefundAmount() view returns(uint256)
func (_BeefyClientWrapper *BeefyClientWrapperSession) MaxRefundAmount() (*big.Int, error) {
	return _BeefyClientWrapper.Contract.MaxRefundAmount(&_BeefyClientWrapper.CallOpts)
}

// MaxRefundAmount is a free data retrieval call binding the contract method 0x28bbc5c1.
//
// Solidity: function maxRefundAmount() view returns(uint256)
func (_BeefyClientWrapper *BeefyClientWrapperCallerSession) MaxRefundAmount() (*big.Int, error) {
	return _BeefyClientWrapper.Contract.MaxRefundAmount(&_BeefyClientWrapper.CallOpts)
}

// Owner is a free data retrieval call binding the contract method 0x8da5cb5b.
//
// Solidity: function owner() view returns(address)
func (_BeefyClientWrapper *BeefyClientWrapperCaller) Owner(opts *bind.CallOpts) (common.Address, error) {
	var out []interface{}
	err := _BeefyClientWrapper.contract.Call(opts, &out, "owner")

	if err != nil {
		return *new(common.Address), err
	}

	out0 := *abi.ConvertType(out[0], new(common.Address)).(*common.Address)

	return out0, err

}

// Owner is a free data retrieval call binding the contract method 0x8da5cb5b.
//
// Solidity: function owner() view returns(address)
func (_BeefyClientWrapper *BeefyClientWrapperSession) Owner() (common.Address, error) {
	return _BeefyClientWrapper.Contract.Owner(&_BeefyClientWrapper.CallOpts)
}

// Owner is a free data retrieval call binding the contract method 0x8da5cb5b.
//
// Solidity: function owner() view returns(address)
func (_BeefyClientWrapper *BeefyClientWrapperCallerSession) Owner() (common.Address, error) {
	return _BeefyClientWrapper.Contract.Owner(&_BeefyClientWrapper.CallOpts)
}

// RefundTarget is a free data retrieval call binding the contract method 0xd679e02a.
//
// Solidity: function refundTarget() view returns(uint256)
func (_BeefyClientWrapper *BeefyClientWrapperCaller) RefundTarget(opts *bind.CallOpts) (*big.Int, error) {
	var out []interface{}
	err := _BeefyClientWrapper.contract.Call(opts, &out, "refundTarget")

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// RefundTarget is a free data retrieval call binding the contract method 0xd679e02a.
//
// Solidity: function refundTarget() view returns(uint256)
func (_BeefyClientWrapper *BeefyClientWrapperSession) RefundTarget() (*big.Int, error) {
	return _BeefyClientWrapper.Contract.RefundTarget(&_BeefyClientWrapper.CallOpts)
}

// RefundTarget is a free data retrieval call binding the contract method 0xd679e02a.
//
// Solidity: function refundTarget() view returns(uint256)
func (_BeefyClientWrapper *BeefyClientWrapperCallerSession) RefundTarget() (*big.Int, error) {
	return _BeefyClientWrapper.Contract.RefundTarget(&_BeefyClientWrapper.CallOpts)
}

// TicketOwner is a free data retrieval call binding the contract method 0xd2e82bfe.
//
// Solidity: function ticketOwner(bytes32 ) view returns(address)
func (_BeefyClientWrapper *BeefyClientWrapperCaller) TicketOwner(opts *bind.CallOpts, arg0 [32]byte) (common.Address, error) {
	var out []interface{}
	err := _BeefyClientWrapper.contract.Call(opts, &out, "ticketOwner", arg0)

	if err != nil {
		return *new(common.Address), err
	}

	out0 := *abi.ConvertType(out[0], new(common.Address)).(*common.Address)

	return out0, err

}

// TicketOwner is a free data retrieval call binding the contract method 0xd2e82bfe.
//
// Solidity: function ticketOwner(bytes32 ) view returns(address)
func (_BeefyClientWrapper *BeefyClientWrapperSession) TicketOwner(arg0 [32]byte) (common.Address, error) {
	return _BeefyClientWrapper.Contract.TicketOwner(&_BeefyClientWrapper.CallOpts, arg0)
}

// TicketOwner is a free data retrieval call binding the contract method 0xd2e82bfe.
//
// Solidity: function ticketOwner(bytes32 ) view returns(address)
func (_BeefyClientWrapper *BeefyClientWrapperCallerSession) TicketOwner(arg0 [32]byte) (common.Address, error) {
	return _BeefyClientWrapper.Contract.TicketOwner(&_BeefyClientWrapper.CallOpts, arg0)
}

// ClearTicket is a paid mutator transaction binding the contract method 0xa82218ed.
//
// Solidity: function clearTicket(bytes32 commitmentHash) returns()
func (_BeefyClientWrapper *BeefyClientWrapperTransactor) ClearTicket(opts *bind.TransactOpts, commitmentHash [32]byte) (*types.Transaction, error) {
	return _BeefyClientWrapper.contract.Transact(opts, "clearTicket", commitmentHash)
}

// ClearTicket is a paid mutator transaction binding the contract method 0xa82218ed.
//
// Solidity: function clearTicket(bytes32 commitmentHash) returns()
func (_BeefyClientWrapper *BeefyClientWrapperSession) ClearTicket(commitmentHash [32]byte) (*types.Transaction, error) {
	return _BeefyClientWrapper.Contract.ClearTicket(&_BeefyClientWrapper.TransactOpts, commitmentHash)
}

// ClearTicket is a paid mutator transaction binding the contract method 0xa82218ed.
//
// Solidity: function clearTicket(bytes32 commitmentHash) returns()
func (_BeefyClientWrapper *BeefyClientWrapperTransactorSession) ClearTicket(commitmentHash [32]byte) (*types.Transaction, error) {
	return _BeefyClientWrapper.Contract.ClearTicket(&_BeefyClientWrapper.TransactOpts, commitmentHash)
}

// CommitPrevRandao is a paid mutator transaction binding the contract method 0xa77cf3d2.
//
// Solidity: function commitPrevRandao(bytes32 commitmentHash) returns()
func (_BeefyClientWrapper *BeefyClientWrapperTransactor) CommitPrevRandao(opts *bind.TransactOpts, commitmentHash [32]byte) (*types.Transaction, error) {
	return _BeefyClientWrapper.contract.Transact(opts, "commitPrevRandao", commitmentHash)
}

// CommitPrevRandao is a paid mutator transaction binding the contract method 0xa77cf3d2.
//
// Solidity: function commitPrevRandao(bytes32 commitmentHash) returns()
func (_BeefyClientWrapper *BeefyClientWrapperSession) CommitPrevRandao(commitmentHash [32]byte) (*types.Transaction, error) {
	return _BeefyClientWrapper.Contract.CommitPrevRandao(&_BeefyClientWrapper.TransactOpts, commitmentHash)
}

// CommitPrevRandao is a paid mutator transaction binding the contract method 0xa77cf3d2.
//
// Solidity: function commitPrevRandao(bytes32 commitmentHash) returns()
func (_BeefyClientWrapper *BeefyClientWrapperTransactorSession) CommitPrevRandao(commitmentHash [32]byte) (*types.Transaction, error) {
	return _BeefyClientWrapper.Contract.CommitPrevRandao(&_BeefyClientWrapper.TransactOpts, commitmentHash)
}

// SetMaxGasPrice is a paid mutator transaction binding the contract method 0xd2fa635e.
//
// Solidity: function setMaxGasPrice(uint256 _maxGasPrice) returns()
func (_BeefyClientWrapper *BeefyClientWrapperTransactor) SetMaxGasPrice(opts *bind.TransactOpts, _maxGasPrice *big.Int) (*types.Transaction, error) {
	return _BeefyClientWrapper.contract.Transact(opts, "setMaxGasPrice", _maxGasPrice)
}

// SetMaxGasPrice is a paid mutator transaction binding the contract method 0xd2fa635e.
//
// Solidity: function setMaxGasPrice(uint256 _maxGasPrice) returns()
func (_BeefyClientWrapper *BeefyClientWrapperSession) SetMaxGasPrice(_maxGasPrice *big.Int) (*types.Transaction, error) {
	return _BeefyClientWrapper.Contract.SetMaxGasPrice(&_BeefyClientWrapper.TransactOpts, _maxGasPrice)
}

// SetMaxGasPrice is a paid mutator transaction binding the contract method 0xd2fa635e.
//
// Solidity: function setMaxGasPrice(uint256 _maxGasPrice) returns()
func (_BeefyClientWrapper *BeefyClientWrapperTransactorSession) SetMaxGasPrice(_maxGasPrice *big.Int) (*types.Transaction, error) {
	return _BeefyClientWrapper.Contract.SetMaxGasPrice(&_BeefyClientWrapper.TransactOpts, _maxGasPrice)
}

// SetMaxRefundAmount is a paid mutator transaction binding the contract method 0x2efbeccd.
//
// Solidity: function setMaxRefundAmount(uint256 _maxRefundAmount) returns()
func (_BeefyClientWrapper *BeefyClientWrapperTransactor) SetMaxRefundAmount(opts *bind.TransactOpts, _maxRefundAmount *big.Int) (*types.Transaction, error) {
	return _BeefyClientWrapper.contract.Transact(opts, "setMaxRefundAmount", _maxRefundAmount)
}

// SetMaxRefundAmount is a paid mutator transaction binding the contract method 0x2efbeccd.
//
// Solidity: function setMaxRefundAmount(uint256 _maxRefundAmount) returns()
func (_BeefyClientWrapper *BeefyClientWrapperSession) SetMaxRefundAmount(_maxRefundAmount *big.Int) (*types.Transaction, error) {
	return _BeefyClientWrapper.Contract.SetMaxRefundAmount(&_BeefyClientWrapper.TransactOpts, _maxRefundAmount)
}

// SetMaxRefundAmount is a paid mutator transaction binding the contract method 0x2efbeccd.
//
// Solidity: function setMaxRefundAmount(uint256 _maxRefundAmount) returns()
func (_BeefyClientWrapper *BeefyClientWrapperTransactorSession) SetMaxRefundAmount(_maxRefundAmount *big.Int) (*types.Transaction, error) {
	return _BeefyClientWrapper.Contract.SetMaxRefundAmount(&_BeefyClientWrapper.TransactOpts, _maxRefundAmount)
}

// SetRefundTarget is a paid mutator transaction binding the contract method 0xc15ea4b5.
//
// Solidity: function setRefundTarget(uint256 _refundTarget) returns()
func (_BeefyClientWrapper *BeefyClientWrapperTransactor) SetRefundTarget(opts *bind.TransactOpts, _refundTarget *big.Int) (*types.Transaction, error) {
	return _BeefyClientWrapper.contract.Transact(opts, "setRefundTarget", _refundTarget)
}

// SetRefundTarget is a paid mutator transaction binding the contract method 0xc15ea4b5.
//
// Solidity: function setRefundTarget(uint256 _refundTarget) returns()
func (_BeefyClientWrapper *BeefyClientWrapperSession) SetRefundTarget(_refundTarget *big.Int) (*types.Transaction, error) {
	return _BeefyClientWrapper.Contract.SetRefundTarget(&_BeefyClientWrapper.TransactOpts, _refundTarget)
}

// SetRefundTarget is a paid mutator transaction binding the contract method 0xc15ea4b5.
//
// Solidity: function setRefundTarget(uint256 _refundTarget) returns()
func (_BeefyClientWrapper *BeefyClientWrapperTransactorSession) SetRefundTarget(_refundTarget *big.Int) (*types.Transaction, error) {
	return _BeefyClientWrapper.Contract.SetRefundTarget(&_BeefyClientWrapper.TransactOpts, _refundTarget)
}

// SubmitFiatShamir is a paid mutator transaction binding the contract method 0xc7d6e93d.
//
// Solidity: function submitFiatShamir((uint32,uint64,(bytes2,bytes)[]) commitment, uint256[] bitfield, (uint8,bytes32,bytes32,uint256,address,bytes32[])[] proofs, (uint8,uint32,bytes32,uint64,uint32,bytes32,bytes32) leaf, bytes32[] leafProof, uint256 leafProofOrder) returns()
func (_BeefyClientWrapper *BeefyClientWrapperTransactor) SubmitFiatShamir(opts *bind.TransactOpts, commitment IBeefyClientCommitment, bitfield []*big.Int, proofs []IBeefyClientValidatorProof, leaf IBeefyClientMMRLeaf, leafProof [][32]byte, leafProofOrder *big.Int) (*types.Transaction, error) {
	return _BeefyClientWrapper.contract.Transact(opts, "submitFiatShamir", commitment, bitfield, proofs, leaf, leafProof, leafProofOrder)
}

// SubmitFiatShamir is a paid mutator transaction binding the contract method 0xc7d6e93d.
//
// Solidity: function submitFiatShamir((uint32,uint64,(bytes2,bytes)[]) commitment, uint256[] bitfield, (uint8,bytes32,bytes32,uint256,address,bytes32[])[] proofs, (uint8,uint32,bytes32,uint64,uint32,bytes32,bytes32) leaf, bytes32[] leafProof, uint256 leafProofOrder) returns()
func (_BeefyClientWrapper *BeefyClientWrapperSession) SubmitFiatShamir(commitment IBeefyClientCommitment, bitfield []*big.Int, proofs []IBeefyClientValidatorProof, leaf IBeefyClientMMRLeaf, leafProof [][32]byte, leafProofOrder *big.Int) (*types.Transaction, error) {
	return _BeefyClientWrapper.Contract.SubmitFiatShamir(&_BeefyClientWrapper.TransactOpts, commitment, bitfield, proofs, leaf, leafProof, leafProofOrder)
}

// SubmitFiatShamir is a paid mutator transaction binding the contract method 0xc7d6e93d.
//
// Solidity: function submitFiatShamir((uint32,uint64,(bytes2,bytes)[]) commitment, uint256[] bitfield, (uint8,bytes32,bytes32,uint256,address,bytes32[])[] proofs, (uint8,uint32,bytes32,uint64,uint32,bytes32,bytes32) leaf, bytes32[] leafProof, uint256 leafProofOrder) returns()
func (_BeefyClientWrapper *BeefyClientWrapperTransactorSession) SubmitFiatShamir(commitment IBeefyClientCommitment, bitfield []*big.Int, proofs []IBeefyClientValidatorProof, leaf IBeefyClientMMRLeaf, leafProof [][32]byte, leafProofOrder *big.Int) (*types.Transaction, error) {
	return _BeefyClientWrapper.Contract.SubmitFiatShamir(&_BeefyClientWrapper.TransactOpts, commitment, bitfield, proofs, leaf, leafProof, leafProofOrder)
}

// SubmitFinal is a paid mutator transaction binding the contract method 0x623b223d.
//
// Solidity: function submitFinal((uint32,uint64,(bytes2,bytes)[]) commitment, uint256[] bitfield, (uint8,bytes32,bytes32,uint256,address,bytes32[])[] proofs, (uint8,uint32,bytes32,uint64,uint32,bytes32,bytes32) leaf, bytes32[] leafProof, uint256 leafProofOrder) returns()
func (_BeefyClientWrapper *BeefyClientWrapperTransactor) SubmitFinal(opts *bind.TransactOpts, commitment IBeefyClientCommitment, bitfield []*big.Int, proofs []IBeefyClientValidatorProof, leaf IBeefyClientMMRLeaf, leafProof [][32]byte, leafProofOrder *big.Int) (*types.Transaction, error) {
	return _BeefyClientWrapper.contract.Transact(opts, "submitFinal", commitment, bitfield, proofs, leaf, leafProof, leafProofOrder)
}

// SubmitFinal is a paid mutator transaction binding the contract method 0x623b223d.
//
// Solidity: function submitFinal((uint32,uint64,(bytes2,bytes)[]) commitment, uint256[] bitfield, (uint8,bytes32,bytes32,uint256,address,bytes32[])[] proofs, (uint8,uint32,bytes32,uint64,uint32,bytes32,bytes32) leaf, bytes32[] leafProof, uint256 leafProofOrder) returns()
func (_BeefyClientWrapper *BeefyClientWrapperSession) SubmitFinal(commitment IBeefyClientCommitment, bitfield []*big.Int, proofs []IBeefyClientValidatorProof, leaf IBeefyClientMMRLeaf, leafProof [][32]byte, leafProofOrder *big.Int) (*types.Transaction, error) {
	return _BeefyClientWrapper.Contract.SubmitFinal(&_BeefyClientWrapper.TransactOpts, commitment, bitfield, proofs, leaf, leafProof, leafProofOrder)
}

// SubmitFinal is a paid mutator transaction binding the contract method 0x623b223d.
//
// Solidity: function submitFinal((uint32,uint64,(bytes2,bytes)[]) commitment, uint256[] bitfield, (uint8,bytes32,bytes32,uint256,address,bytes32[])[] proofs, (uint8,uint32,bytes32,uint64,uint32,bytes32,bytes32) leaf, bytes32[] leafProof, uint256 leafProofOrder) returns()
func (_BeefyClientWrapper *BeefyClientWrapperTransactorSession) SubmitFinal(commitment IBeefyClientCommitment, bitfield []*big.Int, proofs []IBeefyClientValidatorProof, leaf IBeefyClientMMRLeaf, leafProof [][32]byte, leafProofOrder *big.Int) (*types.Transaction, error) {
	return _BeefyClientWrapper.Contract.SubmitFinal(&_BeefyClientWrapper.TransactOpts, commitment, bitfield, proofs, leaf, leafProof, leafProofOrder)
}

// SubmitInitial is a paid mutator transaction binding the contract method 0xbb51f1eb.
//
// Solidity: function submitInitial((uint32,uint64,(bytes2,bytes)[]) commitment, uint256[] bitfield, (uint8,bytes32,bytes32,uint256,address,bytes32[]) proof) returns()
func (_BeefyClientWrapper *BeefyClientWrapperTransactor) SubmitInitial(opts *bind.TransactOpts, commitment IBeefyClientCommitment, bitfield []*big.Int, proof IBeefyClientValidatorProof) (*types.Transaction, error) {
	return _BeefyClientWrapper.contract.Transact(opts, "submitInitial", commitment, bitfield, proof)
}

// SubmitInitial is a paid mutator transaction binding the contract method 0xbb51f1eb.
//
// Solidity: function submitInitial((uint32,uint64,(bytes2,bytes)[]) commitment, uint256[] bitfield, (uint8,bytes32,bytes32,uint256,address,bytes32[]) proof) returns()
func (_BeefyClientWrapper *BeefyClientWrapperSession) SubmitInitial(commitment IBeefyClientCommitment, bitfield []*big.Int, proof IBeefyClientValidatorProof) (*types.Transaction, error) {
	return _BeefyClientWrapper.Contract.SubmitInitial(&_BeefyClientWrapper.TransactOpts, commitment, bitfield, proof)
}

// SubmitInitial is a paid mutator transaction binding the contract method 0xbb51f1eb.
//
// Solidity: function submitInitial((uint32,uint64,(bytes2,bytes)[]) commitment, uint256[] bitfield, (uint8,bytes32,bytes32,uint256,address,bytes32[]) proof) returns()
func (_BeefyClientWrapper *BeefyClientWrapperTransactorSession) SubmitInitial(commitment IBeefyClientCommitment, bitfield []*big.Int, proof IBeefyClientValidatorProof) (*types.Transaction, error) {
	return _BeefyClientWrapper.Contract.SubmitInitial(&_BeefyClientWrapper.TransactOpts, commitment, bitfield, proof)
}

// TransferOwnership is a paid mutator transaction binding the contract method 0xf2fde38b.
//
// Solidity: function transferOwnership(address newOwner) returns()
func (_BeefyClientWrapper *BeefyClientWrapperTransactor) TransferOwnership(opts *bind.TransactOpts, newOwner common.Address) (*types.Transaction, error) {
	return _BeefyClientWrapper.contract.Transact(opts, "transferOwnership", newOwner)
}

// TransferOwnership is a paid mutator transaction binding the contract method 0xf2fde38b.
//
// Solidity: function transferOwnership(address newOwner) returns()
func (_BeefyClientWrapper *BeefyClientWrapperSession) TransferOwnership(newOwner common.Address) (*types.Transaction, error) {
	return _BeefyClientWrapper.Contract.TransferOwnership(&_BeefyClientWrapper.TransactOpts, newOwner)
}

// TransferOwnership is a paid mutator transaction binding the contract method 0xf2fde38b.
//
// Solidity: function transferOwnership(address newOwner) returns()
func (_BeefyClientWrapper *BeefyClientWrapperTransactorSession) TransferOwnership(newOwner common.Address) (*types.Transaction, error) {
	return _BeefyClientWrapper.Contract.TransferOwnership(&_BeefyClientWrapper.TransactOpts, newOwner)
}

// WithdrawFunds is a paid mutator transaction binding the contract method 0xc1075329.
//
// Solidity: function withdrawFunds(address recipient, uint256 amount) returns()
func (_BeefyClientWrapper *BeefyClientWrapperTransactor) WithdrawFunds(opts *bind.TransactOpts, recipient common.Address, amount *big.Int) (*types.Transaction, error) {
	return _BeefyClientWrapper.contract.Transact(opts, "withdrawFunds", recipient, amount)
}

// WithdrawFunds is a paid mutator transaction binding the contract method 0xc1075329.
//
// Solidity: function withdrawFunds(address recipient, uint256 amount) returns()
func (_BeefyClientWrapper *BeefyClientWrapperSession) WithdrawFunds(recipient common.Address, amount *big.Int) (*types.Transaction, error) {
	return _BeefyClientWrapper.Contract.WithdrawFunds(&_BeefyClientWrapper.TransactOpts, recipient, amount)
}

// WithdrawFunds is a paid mutator transaction binding the contract method 0xc1075329.
//
// Solidity: function withdrawFunds(address recipient, uint256 amount) returns()
func (_BeefyClientWrapper *BeefyClientWrapperTransactorSession) WithdrawFunds(recipient common.Address, amount *big.Int) (*types.Transaction, error) {
	return _BeefyClientWrapper.Contract.WithdrawFunds(&_BeefyClientWrapper.TransactOpts, recipient, amount)
}

// Receive is a paid mutator transaction binding the contract receive function.
//
// Solidity: receive() payable returns()
func (_BeefyClientWrapper *BeefyClientWrapperTransactor) Receive(opts *bind.TransactOpts) (*types.Transaction, error) {
	return _BeefyClientWrapper.contract.RawTransact(opts, nil) // calldata is disallowed for receive function
}

// Receive is a paid mutator transaction binding the contract receive function.
//
// Solidity: receive() payable returns()
func (_BeefyClientWrapper *BeefyClientWrapperSession) Receive() (*types.Transaction, error) {
	return _BeefyClientWrapper.Contract.Receive(&_BeefyClientWrapper.TransactOpts)
}

// Receive is a paid mutator transaction binding the contract receive function.
//
// Solidity: receive() payable returns()
func (_BeefyClientWrapper *BeefyClientWrapperTransactorSession) Receive() (*types.Transaction, error) {
	return _BeefyClientWrapper.Contract.Receive(&_BeefyClientWrapper.TransactOpts)
}

// BeefyClientWrapperCostCreditedIterator is returned from FilterCostCredited and is used to iterate over the raw logs and unpacked data for CostCredited events raised by the BeefyClientWrapper contract.
type BeefyClientWrapperCostCreditedIterator struct {
	Event *BeefyClientWrapperCostCredited // Event containing the contract specifics and raw log

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
func (it *BeefyClientWrapperCostCreditedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(BeefyClientWrapperCostCredited)
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
		it.Event = new(BeefyClientWrapperCostCredited)
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
func (it *BeefyClientWrapperCostCreditedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *BeefyClientWrapperCostCreditedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// BeefyClientWrapperCostCredited represents a CostCredited event raised by the BeefyClientWrapper contract.
type BeefyClientWrapperCostCredited struct {
	Relayer        common.Address
	CommitmentHash [32]byte
	Cost           *big.Int
	Raw            types.Log // Blockchain specific contextual infos
}

// FilterCostCredited is a free log retrieval operation binding the contract event 0x793a43b692b5c13204c15d826f125eb739e34b8a6d486f4f94de6b11a7f15cea.
//
// Solidity: event CostCredited(address indexed relayer, bytes32 indexed commitmentHash, uint256 cost)
func (_BeefyClientWrapper *BeefyClientWrapperFilterer) FilterCostCredited(opts *bind.FilterOpts, relayer []common.Address, commitmentHash [][32]byte) (*BeefyClientWrapperCostCreditedIterator, error) {

	var relayerRule []interface{}
	for _, relayerItem := range relayer {
		relayerRule = append(relayerRule, relayerItem)
	}
	var commitmentHashRule []interface{}
	for _, commitmentHashItem := range commitmentHash {
		commitmentHashRule = append(commitmentHashRule, commitmentHashItem)
	}

	logs, sub, err := _BeefyClientWrapper.contract.FilterLogs(opts, "CostCredited", relayerRule, commitmentHashRule)
	if err != nil {
		return nil, err
	}
	return &BeefyClientWrapperCostCreditedIterator{contract: _BeefyClientWrapper.contract, event: "CostCredited", logs: logs, sub: sub}, nil
}

// WatchCostCredited is a free log subscription operation binding the contract event 0x793a43b692b5c13204c15d826f125eb739e34b8a6d486f4f94de6b11a7f15cea.
//
// Solidity: event CostCredited(address indexed relayer, bytes32 indexed commitmentHash, uint256 cost)
func (_BeefyClientWrapper *BeefyClientWrapperFilterer) WatchCostCredited(opts *bind.WatchOpts, sink chan<- *BeefyClientWrapperCostCredited, relayer []common.Address, commitmentHash [][32]byte) (event.Subscription, error) {

	var relayerRule []interface{}
	for _, relayerItem := range relayer {
		relayerRule = append(relayerRule, relayerItem)
	}
	var commitmentHashRule []interface{}
	for _, commitmentHashItem := range commitmentHash {
		commitmentHashRule = append(commitmentHashRule, commitmentHashItem)
	}

	logs, sub, err := _BeefyClientWrapper.contract.WatchLogs(opts, "CostCredited", relayerRule, commitmentHashRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(BeefyClientWrapperCostCredited)
				if err := _BeefyClientWrapper.contract.UnpackLog(event, "CostCredited", log); err != nil {
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

// ParseCostCredited is a log parse operation binding the contract event 0x793a43b692b5c13204c15d826f125eb739e34b8a6d486f4f94de6b11a7f15cea.
//
// Solidity: event CostCredited(address indexed relayer, bytes32 indexed commitmentHash, uint256 cost)
func (_BeefyClientWrapper *BeefyClientWrapperFilterer) ParseCostCredited(log types.Log) (*BeefyClientWrapperCostCredited, error) {
	event := new(BeefyClientWrapperCostCredited)
	if err := _BeefyClientWrapper.contract.UnpackLog(event, "CostCredited", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// BeefyClientWrapperFundsDepositedIterator is returned from FilterFundsDeposited and is used to iterate over the raw logs and unpacked data for FundsDeposited events raised by the BeefyClientWrapper contract.
type BeefyClientWrapperFundsDepositedIterator struct {
	Event *BeefyClientWrapperFundsDeposited // Event containing the contract specifics and raw log

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
func (it *BeefyClientWrapperFundsDepositedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(BeefyClientWrapperFundsDeposited)
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
		it.Event = new(BeefyClientWrapperFundsDeposited)
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
func (it *BeefyClientWrapperFundsDepositedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *BeefyClientWrapperFundsDepositedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// BeefyClientWrapperFundsDeposited represents a FundsDeposited event raised by the BeefyClientWrapper contract.
type BeefyClientWrapperFundsDeposited struct {
	Depositor common.Address
	Amount    *big.Int
	Raw       types.Log // Blockchain specific contextual infos
}

// FilterFundsDeposited is a free log retrieval operation binding the contract event 0x543ba50a5eec5e6178218e364b1d0f396157b3c8fa278522c2cb7fd99407d474.
//
// Solidity: event FundsDeposited(address indexed depositor, uint256 amount)
func (_BeefyClientWrapper *BeefyClientWrapperFilterer) FilterFundsDeposited(opts *bind.FilterOpts, depositor []common.Address) (*BeefyClientWrapperFundsDepositedIterator, error) {

	var depositorRule []interface{}
	for _, depositorItem := range depositor {
		depositorRule = append(depositorRule, depositorItem)
	}

	logs, sub, err := _BeefyClientWrapper.contract.FilterLogs(opts, "FundsDeposited", depositorRule)
	if err != nil {
		return nil, err
	}
	return &BeefyClientWrapperFundsDepositedIterator{contract: _BeefyClientWrapper.contract, event: "FundsDeposited", logs: logs, sub: sub}, nil
}

// WatchFundsDeposited is a free log subscription operation binding the contract event 0x543ba50a5eec5e6178218e364b1d0f396157b3c8fa278522c2cb7fd99407d474.
//
// Solidity: event FundsDeposited(address indexed depositor, uint256 amount)
func (_BeefyClientWrapper *BeefyClientWrapperFilterer) WatchFundsDeposited(opts *bind.WatchOpts, sink chan<- *BeefyClientWrapperFundsDeposited, depositor []common.Address) (event.Subscription, error) {

	var depositorRule []interface{}
	for _, depositorItem := range depositor {
		depositorRule = append(depositorRule, depositorItem)
	}

	logs, sub, err := _BeefyClientWrapper.contract.WatchLogs(opts, "FundsDeposited", depositorRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(BeefyClientWrapperFundsDeposited)
				if err := _BeefyClientWrapper.contract.UnpackLog(event, "FundsDeposited", log); err != nil {
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

// ParseFundsDeposited is a log parse operation binding the contract event 0x543ba50a5eec5e6178218e364b1d0f396157b3c8fa278522c2cb7fd99407d474.
//
// Solidity: event FundsDeposited(address indexed depositor, uint256 amount)
func (_BeefyClientWrapper *BeefyClientWrapperFilterer) ParseFundsDeposited(log types.Log) (*BeefyClientWrapperFundsDeposited, error) {
	event := new(BeefyClientWrapperFundsDeposited)
	if err := _BeefyClientWrapper.contract.UnpackLog(event, "FundsDeposited", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// BeefyClientWrapperFundsWithdrawnIterator is returned from FilterFundsWithdrawn and is used to iterate over the raw logs and unpacked data for FundsWithdrawn events raised by the BeefyClientWrapper contract.
type BeefyClientWrapperFundsWithdrawnIterator struct {
	Event *BeefyClientWrapperFundsWithdrawn // Event containing the contract specifics and raw log

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
func (it *BeefyClientWrapperFundsWithdrawnIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(BeefyClientWrapperFundsWithdrawn)
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
		it.Event = new(BeefyClientWrapperFundsWithdrawn)
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
func (it *BeefyClientWrapperFundsWithdrawnIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *BeefyClientWrapperFundsWithdrawnIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// BeefyClientWrapperFundsWithdrawn represents a FundsWithdrawn event raised by the BeefyClientWrapper contract.
type BeefyClientWrapperFundsWithdrawn struct {
	Recipient common.Address
	Amount    *big.Int
	Raw       types.Log // Blockchain specific contextual infos
}

// FilterFundsWithdrawn is a free log retrieval operation binding the contract event 0xeaff4b37086828766ad3268786972c0cd24259d4c87a80f9d3963a3c3d999b0d.
//
// Solidity: event FundsWithdrawn(address indexed recipient, uint256 amount)
func (_BeefyClientWrapper *BeefyClientWrapperFilterer) FilterFundsWithdrawn(opts *bind.FilterOpts, recipient []common.Address) (*BeefyClientWrapperFundsWithdrawnIterator, error) {

	var recipientRule []interface{}
	for _, recipientItem := range recipient {
		recipientRule = append(recipientRule, recipientItem)
	}

	logs, sub, err := _BeefyClientWrapper.contract.FilterLogs(opts, "FundsWithdrawn", recipientRule)
	if err != nil {
		return nil, err
	}
	return &BeefyClientWrapperFundsWithdrawnIterator{contract: _BeefyClientWrapper.contract, event: "FundsWithdrawn", logs: logs, sub: sub}, nil
}

// WatchFundsWithdrawn is a free log subscription operation binding the contract event 0xeaff4b37086828766ad3268786972c0cd24259d4c87a80f9d3963a3c3d999b0d.
//
// Solidity: event FundsWithdrawn(address indexed recipient, uint256 amount)
func (_BeefyClientWrapper *BeefyClientWrapperFilterer) WatchFundsWithdrawn(opts *bind.WatchOpts, sink chan<- *BeefyClientWrapperFundsWithdrawn, recipient []common.Address) (event.Subscription, error) {

	var recipientRule []interface{}
	for _, recipientItem := range recipient {
		recipientRule = append(recipientRule, recipientItem)
	}

	logs, sub, err := _BeefyClientWrapper.contract.WatchLogs(opts, "FundsWithdrawn", recipientRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(BeefyClientWrapperFundsWithdrawn)
				if err := _BeefyClientWrapper.contract.UnpackLog(event, "FundsWithdrawn", log); err != nil {
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

// ParseFundsWithdrawn is a log parse operation binding the contract event 0xeaff4b37086828766ad3268786972c0cd24259d4c87a80f9d3963a3c3d999b0d.
//
// Solidity: event FundsWithdrawn(address indexed recipient, uint256 amount)
func (_BeefyClientWrapper *BeefyClientWrapperFilterer) ParseFundsWithdrawn(log types.Log) (*BeefyClientWrapperFundsWithdrawn, error) {
	event := new(BeefyClientWrapperFundsWithdrawn)
	if err := _BeefyClientWrapper.contract.UnpackLog(event, "FundsWithdrawn", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// BeefyClientWrapperSubmissionRefundedIterator is returned from FilterSubmissionRefunded and is used to iterate over the raw logs and unpacked data for SubmissionRefunded events raised by the BeefyClientWrapper contract.
type BeefyClientWrapperSubmissionRefundedIterator struct {
	Event *BeefyClientWrapperSubmissionRefunded // Event containing the contract specifics and raw log

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
func (it *BeefyClientWrapperSubmissionRefundedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(BeefyClientWrapperSubmissionRefunded)
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
		it.Event = new(BeefyClientWrapperSubmissionRefunded)
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
func (it *BeefyClientWrapperSubmissionRefundedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *BeefyClientWrapperSubmissionRefundedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// BeefyClientWrapperSubmissionRefunded represents a SubmissionRefunded event raised by the BeefyClientWrapper contract.
type BeefyClientWrapperSubmissionRefunded struct {
	Relayer      common.Address
	Progress     *big.Int
	RefundAmount *big.Int
	Raw          types.Log // Blockchain specific contextual infos
}

// FilterSubmissionRefunded is a free log retrieval operation binding the contract event 0x103cb711554967fbeea0c6394a6dfedc44d0e729191e719d0976438783912f52.
//
// Solidity: event SubmissionRefunded(address indexed relayer, uint256 progress, uint256 refundAmount)
func (_BeefyClientWrapper *BeefyClientWrapperFilterer) FilterSubmissionRefunded(opts *bind.FilterOpts, relayer []common.Address) (*BeefyClientWrapperSubmissionRefundedIterator, error) {

	var relayerRule []interface{}
	for _, relayerItem := range relayer {
		relayerRule = append(relayerRule, relayerItem)
	}

	logs, sub, err := _BeefyClientWrapper.contract.FilterLogs(opts, "SubmissionRefunded", relayerRule)
	if err != nil {
		return nil, err
	}
	return &BeefyClientWrapperSubmissionRefundedIterator{contract: _BeefyClientWrapper.contract, event: "SubmissionRefunded", logs: logs, sub: sub}, nil
}

// WatchSubmissionRefunded is a free log subscription operation binding the contract event 0x103cb711554967fbeea0c6394a6dfedc44d0e729191e719d0976438783912f52.
//
// Solidity: event SubmissionRefunded(address indexed relayer, uint256 progress, uint256 refundAmount)
func (_BeefyClientWrapper *BeefyClientWrapperFilterer) WatchSubmissionRefunded(opts *bind.WatchOpts, sink chan<- *BeefyClientWrapperSubmissionRefunded, relayer []common.Address) (event.Subscription, error) {

	var relayerRule []interface{}
	for _, relayerItem := range relayer {
		relayerRule = append(relayerRule, relayerItem)
	}

	logs, sub, err := _BeefyClientWrapper.contract.WatchLogs(opts, "SubmissionRefunded", relayerRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(BeefyClientWrapperSubmissionRefunded)
				if err := _BeefyClientWrapper.contract.UnpackLog(event, "SubmissionRefunded", log); err != nil {
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

// ParseSubmissionRefunded is a log parse operation binding the contract event 0x103cb711554967fbeea0c6394a6dfedc44d0e729191e719d0976438783912f52.
//
// Solidity: event SubmissionRefunded(address indexed relayer, uint256 progress, uint256 refundAmount)
func (_BeefyClientWrapper *BeefyClientWrapperFilterer) ParseSubmissionRefunded(log types.Log) (*BeefyClientWrapperSubmissionRefunded, error) {
	event := new(BeefyClientWrapperSubmissionRefunded)
	if err := _BeefyClientWrapper.contract.UnpackLog(event, "SubmissionRefunded", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}
