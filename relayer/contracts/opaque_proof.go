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

// ParachainClientDigestItem is an auto generated low-level Go binding around an user-defined struct.
type ParachainClientDigestItem struct {
	Kind              *big.Int
	ConsensusEngineID [4]byte
	Data              []byte
}

// ParachainClientHeadProof is an auto generated low-level Go binding around an user-defined struct.
type ParachainClientHeadProof struct {
	Pos   *big.Int
	Width *big.Int
	Proof [][32]byte
}

// ParachainClientMMRLeafPartial is an auto generated low-level Go binding around an user-defined struct.
type ParachainClientMMRLeafPartial struct {
	Version              uint8
	ParentNumber         uint32
	ParentHash           [32]byte
	NextAuthoritySetID   uint64
	NextAuthoritySetLen  uint32
	NextAuthoritySetRoot [32]byte
}

// ParachainClientParachainHeader is an auto generated low-level Go binding around an user-defined struct.
type ParachainClientParachainHeader struct {
	ParentHash     [32]byte
	Number         *big.Int
	StateRoot      [32]byte
	ExtrinsicsRoot [32]byte
	DigestItems    []ParachainClientDigestItem
}

// ParachainClientProof is an auto generated low-level Go binding around an user-defined struct.
type ParachainClientProof struct {
	Header         ParachainClientParachainHeader
	HeadProof      ParachainClientHeadProof
	LeafPartial    ParachainClientMMRLeafPartial
	LeafProof      [][32]byte
	LeafProofOrder *big.Int
}

// OpaqueProofMetaData contains all meta data concerning the OpaqueProof contract.
var OpaqueProofMetaData = &bind.MetaData{
	ABI: "[{\"inputs\":[{\"components\":[{\"components\":[{\"internalType\":\"bytes32\",\"name\":\"parentHash\",\"type\":\"bytes32\"},{\"internalType\":\"uint256\",\"name\":\"number\",\"type\":\"uint256\"},{\"internalType\":\"bytes32\",\"name\":\"stateRoot\",\"type\":\"bytes32\"},{\"internalType\":\"bytes32\",\"name\":\"extrinsicsRoot\",\"type\":\"bytes32\"},{\"components\":[{\"internalType\":\"uint256\",\"name\":\"kind\",\"type\":\"uint256\"},{\"internalType\":\"bytes4\",\"name\":\"consensusEngineID\",\"type\":\"bytes4\"},{\"internalType\":\"bytes\",\"name\":\"data\",\"type\":\"bytes\"}],\"internalType\":\"structParachainClient.DigestItem[]\",\"name\":\"digestItems\",\"type\":\"tuple[]\"}],\"internalType\":\"structParachainClient.ParachainHeader\",\"name\":\"header\",\"type\":\"tuple\"},{\"components\":[{\"internalType\":\"uint256\",\"name\":\"pos\",\"type\":\"uint256\"},{\"internalType\":\"uint256\",\"name\":\"width\",\"type\":\"uint256\"},{\"internalType\":\"bytes32[]\",\"name\":\"proof\",\"type\":\"bytes32[]\"}],\"internalType\":\"structParachainClient.HeadProof\",\"name\":\"headProof\",\"type\":\"tuple\"},{\"components\":[{\"internalType\":\"uint8\",\"name\":\"version\",\"type\":\"uint8\"},{\"internalType\":\"uint32\",\"name\":\"parentNumber\",\"type\":\"uint32\"},{\"internalType\":\"bytes32\",\"name\":\"parentHash\",\"type\":\"bytes32\"},{\"internalType\":\"uint64\",\"name\":\"nextAuthoritySetID\",\"type\":\"uint64\"},{\"internalType\":\"uint32\",\"name\":\"nextAuthoritySetLen\",\"type\":\"uint32\"},{\"internalType\":\"bytes32\",\"name\":\"nextAuthoritySetRoot\",\"type\":\"bytes32\"}],\"internalType\":\"structParachainClient.MMRLeafPartial\",\"name\":\"leafPartial\",\"type\":\"tuple\"},{\"internalType\":\"bytes32[]\",\"name\":\"leafProof\",\"type\":\"bytes32[]\"},{\"internalType\":\"uint256\",\"name\":\"leafProofOrder\",\"type\":\"uint256\"}],\"internalType\":\"structParachainClient.Proof\",\"name\":\"proof\",\"type\":\"tuple\"}],\"name\":\"dummy\",\"outputs\":[],\"stateMutability\":\"pure\",\"type\":\"function\"}]",
}

