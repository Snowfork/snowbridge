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
	ABI: "[{\"type\":\"receive\",\"stateMutability\":\"payable\"},{\"type\":\"function\",\"name\":\"activeTicket\",\"inputs\":[{\"name\":\"\",\"type\":\"address\",\"internalType\":\"address\"}],\"outputs\":[{\"name\":\"\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"addRelayer\",\"inputs\":[{\"name\":\"relayer\",\"type\":\"address\",\"internalType\":\"address\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"addTip\",\"inputs\":[{\"name\":\"beefyBlockNumber\",\"type\":\"uint32\",\"internalType\":\"uint32\"}],\"outputs\":[],\"stateMutability\":\"payable\"},{\"type\":\"function\",\"name\":\"beefyClient\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"address\",\"internalType\":\"contractIBeefyClient\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"clearTicket\",\"inputs\":[],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"commitPrevRandao\",\"inputs\":[{\"name\":\"commitmentHash\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"createFinalBitfield\",\"inputs\":[{\"name\":\"commitmentHash\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"},{\"name\":\"bitfield\",\"type\":\"uint256[]\",\"internalType\":\"uint256[]\"}],\"outputs\":[{\"name\":\"\",\"type\":\"uint256[]\",\"internalType\":\"uint256[]\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"createInitialBitfield\",\"inputs\":[{\"name\":\"bitsToSet\",\"type\":\"uint256[]\",\"internalType\":\"uint256[]\"},{\"name\":\"length\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[{\"name\":\"\",\"type\":\"uint256[]\",\"internalType\":\"uint256[]\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"creditedGas\",\"inputs\":[{\"name\":\"\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"}],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"currentTurnIndex\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"currentValidatorSet\",\"inputs\":[],\"outputs\":[{\"name\":\"id\",\"type\":\"uint128\",\"internalType\":\"uint128\"},{\"name\":\"length\",\"type\":\"uint128\",\"internalType\":\"uint128\"},{\"name\":\"root\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"getBalance\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"getCreditedGas\",\"inputs\":[{\"name\":\"commitmentHash\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"}],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"getCurrentTurnRelayer\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"address\",\"internalType\":\"address\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"getRelayerCount\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"getRelayers\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"address[]\",\"internalType\":\"address[]\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"getTip\",\"inputs\":[{\"name\":\"beefyBlockNumber\",\"type\":\"uint32\",\"internalType\":\"uint32\"}],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"gracePeriodBlocks\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"implementation\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"address\",\"internalType\":\"address\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"initialize\",\"inputs\":[{\"name\":\"data\",\"type\":\"bytes\",\"internalType\":\"bytes\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"isGracePeriodActive\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"internalType\":\"bool\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"isRelayer\",\"inputs\":[{\"name\":\"\",\"type\":\"address\",\"internalType\":\"address\"}],\"outputs\":[{\"name\":\"\",\"type\":\"bool\",\"internalType\":\"bool\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"lastSubmissionBlock\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"latestBeefyBlock\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"uint64\",\"internalType\":\"uint64\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"maxGasPrice\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"maxRefundAmount\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"minBlockIncrement\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"nextValidatorSet\",\"inputs\":[],\"outputs\":[{\"name\":\"id\",\"type\":\"uint128\",\"internalType\":\"uint128\"},{\"name\":\"length\",\"type\":\"uint128\",\"internalType\":\"uint128\"},{\"name\":\"root\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"owner\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"address\",\"internalType\":\"address\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"randaoCommitDelay\",\"inputs\":[],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"relayers\",\"inputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[{\"name\":\"\",\"type\":\"address\",\"internalType\":\"address\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"removeRelayer\",\"inputs\":[{\"name\":\"relayer\",\"type\":\"address\",\"internalType\":\"address\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"setGracePeriod\",\"inputs\":[{\"name\":\"_gracePeriodBlocks\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"setMaxGasPrice\",\"inputs\":[{\"name\":\"_maxGasPrice\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"setMaxRefundAmount\",\"inputs\":[{\"name\":\"_maxRefundAmount\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"setMinBlockIncrement\",\"inputs\":[{\"name\":\"_minBlockIncrement\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"submitFinal\",\"inputs\":[{\"name\":\"commitment\",\"type\":\"tuple\",\"internalType\":\"structIBeefyClient.Commitment\",\"components\":[{\"name\":\"blockNumber\",\"type\":\"uint32\",\"internalType\":\"uint32\"},{\"name\":\"validatorSetID\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"payload\",\"type\":\"tuple[]\",\"internalType\":\"structIBeefyClient.PayloadItem[]\",\"components\":[{\"name\":\"payloadID\",\"type\":\"bytes2\",\"internalType\":\"bytes2\"},{\"name\":\"data\",\"type\":\"bytes\",\"internalType\":\"bytes\"}]}]},{\"name\":\"bitfield\",\"type\":\"uint256[]\",\"internalType\":\"uint256[]\"},{\"name\":\"proofs\",\"type\":\"tuple[]\",\"internalType\":\"structIBeefyClient.ValidatorProof[]\",\"components\":[{\"name\":\"v\",\"type\":\"uint8\",\"internalType\":\"uint8\"},{\"name\":\"r\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"},{\"name\":\"s\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"},{\"name\":\"index\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"account\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"proof\",\"type\":\"bytes32[]\",\"internalType\":\"bytes32[]\"}]},{\"name\":\"leaf\",\"type\":\"tuple\",\"internalType\":\"structIBeefyClient.MMRLeaf\",\"components\":[{\"name\":\"version\",\"type\":\"uint8\",\"internalType\":\"uint8\"},{\"name\":\"parentNumber\",\"type\":\"uint32\",\"internalType\":\"uint32\"},{\"name\":\"parentHash\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"},{\"name\":\"nextAuthoritySetID\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"nextAuthoritySetLen\",\"type\":\"uint32\",\"internalType\":\"uint32\"},{\"name\":\"nextAuthoritySetRoot\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"},{\"name\":\"parachainHeadsRoot\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"}]},{\"name\":\"leafProof\",\"type\":\"bytes32[]\",\"internalType\":\"bytes32[]\"},{\"name\":\"leafProofOrder\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"claimTipBlocks\",\"type\":\"uint32[]\",\"internalType\":\"uint32[]\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"submitInitial\",\"inputs\":[{\"name\":\"commitment\",\"type\":\"tuple\",\"internalType\":\"structIBeefyClient.Commitment\",\"components\":[{\"name\":\"blockNumber\",\"type\":\"uint32\",\"internalType\":\"uint32\"},{\"name\":\"validatorSetID\",\"type\":\"uint64\",\"internalType\":\"uint64\"},{\"name\":\"payload\",\"type\":\"tuple[]\",\"internalType\":\"structIBeefyClient.PayloadItem[]\",\"components\":[{\"name\":\"payloadID\",\"type\":\"bytes2\",\"internalType\":\"bytes2\"},{\"name\":\"data\",\"type\":\"bytes\",\"internalType\":\"bytes\"}]}]},{\"name\":\"bitfield\",\"type\":\"uint256[]\",\"internalType\":\"uint256[]\"},{\"name\":\"proof\",\"type\":\"tuple\",\"internalType\":\"structIBeefyClient.ValidatorProof\",\"components\":[{\"name\":\"v\",\"type\":\"uint8\",\"internalType\":\"uint8\"},{\"name\":\"r\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"},{\"name\":\"s\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"},{\"name\":\"index\",\"type\":\"uint256\",\"internalType\":\"uint256\"},{\"name\":\"account\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"proof\",\"type\":\"bytes32[]\",\"internalType\":\"bytes32[]\"}]}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"ticketOwner\",\"inputs\":[{\"name\":\"\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"}],\"outputs\":[{\"name\":\"\",\"type\":\"address\",\"internalType\":\"address\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"tips\",\"inputs\":[{\"name\":\"\",\"type\":\"uint32\",\"internalType\":\"uint32\"}],\"outputs\":[{\"name\":\"\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"transferOwnership\",\"inputs\":[{\"name\":\"newOwner\",\"type\":\"address\",\"internalType\":\"address\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"upgradeTo\",\"inputs\":[{\"name\":\"newImplementation\",\"type\":\"address\",\"internalType\":\"address\"},{\"name\":\"expectedCodeHash\",\"type\":\"bytes32\",\"internalType\":\"bytes32\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"withdrawFunds\",\"inputs\":[{\"name\":\"recipient\",\"type\":\"address\",\"internalType\":\"addresspayable\"},{\"name\":\"amount\",\"type\":\"uint256\",\"internalType\":\"uint256\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"event\",\"name\":\"FundsDeposited\",\"inputs\":[{\"name\":\"depositor\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"amount\",\"type\":\"uint256\",\"indexed\":false,\"internalType\":\"uint256\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"FundsWithdrawn\",\"inputs\":[{\"name\":\"recipient\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"amount\",\"type\":\"uint256\",\"indexed\":false,\"internalType\":\"uint256\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"GasCredited\",\"inputs\":[{\"name\":\"relayer\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"commitmentHash\",\"type\":\"bytes32\",\"indexed\":true,\"internalType\":\"bytes32\"},{\"name\":\"gasUsed\",\"type\":\"uint256\",\"indexed\":false,\"internalType\":\"uint256\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"RelayerAdded\",\"inputs\":[{\"name\":\"relayer\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"RelayerRemoved\",\"inputs\":[{\"name\":\"relayer\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"SubmissionRefunded\",\"inputs\":[{\"name\":\"relayer\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"amount\",\"type\":\"uint256\",\"indexed\":false,\"internalType\":\"uint256\"},{\"name\":\"totalGasUsed\",\"type\":\"uint256\",\"indexed\":false,\"internalType\":\"uint256\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"TipAdded\",\"inputs\":[{\"name\":\"tipper\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"beefyBlockNumber\",\"type\":\"uint32\",\"indexed\":true,\"internalType\":\"uint32\"},{\"name\":\"amount\",\"type\":\"uint256\",\"indexed\":false,\"internalType\":\"uint256\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"TipsClaimed\",\"inputs\":[{\"name\":\"relayer\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"},{\"name\":\"totalAmount\",\"type\":\"uint256\",\"indexed\":false,\"internalType\":\"uint256\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"TurnAdvanced\",\"inputs\":[{\"name\":\"newTurnIndex\",\"type\":\"uint256\",\"indexed\":true,\"internalType\":\"uint256\"},{\"name\":\"nextRelayer\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"}],\"anonymous\":false},{\"type\":\"event\",\"name\":\"Upgraded\",\"inputs\":[{\"name\":\"implementation\",\"type\":\"address\",\"indexed\":true,\"internalType\":\"address\"}],\"anonymous\":false},{\"type\":\"error\",\"name\":\"AlreadyInitialized\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"InsufficientBlockIncrement\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"InvalidAddress\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"InvalidCodeHash\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"InvalidContract\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"InvalidTicket\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"NoRelayers\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"NotARelayer\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"NotTicketOwner\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"NotYourTurn\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"RelayerAlreadyExists\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"RelayerNotFound\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"TicketAlreadyActive\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"TransferFailed\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"Unauthorized\",\"inputs\":[]},{\"type\":\"error\",\"name\":\"UnsupportedCompactEncoding\",\"inputs\":[]}]",
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

// ActiveTicket is a free data retrieval call binding the contract method 0x3b38c624.
//
// Solidity: function activeTicket(address ) view returns(bytes32)
func (_BeefyClientWrapper *BeefyClientWrapperCaller) ActiveTicket(opts *bind.CallOpts, arg0 common.Address) ([32]byte, error) {
	var out []interface{}
	err := _BeefyClientWrapper.contract.Call(opts, &out, "activeTicket", arg0)

	if err != nil {
		return *new([32]byte), err
	}

	out0 := *abi.ConvertType(out[0], new([32]byte)).(*[32]byte)

	return out0, err

}

// ActiveTicket is a free data retrieval call binding the contract method 0x3b38c624.
//
// Solidity: function activeTicket(address ) view returns(bytes32)
func (_BeefyClientWrapper *BeefyClientWrapperSession) ActiveTicket(arg0 common.Address) ([32]byte, error) {
	return _BeefyClientWrapper.Contract.ActiveTicket(&_BeefyClientWrapper.CallOpts, arg0)
}

// ActiveTicket is a free data retrieval call binding the contract method 0x3b38c624.
//
// Solidity: function activeTicket(address ) view returns(bytes32)
func (_BeefyClientWrapper *BeefyClientWrapperCallerSession) ActiveTicket(arg0 common.Address) ([32]byte, error) {
	return _BeefyClientWrapper.Contract.ActiveTicket(&_BeefyClientWrapper.CallOpts, arg0)
}

// BeefyClient is a free data retrieval call binding the contract method 0x776c81c3.
//
// Solidity: function beefyClient() view returns(address)
func (_BeefyClientWrapper *BeefyClientWrapperCaller) BeefyClient(opts *bind.CallOpts) (common.Address, error) {
	var out []interface{}
	err := _BeefyClientWrapper.contract.Call(opts, &out, "beefyClient")

	if err != nil {
		return *new(common.Address), err
	}

	out0 := *abi.ConvertType(out[0], new(common.Address)).(*common.Address)

	return out0, err

}

// BeefyClient is a free data retrieval call binding the contract method 0x776c81c3.
//
// Solidity: function beefyClient() view returns(address)
func (_BeefyClientWrapper *BeefyClientWrapperSession) BeefyClient() (common.Address, error) {
	return _BeefyClientWrapper.Contract.BeefyClient(&_BeefyClientWrapper.CallOpts)
}

// BeefyClient is a free data retrieval call binding the contract method 0x776c81c3.
//
// Solidity: function beefyClient() view returns(address)
func (_BeefyClientWrapper *BeefyClientWrapperCallerSession) BeefyClient() (common.Address, error) {
	return _BeefyClientWrapper.Contract.BeefyClient(&_BeefyClientWrapper.CallOpts)
}