// OpaqueProofABI is the input ABI used to generate the binding from.
// Deprecated: Use OpaqueProofMetaData.ABI instead.
var OpaqueProofABI = OpaqueProofMetaData.ABI

// OpaqueProof is an auto generated Go binding around an Ethereum contract.
type OpaqueProof struct {
	OpaqueProofCaller     // Read-only binding to the contract
	OpaqueProofTransactor // Write-only binding to the contract
	OpaqueProofFilterer   // Log filterer for contract events
}

// OpaqueProofCaller is an auto generated read-only Go binding around an Ethereum contract.
type OpaqueProofCaller struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// OpaqueProofTransactor is an auto generated write-only Go binding around an Ethereum contract.
type OpaqueProofTransactor struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// OpaqueProofFilterer is an auto generated log filtering Go binding around an Ethereum contract events.
type OpaqueProofFilterer struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// OpaqueProofSession is an auto generated Go binding around an Ethereum contract,
// with pre-set call and transact options.
type OpaqueProofSession struct {
	Contract     *OpaqueProof      // Generic contract binding to set the session for
	CallOpts     bind.CallOpts     // Call options to use throughout this session
	TransactOpts bind.TransactOpts // Transaction auth options to use throughout this session
}

// OpaqueProofCallerSession is an auto generated read-only Go binding around an Ethereum contract,
// with pre-set call options.
type OpaqueProofCallerSession struct {
	Contract *OpaqueProofCaller // Generic contract caller binding to set the session for
	CallOpts bind.CallOpts      // Call options to use throughout this session
}

// OpaqueProofTransactorSession is an auto generated write-only Go binding around an Ethereum contract,
// with pre-set transact options.
type OpaqueProofTransactorSession struct {
	Contract     *OpaqueProofTransactor // Generic contract transactor binding to set the session for
	TransactOpts bind.TransactOpts      // Transaction auth options to use throughout this session
}

// OpaqueProofRaw is an auto generated low-level Go binding around an Ethereum contract.
type OpaqueProofRaw struct {
	Contract *OpaqueProof // Generic contract binding to access the raw methods on
}

// OpaqueProofCallerRaw is an auto generated low-level read-only Go binding around an Ethereum contract.
type OpaqueProofCallerRaw struct {
	Contract *OpaqueProofCaller // Generic read-only contract binding to access the raw methods on
}

// OpaqueProofTransactorRaw is an auto generated low-level write-only Go binding around an Ethereum contract.
type OpaqueProofTransactorRaw struct {
	Contract *OpaqueProofTransactor // Generic write-only contract binding to access the raw methods on
}

// NewOpaqueProof creates a new instance of OpaqueProof, bound to a specific deployed contract.
func NewOpaqueProof(address common.Address, backend bind.ContractBackend) (*OpaqueProof, error) {
	contract, err := bindOpaqueProof(address, backend, backend, backend)
	if err != nil {
		return nil, err
	}
	return &OpaqueProof{OpaqueProofCaller: OpaqueProofCaller{contract: contract}, OpaqueProofTransactor: OpaqueProofTransactor{contract: contract}, OpaqueProofFilterer: OpaqueProofFilterer{contract: contract}}, nil
}

// NewOpaqueProofCaller creates a new read-only instance of OpaqueProof, bound to a specific deployed contract.
func NewOpaqueProofCaller(address common.Address, caller bind.ContractCaller) (*OpaqueProofCaller, error) {
	contract, err := bindOpaqueProof(address, caller, nil, nil)
	if err != nil {
		return nil, err
	}
	return &OpaqueProofCaller{contract: contract}, nil
}

// NewOpaqueProofTransactor creates a new write-only instance of OpaqueProof, bound to a specific deployed contract.
func NewOpaqueProofTransactor(address common.Address, transactor bind.ContractTransactor) (*OpaqueProofTransactor, error) {
	contract, err := bindOpaqueProof(address, nil, transactor, nil)
	if err != nil {
		return nil, err
	}
	return &OpaqueProofTransactor{contract: contract}, nil
}

// NewOpaqueProofFilterer creates a new log filterer instance of OpaqueProof, bound to a specific deployed contract.
func NewOpaqueProofFilterer(address common.Address, filterer bind.ContractFilterer) (*OpaqueProofFilterer, error) {
	contract, err := bindOpaqueProof(address, nil, nil, filterer)
	if err != nil {
		return nil, err
	}
	return &OpaqueProofFilterer{contract: contract}, nil
}