// CreateFinalBitfield is a free data retrieval call binding the contract method 0x8ab81d13.
//
// Solidity: function createFinalBitfield(bytes32 commitmentHash, uint256[] bitfield) view returns(uint256[])
func (_BeefyClientWrapper *BeefyClientWrapperCaller) CreateFinalBitfield(opts *bind.CallOpts, commitmentHash [32]byte, bitfield []*big.Int) ([]*big.Int, error) {
	var out []interface{}
	err := _BeefyClientWrapper.contract.Call(opts, &out, "createFinalBitfield", commitmentHash, bitfield)

	if err != nil {
		return *new([]*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new([]*big.Int)).(*[]*big.Int)

	return out0, err

}

// CreateFinalBitfield is a free data retrieval call binding the contract method 0x8ab81d13.
//
// Solidity: function createFinalBitfield(bytes32 commitmentHash, uint256[] bitfield) view returns(uint256[])
func (_BeefyClientWrapper *BeefyClientWrapperSession) CreateFinalBitfield(commitmentHash [32]byte, bitfield []*big.Int) ([]*big.Int, error) {
	return _BeefyClientWrapper.Contract.CreateFinalBitfield(&_BeefyClientWrapper.CallOpts, commitmentHash, bitfield)
}

// CreateFinalBitfield is a free data retrieval call binding the contract method 0x8ab81d13.
//
// Solidity: function createFinalBitfield(bytes32 commitmentHash, uint256[] bitfield) view returns(uint256[])
func (_BeefyClientWrapper *BeefyClientWrapperCallerSession) CreateFinalBitfield(commitmentHash [32]byte, bitfield []*big.Int) ([]*big.Int, error) {
	return _BeefyClientWrapper.Contract.CreateFinalBitfield(&_BeefyClientWrapper.CallOpts, commitmentHash, bitfield)
}

// CreateInitialBitfield is a free data retrieval call binding the contract method 0x5da57fe9.
//
// Solidity: function createInitialBitfield(uint256[] bitsToSet, uint256 length) view returns(uint256[])
func (_BeefyClientWrapper *BeefyClientWrapperCaller) CreateInitialBitfield(opts *bind.CallOpts, bitsToSet []*big.Int, length *big.Int) ([]*big.Int, error) {
	var out []interface{}
	err := _BeefyClientWrapper.contract.Call(opts, &out, "createInitialBitfield", bitsToSet, length)

	if err != nil {
		return *new([]*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new([]*big.Int)).(*[]*big.Int)

	return out0, err

}

// CreateInitialBitfield is a free data retrieval call binding the contract method 0x5da57fe9.
//
// Solidity: function createInitialBitfield(uint256[] bitsToSet, uint256 length) view returns(uint256[])
func (_BeefyClientWrapper *BeefyClientWrapperSession) CreateInitialBitfield(bitsToSet []*big.Int, length *big.Int) ([]*big.Int, error) {
	return _BeefyClientWrapper.Contract.CreateInitialBitfield(&_BeefyClientWrapper.CallOpts, bitsToSet, length)
}

// CreateInitialBitfield is a free data retrieval call binding the contract method 0x5da57fe9.
//
// Solidity: function createInitialBitfield(uint256[] bitsToSet, uint256 length) view returns(uint256[])
func (_BeefyClientWrapper *BeefyClientWrapperCallerSession) CreateInitialBitfield(bitsToSet []*big.Int, length *big.Int) ([]*big.Int, error) {
	return _BeefyClientWrapper.Contract.CreateInitialBitfield(&_BeefyClientWrapper.CallOpts, bitsToSet, length)
}

// CreditedGas is a free data retrieval call binding the contract method 0x660b2928.
//
// Solidity: function creditedGas(bytes32 ) view returns(uint256)
func (_BeefyClientWrapper *BeefyClientWrapperCaller) CreditedGas(opts *bind.CallOpts, arg0 [32]byte) (*big.Int, error) {
	var out []interface{}
	err := _BeefyClientWrapper.contract.Call(opts, &out, "creditedGas", arg0)

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// CreditedGas is a free data retrieval call binding the contract method 0x660b2928.
//
// Solidity: function creditedGas(bytes32 ) view returns(uint256)
func (_BeefyClientWrapper *BeefyClientWrapperSession) CreditedGas(arg0 [32]byte) (*big.Int, error) {
	return _BeefyClientWrapper.Contract.CreditedGas(&_BeefyClientWrapper.CallOpts, arg0)
}

// CreditedGas is a free data retrieval call binding the contract method 0x660b2928.
//
// Solidity: function creditedGas(bytes32 ) view returns(uint256)
func (_BeefyClientWrapper *BeefyClientWrapperCallerSession) CreditedGas(arg0 [32]byte) (*big.Int, error) {
	return _BeefyClientWrapper.Contract.CreditedGas(&_BeefyClientWrapper.CallOpts, arg0)
}

// CurrentTurnIndex is a free data retrieval call binding the contract method 0x50efd268.
//
// Solidity: function currentTurnIndex() view returns(uint256)
func (_BeefyClientWrapper *BeefyClientWrapperCaller) CurrentTurnIndex(opts *bind.CallOpts) (*big.Int, error) {
	var out []interface{}
	err := _BeefyClientWrapper.contract.Call(opts, &out, "currentTurnIndex")

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// CurrentTurnIndex is a free data retrieval call binding the contract method 0x50efd268.
//
// Solidity: function currentTurnIndex() view returns(uint256)
func (_BeefyClientWrapper *BeefyClientWrapperSession) CurrentTurnIndex() (*big.Int, error) {
	return _BeefyClientWrapper.Contract.CurrentTurnIndex(&_BeefyClientWrapper.CallOpts)
}

// CurrentTurnIndex is a free data retrieval call binding the contract method 0x50efd268.
//
// Solidity: function currentTurnIndex() view returns(uint256)
func (_BeefyClientWrapper *BeefyClientWrapperCallerSession) CurrentTurnIndex() (*big.Int, error) {
	return _BeefyClientWrapper.Contract.CurrentTurnIndex(&_BeefyClientWrapper.CallOpts)
}

// CurrentValidatorSet is a free data retrieval call binding the contract method 0x2cdea717.
//
// Solidity: function currentValidatorSet() view returns(uint128 id, uint128 length, bytes32 root)
func (_BeefyClientWrapper *BeefyClientWrapperCaller) CurrentValidatorSet(opts *bind.CallOpts) (struct {
	Id     *big.Int
	Length *big.Int
	Root   [32]byte
}, error) {
	var out []interface{}
	err := _BeefyClientWrapper.contract.Call(opts, &out, "currentValidatorSet")

	outstruct := new(struct {
		Id     *big.Int
		Length *big.Int
		Root   [32]byte
	})
	if err != nil {
		return *outstruct, err
	}

	outstruct.Id = *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)
	outstruct.Length = *abi.ConvertType(out[1], new(*big.Int)).(**big.Int)
	outstruct.Root = *abi.ConvertType(out[2], new([32]byte)).(*[32]byte)

	return *outstruct, err

}

// CurrentValidatorSet is a free data retrieval call binding the contract method 0x2cdea717.
//
// Solidity: function currentValidatorSet() view returns(uint128 id, uint128 length, bytes32 root)
func (_BeefyClientWrapper *BeefyClientWrapperSession) CurrentValidatorSet() (struct {
	Id     *big.Int
	Length *big.Int
	Root   [32]byte
}, error) {
	return _BeefyClientWrapper.Contract.CurrentValidatorSet(&_BeefyClientWrapper.CallOpts)
}

// CurrentValidatorSet is a free data retrieval call binding the contract method 0x2cdea717.
//
// Solidity: function currentValidatorSet() view returns(uint128 id, uint128 length, bytes32 root)
func (_BeefyClientWrapper *BeefyClientWrapperCallerSession) CurrentValidatorSet() (struct {
	Id     *big.Int
	Length *big.Int
	Root   [32]byte
}, error) {
	return _BeefyClientWrapper.Contract.CurrentValidatorSet(&_BeefyClientWrapper.CallOpts)
}

// GetBalance is a free data retrieval call binding the contract method 0x12065fe0.
//
// Solidity: function getBalance() view returns(uint256)
func (_BeefyClientWrapper *BeefyClientWrapperCaller) GetBalance(opts *bind.CallOpts) (*big.Int, error) {
	var out []interface{}
	err := _BeefyClientWrapper.contract.Call(opts, &out, "getBalance")

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// GetBalance is a free data retrieval call binding the contract method 0x12065fe0.
//
// Solidity: function getBalance() view returns(uint256)
func (_BeefyClientWrapper *BeefyClientWrapperSession) GetBalance() (*big.Int, error) {
	return _BeefyClientWrapper.Contract.GetBalance(&_BeefyClientWrapper.CallOpts)
}

// GetBalance is a free data retrieval call binding the contract method 0x12065fe0.
//
// Solidity: function getBalance() view returns(uint256)
func (_BeefyClientWrapper *BeefyClientWrapperCallerSession) GetBalance() (*big.Int, error) {
	return _BeefyClientWrapper.Contract.GetBalance(&_BeefyClientWrapper.CallOpts)
}

// GetCreditedGas is a free data retrieval call binding the contract method 0xfd635f1c.
//
// Solidity: function getCreditedGas(bytes32 commitmentHash) view returns(uint256)
func (_BeefyClientWrapper *BeefyClientWrapperCaller) GetCreditedGas(opts *bind.CallOpts, commitmentHash [32]byte) (*big.Int, error) {
	var out []interface{}
	err := _BeefyClientWrapper.contract.Call(opts, &out, "getCreditedGas", commitmentHash)

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// GetCreditedGas is a free data retrieval call binding the contract method 0xfd635f1c.
//
// Solidity: function getCreditedGas(bytes32 commitmentHash) view returns(uint256)
func (_BeefyClientWrapper *BeefyClientWrapperSession) GetCreditedGas(commitmentHash [32]byte) (*big.Int, error) {
	return _BeefyClientWrapper.Contract.GetCreditedGas(&_BeefyClientWrapper.CallOpts, commitmentHash)
}

// GetCreditedGas is a free data retrieval call binding the contract method 0xfd635f1c.
//
// Solidity: function getCreditedGas(bytes32 commitmentHash) view returns(uint256)
func (_BeefyClientWrapper *BeefyClientWrapperCallerSession) GetCreditedGas(commitmentHash [32]byte) (*big.Int, error) {
	return _BeefyClientWrapper.Contract.GetCreditedGas(&_BeefyClientWrapper.CallOpts, commitmentHash)
}

// GetCurrentTurnRelayer is a free data retrieval call binding the contract method 0x5664b04f.
//
// Solidity: function getCurrentTurnRelayer() view returns(address)
func (_BeefyClientWrapper *BeefyClientWrapperCaller) GetCurrentTurnRelayer(opts *bind.CallOpts) (common.Address, error) {
	var out []interface{}
	err := _BeefyClientWrapper.contract.Call(opts, &out, "getCurrentTurnRelayer")

	if err != nil {
		return *new(common.Address), err
	}

	out0 := *abi.ConvertType(out[0], new(common.Address)).(*common.Address)

	return out0, err

}

// GetCurrentTurnRelayer is a free data retrieval call binding the contract method 0x5664b04f.
//
// Solidity: function getCurrentTurnRelayer() view returns(address)
func (_BeefyClientWrapper *BeefyClientWrapperSession) GetCurrentTurnRelayer() (common.Address, error) {
	return _BeefyClientWrapper.Contract.GetCurrentTurnRelayer(&_BeefyClientWrapper.CallOpts)
}

// GetCurrentTurnRelayer is a free data retrieval call binding the contract method 0x5664b04f.
//
// Solidity: function getCurrentTurnRelayer() view returns(address)
func (_BeefyClientWrapper *BeefyClientWrapperCallerSession) GetCurrentTurnRelayer() (common.Address, error) {
	return _BeefyClientWrapper.Contract.GetCurrentTurnRelayer(&_BeefyClientWrapper.CallOpts)
}

// GetRelayerCount is a free data retrieval call binding the contract method 0xe51ea0b5.
//
// Solidity: function getRelayerCount() view returns(uint256)
func (_BeefyClientWrapper *BeefyClientWrapperCaller) GetRelayerCount(opts *bind.CallOpts) (*big.Int, error) {
	var out []interface{}
	err := _BeefyClientWrapper.contract.Call(opts, &out, "getRelayerCount")

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// GetRelayerCount is a free data retrieval call binding the contract method 0xe51ea0b5.
//
// Solidity: function getRelayerCount() view returns(uint256)
func (_BeefyClientWrapper *BeefyClientWrapperSession) GetRelayerCount() (*big.Int, error) {
	return _BeefyClientWrapper.Contract.GetRelayerCount(&_BeefyClientWrapper.CallOpts)
}

// GetRelayerCount is a free data retrieval call binding the contract method 0xe51ea0b5.
//
// Solidity: function getRelayerCount() view returns(uint256)
func (_BeefyClientWrapper *BeefyClientWrapperCallerSession) GetRelayerCount() (*big.Int, error) {
	return _BeefyClientWrapper.Contract.GetRelayerCount(&_BeefyClientWrapper.CallOpts)
}

// GetRelayers is a free data retrieval call binding the contract method 0x179ff4b2.
//
// Solidity: function getRelayers() view returns(address[])
func (_BeefyClientWrapper *BeefyClientWrapperCaller) GetRelayers(opts *bind.CallOpts) ([]common.Address, error) {
	var out []interface{}
	err := _BeefyClientWrapper.contract.Call(opts, &out, "getRelayers")

	if err != nil {
		return *new([]common.Address), err
	}

	out0 := *abi.ConvertType(out[0], new([]common.Address)).(*[]common.Address)

	return out0, err

}

// GetRelayers is a free data retrieval call binding the contract method 0x179ff4b2.
//
// Solidity: function getRelayers() view returns(address[])
func (_BeefyClientWrapper *BeefyClientWrapperSession) GetRelayers() ([]common.Address, error) {
	return _BeefyClientWrapper.Contract.GetRelayers(&_BeefyClientWrapper.CallOpts)
}

// GetRelayers is a free data retrieval call binding the contract method 0x179ff4b2.
//
// Solidity: function getRelayers() view returns(address[])
func (_BeefyClientWrapper *BeefyClientWrapperCallerSession) GetRelayers() ([]common.Address, error) {
	return _BeefyClientWrapper.Contract.GetRelayers(&_BeefyClientWrapper.CallOpts)
}

// GetTip is a free data retrieval call binding the contract method 0xa7e7353c.
//
// Solidity: function getTip(uint32 beefyBlockNumber) view returns(uint256)
func (_BeefyClientWrapper *BeefyClientWrapperCaller) GetTip(opts *bind.CallOpts, beefyBlockNumber uint32) (*big.Int, error) {
	var out []interface{}
	err := _BeefyClientWrapper.contract.Call(opts, &out, "getTip", beefyBlockNumber)

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// GetTip is a free data retrieval call binding the contract method 0xa7e7353c.
//
// Solidity: function getTip(uint32 beefyBlockNumber) view returns(uint256)
func (_BeefyClientWrapper *BeefyClientWrapperSession) GetTip(beefyBlockNumber uint32) (*big.Int, error) {
	return _BeefyClientWrapper.Contract.GetTip(&_BeefyClientWrapper.CallOpts, beefyBlockNumber)
}

// GetTip is a free data retrieval call binding the contract method 0xa7e7353c.
//
// Solidity: function getTip(uint32 beefyBlockNumber) view returns(uint256)
func (_BeefyClientWrapper *BeefyClientWrapperCallerSession) GetTip(beefyBlockNumber uint32) (*big.Int, error) {
	return _BeefyClientWrapper.Contract.GetTip(&_BeefyClientWrapper.CallOpts, beefyBlockNumber)
}

// GracePeriodBlocks is a free data retrieval call binding the contract method 0xde2daaff.
//
// Solidity: function gracePeriodBlocks() view returns(uint256)
func (_BeefyClientWrapper *BeefyClientWrapperCaller) GracePeriodBlocks(opts *bind.CallOpts) (*big.Int, error) {
	var out []interface{}
	err := _BeefyClientWrapper.contract.Call(opts, &out, "gracePeriodBlocks")

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// GracePeriodBlocks is a free data retrieval call binding the contract method 0xde2daaff.
//
// Solidity: function gracePeriodBlocks() view returns(uint256)
func (_BeefyClientWrapper *BeefyClientWrapperSession) GracePeriodBlocks() (*big.Int, error) {
	return _BeefyClientWrapper.Contract.GracePeriodBlocks(&_BeefyClientWrapper.CallOpts)
}

// GracePeriodBlocks is a free data retrieval call binding the contract method 0xde2daaff.
//
// Solidity: function gracePeriodBlocks() view returns(uint256)
func (_BeefyClientWrapper *BeefyClientWrapperCallerSession) GracePeriodBlocks() (*big.Int, error) {
	return _BeefyClientWrapper.Contract.GracePeriodBlocks(&_BeefyClientWrapper.CallOpts)
}

// Implementation is a free data retrieval call binding the contract method 0x5c60da1b.
//
// Solidity: function implementation() view returns(address)
func (_BeefyClientWrapper *BeefyClientWrapperCaller) Implementation(opts *bind.CallOpts) (common.Address, error) {
	var out []interface{}
	err := _BeefyClientWrapper.contract.Call(opts, &out, "implementation")

	if err != nil {
		return *new(common.Address), err
	}

	out0 := *abi.ConvertType(out[0], new(common.Address)).(*common.Address)

	return out0, err

}

// Implementation is a free data retrieval call binding the contract method 0x5c60da1b.
//
// Solidity: function implementation() view returns(address)
func (_BeefyClientWrapper *BeefyClientWrapperSession) Implementation() (common.Address, error) {
	return _BeefyClientWrapper.Contract.Implementation(&_BeefyClientWrapper.CallOpts)
}

// Implementation is a free data retrieval call binding the contract method 0x5c60da1b.
//
// Solidity: function implementation() view returns(address)
func (_BeefyClientWrapper *BeefyClientWrapperCallerSession) Implementation() (common.Address, error) {
	return _BeefyClientWrapper.Contract.Implementation(&_BeefyClientWrapper.CallOpts)
}

// IsGracePeriodActive is a free data retrieval call binding the contract method 0xb965655a.
//
// Solidity: function isGracePeriodActive() view returns(bool)
func (_BeefyClientWrapper *BeefyClientWrapperCaller) IsGracePeriodActive(opts *bind.CallOpts) (bool, error) {
	var out []interface{}
	err := _BeefyClientWrapper.contract.Call(opts, &out, "isGracePeriodActive")

	if err != nil {
		return *new(bool), err
	}

	out0 := *abi.ConvertType(out[0], new(bool)).(*bool)

	return out0, err

}

// IsGracePeriodActive is a free data retrieval call binding the contract method 0xb965655a.
//
// Solidity: function isGracePeriodActive() view returns(bool)
func (_BeefyClientWrapper *BeefyClientWrapperSession) IsGracePeriodActive() (bool, error) {
	return _BeefyClientWrapper.Contract.IsGracePeriodActive(&_BeefyClientWrapper.CallOpts)
}

// IsGracePeriodActive is a free data retrieval call binding the contract method 0xb965655a.
//
// Solidity: function isGracePeriodActive() view returns(bool)
func (_BeefyClientWrapper *BeefyClientWrapperCallerSession) IsGracePeriodActive() (bool, error) {
	return _BeefyClientWrapper.Contract.IsGracePeriodActive(&_BeefyClientWrapper.CallOpts)
}

// IsRelayer is a free data retrieval call binding the contract method 0x541d5548.
//
// Solidity: function isRelayer(address ) view returns(bool)
func (_BeefyClientWrapper *BeefyClientWrapperCaller) IsRelayer(opts *bind.CallOpts, arg0 common.Address) (bool, error) {
	var out []interface{}
	err := _BeefyClientWrapper.contract.Call(opts, &out, "isRelayer", arg0)

	if err != nil {
		return *new(bool), err
	}

	out0 := *abi.ConvertType(out[0], new(bool)).(*bool)

	return out0, err

}

// IsRelayer is a free data retrieval call binding the contract method 0x541d5548.
//
// Solidity: function isRelayer(address ) view returns(bool)
func (_BeefyClientWrapper *BeefyClientWrapperSession) IsRelayer(arg0 common.Address) (bool, error) {
	return _BeefyClientWrapper.Contract.IsRelayer(&_BeefyClientWrapper.CallOpts, arg0)
}

// IsRelayer is a free data retrieval call binding the contract method 0x541d5548.
//
// Solidity: function isRelayer(address ) view returns(bool)
func (_BeefyClientWrapper *BeefyClientWrapperCallerSession) IsRelayer(arg0 common.Address) (bool, error) {
	return _BeefyClientWrapper.Contract.IsRelayer(&_BeefyClientWrapper.CallOpts, arg0)
}

// LastSubmissionBlock is a free data retrieval call binding the contract method 0x1f498355.
//
// Solidity: function lastSubmissionBlock() view returns(uint256)
func (_BeefyClientWrapper *BeefyClientWrapperCaller) LastSubmissionBlock(opts *bind.CallOpts) (*big.Int, error) {
	var out []interface{}
	err := _BeefyClientWrapper.contract.Call(opts, &out, "lastSubmissionBlock")

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// LastSubmissionBlock is a free data retrieval call binding the contract method 0x1f498355.
//
// Solidity: function lastSubmissionBlock() view returns(uint256)
func (_BeefyClientWrapper *BeefyClientWrapperSession) LastSubmissionBlock() (*big.Int, error) {
	return _BeefyClientWrapper.Contract.LastSubmissionBlock(&_BeefyClientWrapper.CallOpts)
}

// LastSubmissionBlock is a free data retrieval call binding the contract method 0x1f498355.
//
// Solidity: function lastSubmissionBlock() view returns(uint256)
func (_BeefyClientWrapper *BeefyClientWrapperCallerSession) LastSubmissionBlock() (*big.Int, error) {
	return _BeefyClientWrapper.Contract.LastSubmissionBlock(&_BeefyClientWrapper.CallOpts)
}

// LatestBeefyBlock is a free data retrieval call binding the contract method 0x66ae69a0.
//
// Solidity: function latestBeefyBlock() view returns(uint64)
func (_BeefyClientWrapper *BeefyClientWrapperCaller) LatestBeefyBlock(opts *bind.CallOpts) (uint64, error) {
	var out []interface{}
	err := _BeefyClientWrapper.contract.Call(opts, &out, "latestBeefyBlock")

	if err != nil {
		return *new(uint64), err
	}

	out0 := *abi.ConvertType(out[0], new(uint64)).(*uint64)

	return out0, err

}

// LatestBeefyBlock is a free data retrieval call binding the contract method 0x66ae69a0.
//
// Solidity: function latestBeefyBlock() view returns(uint64)
func (_BeefyClientWrapper *BeefyClientWrapperSession) LatestBeefyBlock() (uint64, error) {
	return _BeefyClientWrapper.Contract.LatestBeefyBlock(&_BeefyClientWrapper.CallOpts)
}

// LatestBeefyBlock is a free data retrieval call binding the contract method 0x66ae69a0.
//
// Solidity: function latestBeefyBlock() view returns(uint64)
func (_BeefyClientWrapper *BeefyClientWrapperCallerSession) LatestBeefyBlock() (uint64, error) {
	return _BeefyClientWrapper.Contract.LatestBeefyBlock(&_BeefyClientWrapper.CallOpts)
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

// MinBlockIncrement is a free data retrieval call binding the contract method 0x4bfcad80.
//
// Solidity: function minBlockIncrement() view returns(uint256)
func (_BeefyClientWrapper *BeefyClientWrapperCaller) MinBlockIncrement(opts *bind.CallOpts) (*big.Int, error) {
	var out []interface{}
	err := _BeefyClientWrapper.contract.Call(opts, &out, "minBlockIncrement")

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// MinBlockIncrement is a free data retrieval call binding the contract method 0x4bfcad80.
//
// Solidity: function minBlockIncrement() view returns(uint256)
func (_BeefyClientWrapper *BeefyClientWrapperSession) MinBlockIncrement() (*big.Int, error) {
	return _BeefyClientWrapper.Contract.MinBlockIncrement(&_BeefyClientWrapper.CallOpts)
}

// MinBlockIncrement is a free data retrieval call binding the contract method 0x4bfcad80.
//
// Solidity: function minBlockIncrement() view returns(uint256)
func (_BeefyClientWrapper *BeefyClientWrapperCallerSession) MinBlockIncrement() (*big.Int, error) {
	return _BeefyClientWrapper.Contract.MinBlockIncrement(&_BeefyClientWrapper.CallOpts)
}

// NextValidatorSet is a free data retrieval call binding the contract method 0x36667513.
//
// Solidity: function nextValidatorSet() view returns(uint128 id, uint128 length, bytes32 root)
func (_BeefyClientWrapper *BeefyClientWrapperCaller) NextValidatorSet(opts *bind.CallOpts) (struct {
	Id     *big.Int
	Length *big.Int
	Root   [32]byte
}, error) {
	var out []interface{}
	err := _BeefyClientWrapper.contract.Call(opts, &out, "nextValidatorSet")

	outstruct := new(struct {
		Id     *big.Int
		Length *big.Int
		Root   [32]byte
	})
	if err != nil {
		return *outstruct, err
	}

	outstruct.Id = *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)
	outstruct.Length = *abi.ConvertType(out[1], new(*big.Int)).(**big.Int)
	outstruct.Root = *abi.ConvertType(out[2], new([32]byte)).(*[32]byte)

	return *outstruct, err

}

// NextValidatorSet is a free data retrieval call binding the contract method 0x36667513.
//
// Solidity: function nextValidatorSet() view returns(uint128 id, uint128 length, bytes32 root)
func (_BeefyClientWrapper *BeefyClientWrapperSession) NextValidatorSet() (struct {
	Id     *big.Int
	Length *big.Int
	Root   [32]byte
}, error) {
	return _BeefyClientWrapper.Contract.NextValidatorSet(&_BeefyClientWrapper.CallOpts)
}

// NextValidatorSet is a free data retrieval call binding the contract method 0x36667513.
//
// Solidity: function nextValidatorSet() view returns(uint128 id, uint128 length, bytes32 root)
func (_BeefyClientWrapper *BeefyClientWrapperCallerSession) NextValidatorSet() (struct {
	Id     *big.Int
	Length *big.Int
	Root   [32]byte
}, error) {
	return _BeefyClientWrapper.Contract.NextValidatorSet(&_BeefyClientWrapper.CallOpts)
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

// RandaoCommitDelay is a free data retrieval call binding the contract method 0x591d99ee.
//
// Solidity: function randaoCommitDelay() view returns(uint256)
func (_BeefyClientWrapper *BeefyClientWrapperCaller) RandaoCommitDelay(opts *bind.CallOpts) (*big.Int, error) {
	var out []interface{}
	err := _BeefyClientWrapper.contract.Call(opts, &out, "randaoCommitDelay")

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// RandaoCommitDelay is a free data retrieval call binding the contract method 0x591d99ee.
//
// Solidity: function randaoCommitDelay() view returns(uint256)
func (_BeefyClientWrapper *BeefyClientWrapperSession) RandaoCommitDelay() (*big.Int, error) {
	return _BeefyClientWrapper.Contract.RandaoCommitDelay(&_BeefyClientWrapper.CallOpts)
}

// RandaoCommitDelay is a free data retrieval call binding the contract method 0x591d99ee.
//
// Solidity: function randaoCommitDelay() view returns(uint256)
func (_BeefyClientWrapper *BeefyClientWrapperCallerSession) RandaoCommitDelay() (*big.Int, error) {
	return _BeefyClientWrapper.Contract.RandaoCommitDelay(&_BeefyClientWrapper.CallOpts)
}

// Relayers is a free data retrieval call binding the contract method 0x9a48e7f9.
//
// Solidity: function relayers(uint256 ) view returns(address)
func (_BeefyClientWrapper *BeefyClientWrapperCaller) Relayers(opts *bind.CallOpts, arg0 *big.Int) (common.Address, error) {
	var out []interface{}
	err := _BeefyClientWrapper.contract.Call(opts, &out, "relayers", arg0)

	if err != nil {
		return *new(common.Address), err
	}

	out0 := *abi.ConvertType(out[0], new(common.Address)).(*common.Address)

	return out0, err

}

// Relayers is a free data retrieval call binding the contract method 0x9a48e7f9.
//
// Solidity: function relayers(uint256 ) view returns(address)
func (_BeefyClientWrapper *BeefyClientWrapperSession) Relayers(arg0 *big.Int) (common.Address, error) {
	return _BeefyClientWrapper.Contract.Relayers(&_BeefyClientWrapper.CallOpts, arg0)
}

// Relayers is a free data retrieval call binding the contract method 0x9a48e7f9.
//
// Solidity: function relayers(uint256 ) view returns(address)
func (_BeefyClientWrapper *BeefyClientWrapperCallerSession) Relayers(arg0 *big.Int) (common.Address, error) {
	return _BeefyClientWrapper.Contract.Relayers(&_BeefyClientWrapper.CallOpts, arg0)
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

// Tips is a free data retrieval call binding the contract method 0x1756fe14.
//
// Solidity: function tips(uint32 ) view returns(uint256)
func (_BeefyClientWrapper *BeefyClientWrapperCaller) Tips(opts *bind.CallOpts, arg0 uint32) (*big.Int, error) {
	var out []interface{}
	err := _BeefyClientWrapper.contract.Call(opts, &out, "tips", arg0)

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// Tips is a free data retrieval call binding the contract method 0x1756fe14.
//
// Solidity: function tips(uint32 ) view returns(uint256)
func (_BeefyClientWrapper *BeefyClientWrapperSession) Tips(arg0 uint32) (*big.Int, error) {
	return _BeefyClientWrapper.Contract.Tips(&_BeefyClientWrapper.CallOpts, arg0)
}

// Tips is a free data retrieval call binding the contract method 0x1756fe14.
//
// Solidity: function tips(uint32 ) view returns(uint256)
func (_BeefyClientWrapper *BeefyClientWrapperCallerSession) Tips(arg0 uint32) (*big.Int, error) {
	return _BeefyClientWrapper.Contract.Tips(&_BeefyClientWrapper.CallOpts, arg0)
}

// AddRelayer is a paid mutator transaction binding the contract method 0xdd39f00d.
//
// Solidity: function addRelayer(address relayer) returns()
func (_BeefyClientWrapper *BeefyClientWrapperTransactor) AddRelayer(opts *bind.TransactOpts, relayer common.Address) (*types.Transaction, error) {
	return _BeefyClientWrapper.contract.Transact(opts, "addRelayer", relayer)
}

// AddRelayer is a paid mutator transaction binding the contract method 0xdd39f00d.
//
// Solidity: function addRelayer(address relayer) returns()
func (_BeefyClientWrapper *BeefyClientWrapperSession) AddRelayer(relayer common.Address) (*types.Transaction, error) {
	return _BeefyClientWrapper.Contract.AddRelayer(&_BeefyClientWrapper.TransactOpts, relayer)
}

// AddRelayer is a paid mutator transaction binding the contract method 0xdd39f00d.
//
// Solidity: function addRelayer(address relayer) returns()
func (_BeefyClientWrapper *BeefyClientWrapperTransactorSession) AddRelayer(relayer common.Address) (*types.Transaction, error) {
	return _BeefyClientWrapper.Contract.AddRelayer(&_BeefyClientWrapper.TransactOpts, relayer)
}

// AddTip is a paid mutator transaction binding the contract method 0x9f404590.
//
// Solidity: function addTip(uint32 beefyBlockNumber) payable returns()
func (_BeefyClientWrapper *BeefyClientWrapperTransactor) AddTip(opts *bind.TransactOpts, beefyBlockNumber uint32) (*types.Transaction, error) {
	return _BeefyClientWrapper.contract.Transact(opts, "addTip", beefyBlockNumber)
}

// AddTip is a paid mutator transaction binding the contract method 0x9f404590.
//
// Solidity: function addTip(uint32 beefyBlockNumber) payable returns()
func (_BeefyClientWrapper *BeefyClientWrapperSession) AddTip(beefyBlockNumber uint32) (*types.Transaction, error) {
	return _BeefyClientWrapper.Contract.AddTip(&_BeefyClientWrapper.TransactOpts, beefyBlockNumber)
}

// AddTip is a paid mutator transaction binding the contract method 0x9f404590.
//
// Solidity: function addTip(uint32 beefyBlockNumber) payable returns()
func (_BeefyClientWrapper *BeefyClientWrapperTransactorSession) AddTip(beefyBlockNumber uint32) (*types.Transaction, error) {
	return _BeefyClientWrapper.Contract.AddTip(&_BeefyClientWrapper.TransactOpts, beefyBlockNumber)
}

// ClearTicket is a paid mutator transaction binding the contract method 0x15738ab4.
//
// Solidity: function clearTicket() returns()
func (_BeefyClientWrapper *BeefyClientWrapperTransactor) ClearTicket(opts *bind.TransactOpts) (*types.Transaction, error) {
	return _BeefyClientWrapper.contract.Transact(opts, "clearTicket")
}

// ClearTicket is a paid mutator transaction binding the contract method 0x15738ab4.
//
// Solidity: function clearTicket() returns()
func (_BeefyClientWrapper *BeefyClientWrapperSession) ClearTicket() (*types.Transaction, error) {
	return _BeefyClientWrapper.Contract.ClearTicket(&_BeefyClientWrapper.TransactOpts)
}

// ClearTicket is a paid mutator transaction binding the contract method 0x15738ab4.
//
// Solidity: function clearTicket() returns()
func (_BeefyClientWrapper *BeefyClientWrapperTransactorSession) ClearTicket() (*types.Transaction, error) {
	return _BeefyClientWrapper.Contract.ClearTicket(&_BeefyClientWrapper.TransactOpts)
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

// Initialize is a paid mutator transaction binding the contract method 0x439fab91.
//
// Solidity: function initialize(bytes data) returns()
func (_BeefyClientWrapper *BeefyClientWrapperTransactor) Initialize(opts *bind.TransactOpts, data []byte) (*types.Transaction, error) {
	return _BeefyClientWrapper.contract.Transact(opts, "initialize", data)
}

// Initialize is a paid mutator transaction binding the contract method 0x439fab91.
//
// Solidity: function initialize(bytes data) returns()
func (_BeefyClientWrapper *BeefyClientWrapperSession) Initialize(data []byte) (*types.Transaction, error) {
	return _BeefyClientWrapper.Contract.Initialize(&_BeefyClientWrapper.TransactOpts, data)
}

// Initialize is a paid mutator transaction binding the contract method 0x439fab91.
//
// Solidity: function initialize(bytes data) returns()
func (_BeefyClientWrapper *BeefyClientWrapperTransactorSession) Initialize(data []byte) (*types.Transaction, error) {
	return _BeefyClientWrapper.Contract.Initialize(&_BeefyClientWrapper.TransactOpts, data)
}

// RemoveRelayer is a paid mutator transaction binding the contract method 0x60f0a5ac.
//
// Solidity: function removeRelayer(address relayer) returns()
func (_BeefyClientWrapper *BeefyClientWrapperTransactor) RemoveRelayer(opts *bind.TransactOpts, relayer common.Address) (*types.Transaction, error) {
	return _BeefyClientWrapper.contract.Transact(opts, "removeRelayer", relayer)
}

// RemoveRelayer is a paid mutator transaction binding the contract method 0x60f0a5ac.
//
// Solidity: function removeRelayer(address relayer) returns()
func (_BeefyClientWrapper *BeefyClientWrapperSession) RemoveRelayer(relayer common.Address) (*types.Transaction, error) {
	return _BeefyClientWrapper.Contract.RemoveRelayer(&_BeefyClientWrapper.TransactOpts, relayer)
}

// RemoveRelayer is a paid mutator transaction binding the contract method 0x60f0a5ac.
//
// Solidity: function removeRelayer(address relayer) returns()
func (_BeefyClientWrapper *BeefyClientWrapperTransactorSession) RemoveRelayer(relayer common.Address) (*types.Transaction, error) {
	return _BeefyClientWrapper.Contract.RemoveRelayer(&_BeefyClientWrapper.TransactOpts, relayer)
}

// SetGracePeriod is a paid mutator transaction binding the contract method 0xf2f65960.
//
// Solidity: function setGracePeriod(uint256 _gracePeriodBlocks) returns()
func (_BeefyClientWrapper *BeefyClientWrapperTransactor) SetGracePeriod(opts *bind.TransactOpts, _gracePeriodBlocks *big.Int) (*types.Transaction, error) {
	return _BeefyClientWrapper.contract.Transact(opts, "setGracePeriod", _gracePeriodBlocks)
}

// SetGracePeriod is a paid mutator transaction binding the contract method 0xf2f65960.
//
// Solidity: function setGracePeriod(uint256 _gracePeriodBlocks) returns()
func (_BeefyClientWrapper *BeefyClientWrapperSession) SetGracePeriod(_gracePeriodBlocks *big.Int) (*types.Transaction, error) {
	return _BeefyClientWrapper.Contract.SetGracePeriod(&_BeefyClientWrapper.TransactOpts, _gracePeriodBlocks)
}

// SetGracePeriod is a paid mutator transaction binding the contract method 0xf2f65960.
//
// Solidity: function setGracePeriod(uint256 _gracePeriodBlocks) returns()
func (_BeefyClientWrapper *BeefyClientWrapperTransactorSession) SetGracePeriod(_gracePeriodBlocks *big.Int) (*types.Transaction, error) {
	return _BeefyClientWrapper.Contract.SetGracePeriod(&_BeefyClientWrapper.TransactOpts, _gracePeriodBlocks)
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

// SetMinBlockIncrement is a paid mutator transaction binding the contract method 0x3f358a76.
//
// Solidity: function setMinBlockIncrement(uint256 _minBlockIncrement) returns()
func (_BeefyClientWrapper *BeefyClientWrapperTransactor) SetMinBlockIncrement(opts *bind.TransactOpts, _minBlockIncrement *big.Int) (*types.Transaction, error) {
	return _BeefyClientWrapper.contract.Transact(opts, "setMinBlockIncrement", _minBlockIncrement)
}

// SetMinBlockIncrement is a paid mutator transaction binding the contract method 0x3f358a76.
//
// Solidity: function setMinBlockIncrement(uint256 _minBlockIncrement) returns()
func (_BeefyClientWrapper *BeefyClientWrapperSession) SetMinBlockIncrement(_minBlockIncrement *big.Int) (*types.Transaction, error) {
	return _BeefyClientWrapper.Contract.SetMinBlockIncrement(&_BeefyClientWrapper.TransactOpts, _minBlockIncrement)
}

// SetMinBlockIncrement is a paid mutator transaction binding the contract method 0x3f358a76.
//
// Solidity: function setMinBlockIncrement(uint256 _minBlockIncrement) returns()
func (_BeefyClientWrapper *BeefyClientWrapperTransactorSession) SetMinBlockIncrement(_minBlockIncrement *big.Int) (*types.Transaction, error) {
	return _BeefyClientWrapper.Contract.SetMinBlockIncrement(&_BeefyClientWrapper.TransactOpts, _minBlockIncrement)
}

// SubmitFinal is a paid mutator transaction binding the contract method 0xfafd0631.
//
// Solidity: function submitFinal((uint32,uint64,(bytes2,bytes)[]) commitment, uint256[] bitfield, (uint8,bytes32,bytes32,uint256,address,bytes32[])[] proofs, (uint8,uint32,bytes32,uint64,uint32,bytes32,bytes32) leaf, bytes32[] leafProof, uint256 leafProofOrder, uint32[] claimTipBlocks) returns()
func (_BeefyClientWrapper *BeefyClientWrapperTransactor) SubmitFinal(opts *bind.TransactOpts, commitment IBeefyClientCommitment, bitfield []*big.Int, proofs []IBeefyClientValidatorProof, leaf IBeefyClientMMRLeaf, leafProof [][32]byte, leafProofOrder *big.Int, claimTipBlocks []uint32) (*types.Transaction, error) {
	return _BeefyClientWrapper.contract.Transact(opts, "submitFinal", commitment, bitfield, proofs, leaf, leafProof, leafProofOrder, claimTipBlocks)
}

// SubmitFinal is a paid mutator transaction binding the contract method 0xfafd0631.
//
// Solidity: function submitFinal((uint32,uint64,(bytes2,bytes)[]) commitment, uint256[] bitfield, (uint8,bytes32,bytes32,uint256,address,bytes32[])[] proofs, (uint8,uint32,bytes32,uint64,uint32,bytes32,bytes32) leaf, bytes32[] leafProof, uint256 leafProofOrder, uint32[] claimTipBlocks) returns()
func (_BeefyClientWrapper *BeefyClientWrapperSession) SubmitFinal(commitment IBeefyClientCommitment, bitfield []*big.Int, proofs []IBeefyClientValidatorProof, leaf IBeefyClientMMRLeaf, leafProof [][32]byte, leafProofOrder *big.Int, claimTipBlocks []uint32) (*types.Transaction, error) {
	return _BeefyClientWrapper.Contract.SubmitFinal(&_BeefyClientWrapper.TransactOpts, commitment, bitfield, proofs, leaf, leafProof, leafProofOrder, claimTipBlocks)
}

// SubmitFinal is a paid mutator transaction binding the contract method 0xfafd0631.
//
// Solidity: function submitFinal((uint32,uint64,(bytes2,bytes)[]) commitment, uint256[] bitfield, (uint8,bytes32,bytes32,uint256,address,bytes32[])[] proofs, (uint8,uint32,bytes32,uint64,uint32,bytes32,bytes32) leaf, bytes32[] leafProof, uint256 leafProofOrder, uint32[] claimTipBlocks) returns()
func (_BeefyClientWrapper *BeefyClientWrapperTransactorSession) SubmitFinal(commitment IBeefyClientCommitment, bitfield []*big.Int, proofs []IBeefyClientValidatorProof, leaf IBeefyClientMMRLeaf, leafProof [][32]byte, leafProofOrder *big.Int, claimTipBlocks []uint32) (*types.Transaction, error) {
	return _BeefyClientWrapper.Contract.SubmitFinal(&_BeefyClientWrapper.TransactOpts, commitment, bitfield, proofs, leaf, leafProof, leafProofOrder, claimTipBlocks)
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

// UpgradeTo is a paid mutator transaction binding the contract method 0x50747ac8.
//
// Solidity: function upgradeTo(address newImplementation, bytes32 expectedCodeHash) returns()
func (_BeefyClientWrapper *BeefyClientWrapperTransactor) UpgradeTo(opts *bind.TransactOpts, newImplementation common.Address, expectedCodeHash [32]byte) (*types.Transaction, error) {
	return _BeefyClientWrapper.contract.Transact(opts, "upgradeTo", newImplementation, expectedCodeHash)
}

// UpgradeTo is a paid mutator transaction binding the contract method 0x50747ac8.
//
// Solidity: function upgradeTo(address newImplementation, bytes32 expectedCodeHash) returns()
func (_BeefyClientWrapper *BeefyClientWrapperSession) UpgradeTo(newImplementation common.Address, expectedCodeHash [32]byte) (*types.Transaction, error) {
	return _BeefyClientWrapper.Contract.UpgradeTo(&_BeefyClientWrapper.TransactOpts, newImplementation, expectedCodeHash)
}

// UpgradeTo is a paid mutator transaction binding the contract method 0x50747ac8.
//
// Solidity: function upgradeTo(address newImplementation, bytes32 expectedCodeHash) returns()
func (_BeefyClientWrapper *BeefyClientWrapperTransactorSession) UpgradeTo(newImplementation common.Address, expectedCodeHash [32]byte) (*types.Transaction, error) {
	return _BeefyClientWrapper.Contract.UpgradeTo(&_BeefyClientWrapper.TransactOpts, newImplementation, expectedCodeHash)
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

// BeefyClientWrapperGasCreditedIterator is returned from FilterGasCredited and is used to iterate over the raw logs and unpacked data for GasCredited events raised by the BeefyClientWrapper contract.
type BeefyClientWrapperGasCreditedIterator struct {
	Event *BeefyClientWrapperGasCredited // Event containing the contract specifics and raw log

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
func (it *BeefyClientWrapperGasCreditedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(BeefyClientWrapperGasCredited)
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
		it.Event = new(BeefyClientWrapperGasCredited)
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
func (it *BeefyClientWrapperGasCreditedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *BeefyClientWrapperGasCreditedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// BeefyClientWrapperGasCredited represents a GasCredited event raised by the BeefyClientWrapper contract.
type BeefyClientWrapperGasCredited struct {
	Relayer        common.Address
	CommitmentHash [32]byte
	GasUsed        *big.Int
	Raw            types.Log // Blockchain specific contextual infos
}

// FilterGasCredited is a free log retrieval operation binding the contract event 0xa96627e523aafcce96e4a95478ed8181042d6fcd32fd0892e1743408da8948e2.
//
// Solidity: event GasCredited(address indexed relayer, bytes32 indexed commitmentHash, uint256 gasUsed)
func (_BeefyClientWrapper *BeefyClientWrapperFilterer) FilterGasCredited(opts *bind.FilterOpts, relayer []common.Address, commitmentHash [][32]byte) (*BeefyClientWrapperGasCreditedIterator, error) {

	var relayerRule []interface{}
	for _, relayerItem := range relayer {
		relayerRule = append(relayerRule, relayerItem)
	}
	var commitmentHashRule []interface{}
	for _, commitmentHashItem := range commitmentHash {
		commitmentHashRule = append(commitmentHashRule, commitmentHashItem)
	}

	logs, sub, err := _BeefyClientWrapper.contract.FilterLogs(opts, "GasCredited", relayerRule, commitmentHashRule)
	if err != nil {
		return nil, err
	}
	return &BeefyClientWrapperGasCreditedIterator{contract: _BeefyClientWrapper.contract, event: "GasCredited", logs: logs, sub: sub}, nil
}

// WatchGasCredited is a free log subscription operation binding the contract event 0xa96627e523aafcce96e4a95478ed8181042d6fcd32fd0892e1743408da8948e2.
//
// Solidity: event GasCredited(address indexed relayer, bytes32 indexed commitmentHash, uint256 gasUsed)
func (_BeefyClientWrapper *BeefyClientWrapperFilterer) WatchGasCredited(opts *bind.WatchOpts, sink chan<- *BeefyClientWrapperGasCredited, relayer []common.Address, commitmentHash [][32]byte) (event.Subscription, error) {

	var relayerRule []interface{}
	for _, relayerItem := range relayer {
		relayerRule = append(relayerRule, relayerItem)
	}
	var commitmentHashRule []interface{}
	for _, commitmentHashItem := range commitmentHash {
		commitmentHashRule = append(commitmentHashRule, commitmentHashItem)
	}

	logs, sub, err := _BeefyClientWrapper.contract.WatchLogs(opts, "GasCredited", relayerRule, commitmentHashRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(BeefyClientWrapperGasCredited)
				if err := _BeefyClientWrapper.contract.UnpackLog(event, "GasCredited", log); err != nil {
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

// ParseGasCredited is a log parse operation binding the contract event 0xa96627e523aafcce96e4a95478ed8181042d6fcd32fd0892e1743408da8948e2.
//
// Solidity: event GasCredited(address indexed relayer, bytes32 indexed commitmentHash, uint256 gasUsed)
func (_BeefyClientWrapper *BeefyClientWrapperFilterer) ParseGasCredited(log types.Log) (*BeefyClientWrapperGasCredited, error) {
	event := new(BeefyClientWrapperGasCredited)
	if err := _BeefyClientWrapper.contract.UnpackLog(event, "GasCredited", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// BeefyClientWrapperRelayerAddedIterator is returned from FilterRelayerAdded and is used to iterate over the raw logs and unpacked data for RelayerAdded events raised by the BeefyClientWrapper contract.
type BeefyClientWrapperRelayerAddedIterator struct {
	Event *BeefyClientWrapperRelayerAdded // Event containing the contract specifics and raw log

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
func (it *BeefyClientWrapperRelayerAddedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(BeefyClientWrapperRelayerAdded)
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
		it.Event = new(BeefyClientWrapperRelayerAdded)
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
func (it *BeefyClientWrapperRelayerAddedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *BeefyClientWrapperRelayerAddedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// BeefyClientWrapperRelayerAdded represents a RelayerAdded event raised by the BeefyClientWrapper contract.
type BeefyClientWrapperRelayerAdded struct {
	Relayer common.Address
	Raw     types.Log // Blockchain specific contextual infos
}

// FilterRelayerAdded is a free log retrieval operation binding the contract event 0x03580ee9f53a62b7cb409a2cb56f9be87747dd15017afc5cef6eef321e4fb2c5.
//
// Solidity: event RelayerAdded(address indexed relayer)
func (_BeefyClientWrapper *BeefyClientWrapperFilterer) FilterRelayerAdded(opts *bind.FilterOpts, relayer []common.Address) (*BeefyClientWrapperRelayerAddedIterator, error) {

	var relayerRule []interface{}
	for _, relayerItem := range relayer {
		relayerRule = append(relayerRule, relayerItem)
	}

	logs, sub, err := _BeefyClientWrapper.contract.FilterLogs(opts, "RelayerAdded", relayerRule)
	if err != nil {
		return nil, err
	}
	return &BeefyClientWrapperRelayerAddedIterator{contract: _BeefyClientWrapper.contract, event: "RelayerAdded", logs: logs, sub: sub}, nil
}

// WatchRelayerAdded is a free log subscription operation binding the contract event 0x03580ee9f53a62b7cb409a2cb56f9be87747dd15017afc5cef6eef321e4fb2c5.
//
// Solidity: event RelayerAdded(address indexed relayer)
func (_BeefyClientWrapper *BeefyClientWrapperFilterer) WatchRelayerAdded(opts *bind.WatchOpts, sink chan<- *BeefyClientWrapperRelayerAdded, relayer []common.Address) (event.Subscription, error) {

	var relayerRule []interface{}
	for _, relayerItem := range relayer {
		relayerRule = append(relayerRule, relayerItem)
	}

	logs, sub, err := _BeefyClientWrapper.contract.WatchLogs(opts, "RelayerAdded", relayerRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(BeefyClientWrapperRelayerAdded)
				if err := _BeefyClientWrapper.contract.UnpackLog(event, "RelayerAdded", log); err != nil {
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

// ParseRelayerAdded is a log parse operation binding the contract event 0x03580ee9f53a62b7cb409a2cb56f9be87747dd15017afc5cef6eef321e4fb2c5.
//
// Solidity: event RelayerAdded(address indexed relayer)
func (_BeefyClientWrapper *BeefyClientWrapperFilterer) ParseRelayerAdded(log types.Log) (*BeefyClientWrapperRelayerAdded, error) {
	event := new(BeefyClientWrapperRelayerAdded)
	if err := _BeefyClientWrapper.contract.UnpackLog(event, "RelayerAdded", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// BeefyClientWrapperRelayerRemovedIterator is returned from FilterRelayerRemoved and is used to iterate over the raw logs and unpacked data for RelayerRemoved events raised by the BeefyClientWrapper contract.
type BeefyClientWrapperRelayerRemovedIterator struct {
	Event *BeefyClientWrapperRelayerRemoved // Event containing the contract specifics and raw log

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
func (it *BeefyClientWrapperRelayerRemovedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(BeefyClientWrapperRelayerRemoved)
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
		it.Event = new(BeefyClientWrapperRelayerRemoved)
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
func (it *BeefyClientWrapperRelayerRemovedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *BeefyClientWrapperRelayerRemovedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// BeefyClientWrapperRelayerRemoved represents a RelayerRemoved event raised by the BeefyClientWrapper contract.
type BeefyClientWrapperRelayerRemoved struct {
	Relayer common.Address
	Raw     types.Log // Blockchain specific contextual infos
}

// FilterRelayerRemoved is a free log retrieval operation binding the contract event 0x10e1f7ce9fd7d1b90a66d13a2ab3cb8dd7f29f3f8d520b143b063ccfbab6906b.
//
// Solidity: event RelayerRemoved(address indexed relayer)
func (_BeefyClientWrapper *BeefyClientWrapperFilterer) FilterRelayerRemoved(opts *bind.FilterOpts, relayer []common.Address) (*BeefyClientWrapperRelayerRemovedIterator, error) {

	var relayerRule []interface{}
	for _, relayerItem := range relayer {
		relayerRule = append(relayerRule, relayerItem)
	}

	logs, sub, err := _BeefyClientWrapper.contract.FilterLogs(opts, "RelayerRemoved", relayerRule)
	if err != nil {
		return nil, err
	}
	return &BeefyClientWrapperRelayerRemovedIterator{contract: _BeefyClientWrapper.contract, event: "RelayerRemoved", logs: logs, sub: sub}, nil
}

// WatchRelayerRemoved is a free log subscription operation binding the contract event 0x10e1f7ce9fd7d1b90a66d13a2ab3cb8dd7f29f3f8d520b143b063ccfbab6906b.
//
// Solidity: event RelayerRemoved(address indexed relayer)
func (_BeefyClientWrapper *BeefyClientWrapperFilterer) WatchRelayerRemoved(opts *bind.WatchOpts, sink chan<- *BeefyClientWrapperRelayerRemoved, relayer []common.Address) (event.Subscription, error) {

	var relayerRule []interface{}
	for _, relayerItem := range relayer {
		relayerRule = append(relayerRule, relayerItem)
	}

	logs, sub, err := _BeefyClientWrapper.contract.WatchLogs(opts, "RelayerRemoved", relayerRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(BeefyClientWrapperRelayerRemoved)
				if err := _BeefyClientWrapper.contract.UnpackLog(event, "RelayerRemoved", log); err != nil {
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

// ParseRelayerRemoved is a log parse operation binding the contract event 0x10e1f7ce9fd7d1b90a66d13a2ab3cb8dd7f29f3f8d520b143b063ccfbab6906b.
//
// Solidity: event RelayerRemoved(address indexed relayer)
func (_BeefyClientWrapper *BeefyClientWrapperFilterer) ParseRelayerRemoved(log types.Log) (*BeefyClientWrapperRelayerRemoved, error) {
	event := new(BeefyClientWrapperRelayerRemoved)
	if err := _BeefyClientWrapper.contract.UnpackLog(event, "RelayerRemoved", log); err != nil {
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
	Amount       *big.Int
	TotalGasUsed *big.Int
	Raw          types.Log // Blockchain specific contextual infos
}

// FilterSubmissionRefunded is a free log retrieval operation binding the contract event 0x103cb711554967fbeea0c6394a6dfedc44d0e729191e719d0976438783912f52.
//
// Solidity: event SubmissionRefunded(address indexed relayer, uint256 amount, uint256 totalGasUsed)
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
// Solidity: event SubmissionRefunded(address indexed relayer, uint256 amount, uint256 totalGasUsed)
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
// Solidity: event SubmissionRefunded(address indexed relayer, uint256 amount, uint256 totalGasUsed)
func (_BeefyClientWrapper *BeefyClientWrapperFilterer) ParseSubmissionRefunded(log types.Log) (*BeefyClientWrapperSubmissionRefunded, error) {
	event := new(BeefyClientWrapperSubmissionRefunded)
	if err := _BeefyClientWrapper.contract.UnpackLog(event, "SubmissionRefunded", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// BeefyClientWrapperTipAddedIterator is returned from FilterTipAdded and is used to iterate over the raw logs and unpacked data for TipAdded events raised by the BeefyClientWrapper contract.
type BeefyClientWrapperTipAddedIterator struct {
	Event *BeefyClientWrapperTipAdded // Event containing the contract specifics and raw log

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
func (it *BeefyClientWrapperTipAddedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(BeefyClientWrapperTipAdded)
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
		it.Event = new(BeefyClientWrapperTipAdded)
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
func (it *BeefyClientWrapperTipAddedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *BeefyClientWrapperTipAddedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// BeefyClientWrapperTipAdded represents a TipAdded event raised by the BeefyClientWrapper contract.
type BeefyClientWrapperTipAdded struct {
	Tipper           common.Address
	BeefyBlockNumber uint32
	Amount           *big.Int
	Raw              types.Log // Blockchain specific contextual infos
}

// FilterTipAdded is a free log retrieval operation binding the contract event 0x534f476b78e1964cad3783848d29e18156e2e9a2a3cfe8f93735b141d8c1285a.
//
// Solidity: event TipAdded(address indexed tipper, uint32 indexed beefyBlockNumber, uint256 amount)
func (_BeefyClientWrapper *BeefyClientWrapperFilterer) FilterTipAdded(opts *bind.FilterOpts, tipper []common.Address, beefyBlockNumber []uint32) (*BeefyClientWrapperTipAddedIterator, error) {

	var tipperRule []interface{}
	for _, tipperItem := range tipper {
		tipperRule = append(tipperRule, tipperItem)
	}
	var beefyBlockNumberRule []interface{}
	for _, beefyBlockNumberItem := range beefyBlockNumber {
		beefyBlockNumberRule = append(beefyBlockNumberRule, beefyBlockNumberItem)
	}

	logs, sub, err := _BeefyClientWrapper.contract.FilterLogs(opts, "TipAdded", tipperRule, beefyBlockNumberRule)
	if err != nil {
		return nil, err
	}
	return &BeefyClientWrapperTipAddedIterator{contract: _BeefyClientWrapper.contract, event: "TipAdded", logs: logs, sub: sub}, nil
}

// WatchTipAdded is a free log subscription operation binding the contract event 0x534f476b78e1964cad3783848d29e18156e2e9a2a3cfe8f93735b141d8c1285a.
//
// Solidity: event TipAdded(address indexed tipper, uint32 indexed beefyBlockNumber, uint256 amount)
func (_BeefyClientWrapper *BeefyClientWrapperFilterer) WatchTipAdded(opts *bind.WatchOpts, sink chan<- *BeefyClientWrapperTipAdded, tipper []common.Address, beefyBlockNumber []uint32) (event.Subscription, error) {

	var tipperRule []interface{}
	for _, tipperItem := range tipper {
		tipperRule = append(tipperRule, tipperItem)
	}
	var beefyBlockNumberRule []interface{}
	for _, beefyBlockNumberItem := range beefyBlockNumber {
		beefyBlockNumberRule = append(beefyBlockNumberRule, beefyBlockNumberItem)
	}

	logs, sub, err := _BeefyClientWrapper.contract.WatchLogs(opts, "TipAdded", tipperRule, beefyBlockNumberRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(BeefyClientWrapperTipAdded)
				if err := _BeefyClientWrapper.contract.UnpackLog(event, "TipAdded", log); err != nil {
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

// ParseTipAdded is a log parse operation binding the contract event 0x534f476b78e1964cad3783848d29e18156e2e9a2a3cfe8f93735b141d8c1285a.
//
// Solidity: event TipAdded(address indexed tipper, uint32 indexed beefyBlockNumber, uint256 amount)
func (_BeefyClientWrapper *BeefyClientWrapperFilterer) ParseTipAdded(log types.Log) (*BeefyClientWrapperTipAdded, error) {
	event := new(BeefyClientWrapperTipAdded)
	if err := _BeefyClientWrapper.contract.UnpackLog(event, "TipAdded", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// BeefyClientWrapperTipsClaimedIterator is returned from FilterTipsClaimed and is used to iterate over the raw logs and unpacked data for TipsClaimed events raised by the BeefyClientWrapper contract.
type BeefyClientWrapperTipsClaimedIterator struct {
	Event *BeefyClientWrapperTipsClaimed // Event containing the contract specifics and raw log

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
func (it *BeefyClientWrapperTipsClaimedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(BeefyClientWrapperTipsClaimed)
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
		it.Event = new(BeefyClientWrapperTipsClaimed)
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
func (it *BeefyClientWrapperTipsClaimedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *BeefyClientWrapperTipsClaimedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// BeefyClientWrapperTipsClaimed represents a TipsClaimed event raised by the BeefyClientWrapper contract.
type BeefyClientWrapperTipsClaimed struct {
	Relayer     common.Address
	TotalAmount *big.Int
	Raw         types.Log // Blockchain specific contextual infos
}

// FilterTipsClaimed is a free log retrieval operation binding the contract event 0xf5e7a62133aaed5c8783583a823343c7c0c0f51f0e2e8c76ec50feffc002b1f3.
//
// Solidity: event TipsClaimed(address indexed relayer, uint256 totalAmount)
func (_BeefyClientWrapper *BeefyClientWrapperFilterer) FilterTipsClaimed(opts *bind.FilterOpts, relayer []common.Address) (*BeefyClientWrapperTipsClaimedIterator, error) {

	var relayerRule []interface{}
	for _, relayerItem := range relayer {
		relayerRule = append(relayerRule, relayerItem)
	}

	logs, sub, err := _BeefyClientWrapper.contract.FilterLogs(opts, "TipsClaimed", relayerRule)
	if err != nil {
		return nil, err
	}
	return &BeefyClientWrapperTipsClaimedIterator{contract: _BeefyClientWrapper.contract, event: "TipsClaimed", logs: logs, sub: sub}, nil
}

// WatchTipsClaimed is a free log subscription operation binding the contract event 0xf5e7a62133aaed5c8783583a823343c7c0c0f51f0e2e8c76ec50feffc002b1f3.
//
// Solidity: event TipsClaimed(address indexed relayer, uint256 totalAmount)
func (_BeefyClientWrapper *BeefyClientWrapperFilterer) WatchTipsClaimed(opts *bind.WatchOpts, sink chan<- *BeefyClientWrapperTipsClaimed, relayer []common.Address) (event.Subscription, error) {

	var relayerRule []interface{}
	for _, relayerItem := range relayer {
		relayerRule = append(relayerRule, relayerItem)
	}

	logs, sub, err := _BeefyClientWrapper.contract.WatchLogs(opts, "TipsClaimed", relayerRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(BeefyClientWrapperTipsClaimed)
				if err := _BeefyClientWrapper.contract.UnpackLog(event, "TipsClaimed", log); err != nil {
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

// ParseTipsClaimed is a log parse operation binding the contract event 0xf5e7a62133aaed5c8783583a823343c7c0c0f51f0e2e8c76ec50feffc002b1f3.
//
// Solidity: event TipsClaimed(address indexed relayer, uint256 totalAmount)
func (_BeefyClientWrapper *BeefyClientWrapperFilterer) ParseTipsClaimed(log types.Log) (*BeefyClientWrapperTipsClaimed, error) {
	event := new(BeefyClientWrapperTipsClaimed)
	if err := _BeefyClientWrapper.contract.UnpackLog(event, "TipsClaimed", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// BeefyClientWrapperTurnAdvancedIterator is returned from FilterTurnAdvanced and is used to iterate over the raw logs and unpacked data for TurnAdvanced events raised by the BeefyClientWrapper contract.
type BeefyClientWrapperTurnAdvancedIterator struct {
	Event *BeefyClientWrapperTurnAdvanced // Event containing the contract specifics and raw log

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
func (it *BeefyClientWrapperTurnAdvancedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(BeefyClientWrapperTurnAdvanced)
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
		it.Event = new(BeefyClientWrapperTurnAdvanced)
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
func (it *BeefyClientWrapperTurnAdvancedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *BeefyClientWrapperTurnAdvancedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// BeefyClientWrapperTurnAdvanced represents a TurnAdvanced event raised by the BeefyClientWrapper contract.
type BeefyClientWrapperTurnAdvanced struct {
	NewTurnIndex *big.Int
	NextRelayer  common.Address
	Raw          types.Log // Blockchain specific contextual infos
}

// FilterTurnAdvanced is a free log retrieval operation binding the contract event 0xa1fbd218639d21bc371d11c5d99434f6b493f6b7c3740271906c91b62a215434.
//
// Solidity: event TurnAdvanced(uint256 indexed newTurnIndex, address indexed nextRelayer)
func (_BeefyClientWrapper *BeefyClientWrapperFilterer) FilterTurnAdvanced(opts *bind.FilterOpts, newTurnIndex []*big.Int, nextRelayer []common.Address) (*BeefyClientWrapperTurnAdvancedIterator, error) {

	var newTurnIndexRule []interface{}
	for _, newTurnIndexItem := range newTurnIndex {
		newTurnIndexRule = append(newTurnIndexRule, newTurnIndexItem)
	}
	var nextRelayerRule []interface{}
	for _, nextRelayerItem := range nextRelayer {
		nextRelayerRule = append(nextRelayerRule, nextRelayerItem)
	}

	logs, sub, err := _BeefyClientWrapper.contract.FilterLogs(opts, "TurnAdvanced", newTurnIndexRule, nextRelayerRule)
	if err != nil {
		return nil, err
	}
	return &BeefyClientWrapperTurnAdvancedIterator{contract: _BeefyClientWrapper.contract, event: "TurnAdvanced", logs: logs, sub: sub}, nil
}

// WatchTurnAdvanced is a free log subscription operation binding the contract event 0xa1fbd218639d21bc371d11c5d99434f6b493f6b7c3740271906c91b62a215434.
//
// Solidity: event TurnAdvanced(uint256 indexed newTurnIndex, address indexed nextRelayer)
func (_BeefyClientWrapper *BeefyClientWrapperFilterer) WatchTurnAdvanced(opts *bind.WatchOpts, sink chan<- *BeefyClientWrapperTurnAdvanced, newTurnIndex []*big.Int, nextRelayer []common.Address) (event.Subscription, error) {

	var newTurnIndexRule []interface{}
	for _, newTurnIndexItem := range newTurnIndex {
		newTurnIndexRule = append(newTurnIndexRule, newTurnIndexItem)
	}
	var nextRelayerRule []interface{}
	for _, nextRelayerItem := range nextRelayer {
		nextRelayerRule = append(nextRelayerRule, nextRelayerItem)
	}

	logs, sub, err := _BeefyClientWrapper.contract.WatchLogs(opts, "TurnAdvanced", newTurnIndexRule, nextRelayerRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(BeefyClientWrapperTurnAdvanced)
				if err := _BeefyClientWrapper.contract.UnpackLog(event, "TurnAdvanced", log); err != nil {
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

// ParseTurnAdvanced is a log parse operation binding the contract event 0xa1fbd218639d21bc371d11c5d99434f6b493f6b7c3740271906c91b62a215434.
//
// Solidity: event TurnAdvanced(uint256 indexed newTurnIndex, address indexed nextRelayer)
func (_BeefyClientWrapper *BeefyClientWrapperFilterer) ParseTurnAdvanced(log types.Log) (*BeefyClientWrapperTurnAdvanced, error) {
	event := new(BeefyClientWrapperTurnAdvanced)
	if err := _BeefyClientWrapper.contract.UnpackLog(event, "TurnAdvanced", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// BeefyClientWrapperUpgradedIterator is returned from FilterUpgraded and is used to iterate over the raw logs and unpacked data for Upgraded events raised by the BeefyClientWrapper contract.
type BeefyClientWrapperUpgradedIterator struct {
	Event *BeefyClientWrapperUpgraded // Event containing the contract specifics and raw log

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
func (it *BeefyClientWrapperUpgradedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(BeefyClientWrapperUpgraded)
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
		it.Event = new(BeefyClientWrapperUpgraded)
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
func (it *BeefyClientWrapperUpgradedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *BeefyClientWrapperUpgradedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// BeefyClientWrapperUpgraded represents a Upgraded event raised by the BeefyClientWrapper contract.
type BeefyClientWrapperUpgraded struct {
	Implementation common.Address
	Raw            types.Log // Blockchain specific contextual infos
}

// FilterUpgraded is a free log retrieval operation binding the contract event 0xbc7cd75a20ee27fd9adebab32041f755214dbc6bffa90cc0225b39da2e5c2d3b.
//
// Solidity: event Upgraded(address indexed implementation)
func (_BeefyClientWrapper *BeefyClientWrapperFilterer) FilterUpgraded(opts *bind.FilterOpts, implementation []common.Address) (*BeefyClientWrapperUpgradedIterator, error) {

	var implementationRule []interface{}
	for _, implementationItem := range implementation {
		implementationRule = append(implementationRule, implementationItem)
	}

	logs, sub, err := _BeefyClientWrapper.contract.FilterLogs(opts, "Upgraded", implementationRule)
	if err != nil {
		return nil, err
	}
	return &BeefyClientWrapperUpgradedIterator{contract: _BeefyClientWrapper.contract, event: "Upgraded", logs: logs, sub: sub}, nil
}

// WatchUpgraded is a free log subscription operation binding the contract event 0xbc7cd75a20ee27fd9adebab32041f755214dbc6bffa90cc0225b39da2e5c2d3b.
//
// Solidity: event Upgraded(address indexed implementation)
func (_BeefyClientWrapper *BeefyClientWrapperFilterer) WatchUpgraded(opts *bind.WatchOpts, sink chan<- *BeefyClientWrapperUpgraded, implementation []common.Address) (event.Subscription, error) {

	var implementationRule []interface{}
	for _, implementationItem := range implementation {
		implementationRule = append(implementationRule, implementationItem)
	}

	logs, sub, err := _BeefyClientWrapper.contract.WatchLogs(opts, "Upgraded", implementationRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(BeefyClientWrapperUpgraded)
				if err := _BeefyClientWrapper.contract.UnpackLog(event, "Upgraded", log); err != nil {
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

// ParseUpgraded is a log parse operation binding the contract event 0xbc7cd75a20ee27fd9adebab32041f755214dbc6bffa90cc0225b39da2e5c2d3b.
//
// Solidity: event Upgraded(address indexed implementation)
func (_BeefyClientWrapper *BeefyClientWrapperFilterer) ParseUpgraded(log types.Log) (*BeefyClientWrapperUpgraded, error) {
	event := new(BeefyClientWrapperUpgraded)
	if err := _BeefyClientWrapper.contract.UnpackLog(event, "Upgraded", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}