// bindOpaqueProof binds a generic wrapper to an already deployed contract.
func bindOpaqueProof(address common.Address, caller bind.ContractCaller, transactor bind.ContractTransactor, filterer bind.ContractFilterer) (*bind.BoundContract, error) {
	parsed, err := abi.JSON(strings.NewReader(OpaqueProofABI))
	if err != nil {
		return nil, err
	}
	return bind.NewBoundContract(address, parsed, caller, transactor, filterer), nil
}

// Call invokes the (constant) contract method with params as input values and
// sets the output to result. The result type might be a single field for simple
// returns, a slice of interfaces for anonymous returns and a struct for named
// returns.
func (_OpaqueProof *OpaqueProofRaw) Call(opts *bind.CallOpts, result *[]interface{}, method string, params ...interface{}) error {
	return _OpaqueProof.Contract.OpaqueProofCaller.contract.Call(opts, result, method, params...)
}

// Transfer initiates a plain transaction to move funds to the contract, calling
// its default method if one is available.
func (_OpaqueProof *OpaqueProofRaw) Transfer(opts *bind.TransactOpts) (*types.Transaction, error) {
	return _OpaqueProof.Contract.OpaqueProofTransactor.contract.Transfer(opts)
}

// Transact invokes the (paid) contract method with params as input values.
func (_OpaqueProof *OpaqueProofRaw) Transact(opts *bind.TransactOpts, method string, params ...interface{}) (*types.Transaction, error) {
	return _OpaqueProof.Contract.OpaqueProofTransactor.contract.Transact(opts, method, params...)
}

// Call invokes the (constant) contract method with params as input values and
// sets the output to result. The result type might be a single field for simple
// returns, a slice of interfaces for anonymous returns and a struct for named
// returns.
func (_OpaqueProof *OpaqueProofCallerRaw) Call(opts *bind.CallOpts, result *[]interface{}, method string, params ...interface{}) error {
	return _OpaqueProof.Contract.contract.Call(opts, result, method, params...)
}

// Transfer initiates a plain transaction to move funds to the contract, calling
// its default method if one is available.
func (_OpaqueProof *OpaqueProofTransactorRaw) Transfer(opts *bind.TransactOpts) (*types.Transaction, error) {
	return _OpaqueProof.Contract.contract.Transfer(opts)
}

// Transact invokes the (paid) contract method with params as input values.
func (_OpaqueProof *OpaqueProofTransactorRaw) Transact(opts *bind.TransactOpts, method string, params ...interface{}) (*types.Transaction, error) {
	return _OpaqueProof.Contract.contract.Transact(opts, method, params...)
}

// Dummy is a free data retrieval call binding the contract method 0xa454dc91.
//
// Solidity: function dummy(((bytes32,uint256,bytes32,bytes32,(uint256,bytes4,bytes)[]),(uint256,uint256,bytes32[]),(uint8,uint32,bytes32,uint64,uint32,bytes32),bytes32[],uint256) proof) pure returns()
func (_OpaqueProof *OpaqueProofCaller) Dummy(opts *bind.CallOpts, proof ParachainClientProof) error {
	var out []interface{}
	err := _OpaqueProof.contract.Call(opts, &out, "dummy", proof)

	if err != nil {
		return err
	}

	return err

}

// Dummy is a free data retrieval call binding the contract method 0xa454dc91.
//
// Solidity: function dummy(((bytes32,uint256,bytes32,bytes32,(uint256,bytes4,bytes)[]),(uint256,uint256,bytes32[]),(uint8,uint32,bytes32,uint64,uint32,bytes32),bytes32[],uint256) proof) pure returns()
func (_OpaqueProof *OpaqueProofSession) Dummy(proof ParachainClientProof) error {
	return _OpaqueProof.Contract.Dummy(&_OpaqueProof.CallOpts, proof)
}

// Dummy is a free data retrieval call binding the contract method 0xa454dc91.
//
// Solidity: function dummy(((bytes32,uint256,bytes32,bytes32,(uint256,bytes4,bytes)[]),(uint256,uint256,bytes32[]),(uint8,uint32,bytes32,uint64,uint32,bytes32),bytes32[],uint256) proof) pure returns()
func (_OpaqueProof *OpaqueProofCallerSession) Dummy(proof ParachainClientProof) error {
	return _OpaqueProof.Contract.Dummy(&_OpaqueProof.CallOpts, proof)
}
