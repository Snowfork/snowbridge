// Code generated - DO NOT EDIT.
// This file is a generated binding and any manual changes will be lost.

package paraclient

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

// MMRProof is an auto generated low-level Go binding around an user-defined struct.
type MMRProof struct {
	Items [][32]byte
	Order uint64
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

// ParachainClientProof is an auto generated low-level Go binding around an user-defined struct.
type ParachainClientProof struct {
	HeadPrefix  []byte
	HeadSuffix  []byte
	HeadProof   ParachainClientHeadProof
	LeafPartial ParachainClientMMRLeafPartial
	LeafProof   MMRProof
}

// ParachainClientMetaData contains all meta data concerning the ParachainClient contract.
var ParachainClientMetaData = &bind.MetaData{
	ABI: "[{\"inputs\":[{\"internalType\":\"contractBeefyClient\",\"name\":\"_client\",\"type\":\"address\"},{\"internalType\":\"uint32\",\"name\":\"_parachainID\",\"type\":\"uint32\"}],\"stateMutability\":\"nonpayable\",\"type\":\"constructor\"},{\"inputs\":[],\"name\":\"beefyClient\",\"outputs\":[{\"internalType\":\"contractBeefyClient\",\"name\":\"\",\"type\":\"address\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"encodedParachainID\",\"outputs\":[{\"internalType\":\"bytes4\",\"name\":\"\",\"type\":\"bytes4\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"parachainID\",\"outputs\":[{\"internalType\":\"uint32\",\"name\":\"\",\"type\":\"uint32\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"commitment\",\"type\":\"bytes32\"},{\"components\":[{\"internalType\":\"bytes\",\"name\":\"headPrefix\",\"type\":\"bytes\"},{\"internalType\":\"bytes\",\"name\":\"headSuffix\",\"type\":\"bytes\"},{\"components\":[{\"internalType\":\"uint256\",\"name\":\"pos\",\"type\":\"uint256\"},{\"internalType\":\"uint256\",\"name\":\"width\",\"type\":\"uint256\"},{\"internalType\":\"bytes32[]\",\"name\":\"proof\",\"type\":\"bytes32[]\"}],\"internalType\":\"structParachainClient.HeadProof\",\"name\":\"headProof\",\"type\":\"tuple\"},{\"components\":[{\"internalType\":\"uint8\",\"name\":\"version\",\"type\":\"uint8\"},{\"internalType\":\"uint32\",\"name\":\"parentNumber\",\"type\":\"uint32\"},{\"internalType\":\"bytes32\",\"name\":\"parentHash\",\"type\":\"bytes32\"},{\"internalType\":\"uint64\",\"name\":\"nextAuthoritySetID\",\"type\":\"uint64\"},{\"internalType\":\"uint32\",\"name\":\"nextAuthoritySetLen\",\"type\":\"uint32\"},{\"internalType\":\"bytes32\",\"name\":\"nextAuthoritySetRoot\",\"type\":\"bytes32\"}],\"internalType\":\"structParachainClient.MMRLeafPartial\",\"name\":\"leafPartial\",\"type\":\"tuple\"},{\"components\":[{\"internalType\":\"bytes32[]\",\"name\":\"items\",\"type\":\"bytes32[]\"},{\"internalType\":\"uint64\",\"name\":\"order\",\"type\":\"uint64\"}],\"internalType\":\"structMMRProof\",\"name\":\"leafProof\",\"type\":\"tuple\"}],\"internalType\":\"structParachainClient.Proof\",\"name\":\"proof\",\"type\":\"tuple\"}],\"name\":\"verifyCommitment\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"commitment\",\"type\":\"bytes32\"},{\"internalType\":\"bytes\",\"name\":\"opaqueProof\",\"type\":\"bytes\"}],\"name\":\"verifyCommitment\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\"}],\"stateMutability\":\"view\",\"type\":\"function\"}]",
}

// ParachainClientABI is the input ABI used to generate the binding from.
// Deprecated: Use ParachainClientMetaData.ABI instead.
var ParachainClientABI = ParachainClientMetaData.ABI

// ParachainClient is an auto generated Go binding around an Ethereum contract.
type ParachainClient struct {
	ParachainClientCaller     // Read-only binding to the contract
	ParachainClientTransactor // Write-only binding to the contract
	ParachainClientFilterer   // Log filterer for contract events
}

// ParachainClientCaller is an auto generated read-only Go binding around an Ethereum contract.
type ParachainClientCaller struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// ParachainClientTransactor is an auto generated write-only Go binding around an Ethereum contract.
type ParachainClientTransactor struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// ParachainClientFilterer is an auto generated log filtering Go binding around an Ethereum contract events.
type ParachainClientFilterer struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// ParachainClientSession is an auto generated Go binding around an Ethereum contract,
// with pre-set call and transact options.
type ParachainClientSession struct {
	Contract     *ParachainClient  // Generic contract binding to set the session for
	CallOpts     bind.CallOpts     // Call options to use throughout this session
	TransactOpts bind.TransactOpts // Transaction auth options to use throughout this session
}

// ParachainClientCallerSession is an auto generated read-only Go binding around an Ethereum contract,
// with pre-set call options.
type ParachainClientCallerSession struct {
	Contract *ParachainClientCaller // Generic contract caller binding to set the session for
	CallOpts bind.CallOpts          // Call options to use throughout this session
}

// ParachainClientTransactorSession is an auto generated write-only Go binding around an Ethereum contract,
// with pre-set transact options.
type ParachainClientTransactorSession struct {
	Contract     *ParachainClientTransactor // Generic contract transactor binding to set the session for
	TransactOpts bind.TransactOpts          // Transaction auth options to use throughout this session
}

// ParachainClientRaw is an auto generated low-level Go binding around an Ethereum contract.
type ParachainClientRaw struct {
	Contract *ParachainClient // Generic contract binding to access the raw methods on
}

// ParachainClientCallerRaw is an auto generated low-level read-only Go binding around an Ethereum contract.
type ParachainClientCallerRaw struct {
	Contract *ParachainClientCaller // Generic read-only contract binding to access the raw methods on
}

// ParachainClientTransactorRaw is an auto generated low-level write-only Go binding around an Ethereum contract.
type ParachainClientTransactorRaw struct {
	Contract *ParachainClientTransactor // Generic write-only contract binding to access the raw methods on
}

// NewParachainClient creates a new instance of ParachainClient, bound to a specific deployed contract.
func NewParachainClient(address common.Address, backend bind.ContractBackend) (*ParachainClient, error) {
	contract, err := bindParachainClient(address, backend, backend, backend)
	if err != nil {
		return nil, err
	}
	return &ParachainClient{ParachainClientCaller: ParachainClientCaller{contract: contract}, ParachainClientTransactor: ParachainClientTransactor{contract: contract}, ParachainClientFilterer: ParachainClientFilterer{contract: contract}}, nil
}

// NewParachainClientCaller creates a new read-only instance of ParachainClient, bound to a specific deployed contract.
func NewParachainClientCaller(address common.Address, caller bind.ContractCaller) (*ParachainClientCaller, error) {
	contract, err := bindParachainClient(address, caller, nil, nil)
	if err != nil {
		return nil, err
	}
	return &ParachainClientCaller{contract: contract}, nil
}

// NewParachainClientTransactor creates a new write-only instance of ParachainClient, bound to a specific deployed contract.
func NewParachainClientTransactor(address common.Address, transactor bind.ContractTransactor) (*ParachainClientTransactor, error) {
	contract, err := bindParachainClient(address, nil, transactor, nil)
	if err != nil {
		return nil, err
	}
	return &ParachainClientTransactor{contract: contract}, nil
}

// NewParachainClientFilterer creates a new log filterer instance of ParachainClient, bound to a specific deployed contract.
func NewParachainClientFilterer(address common.Address, filterer bind.ContractFilterer) (*ParachainClientFilterer, error) {
	contract, err := bindParachainClient(address, nil, nil, filterer)
	if err != nil {
		return nil, err
	}
	return &ParachainClientFilterer{contract: contract}, nil
}

// bindParachainClient binds a generic wrapper to an already deployed contract.
func bindParachainClient(address common.Address, caller bind.ContractCaller, transactor bind.ContractTransactor, filterer bind.ContractFilterer) (*bind.BoundContract, error) {
	parsed, err := abi.JSON(strings.NewReader(ParachainClientABI))
	if err != nil {
		return nil, err
	}
	return bind.NewBoundContract(address, parsed, caller, transactor, filterer), nil
}

// Call invokes the (constant) contract method with params as input values and
// sets the output to result. The result type might be a single field for simple
// returns, a slice of interfaces for anonymous returns and a struct for named
// returns.
func (_ParachainClient *ParachainClientRaw) Call(opts *bind.CallOpts, result *[]interface{}, method string, params ...interface{}) error {
	return _ParachainClient.Contract.ParachainClientCaller.contract.Call(opts, result, method, params...)
}

// Transfer initiates a plain transaction to move funds to the contract, calling
// its default method if one is available.
func (_ParachainClient *ParachainClientRaw) Transfer(opts *bind.TransactOpts) (*types.Transaction, error) {
	return _ParachainClient.Contract.ParachainClientTransactor.contract.Transfer(opts)
}

// Transact invokes the (paid) contract method with params as input values.
func (_ParachainClient *ParachainClientRaw) Transact(opts *bind.TransactOpts, method string, params ...interface{}) (*types.Transaction, error) {
	return _ParachainClient.Contract.ParachainClientTransactor.contract.Transact(opts, method, params...)
}

// Call invokes the (constant) contract method with params as input values and
// sets the output to result. The result type might be a single field for simple
// returns, a slice of interfaces for anonymous returns and a struct for named
// returns.
func (_ParachainClient *ParachainClientCallerRaw) Call(opts *bind.CallOpts, result *[]interface{}, method string, params ...interface{}) error {
	return _ParachainClient.Contract.contract.Call(opts, result, method, params...)
}

// Transfer initiates a plain transaction to move funds to the contract, calling
// its default method if one is available.
func (_ParachainClient *ParachainClientTransactorRaw) Transfer(opts *bind.TransactOpts) (*types.Transaction, error) {
	return _ParachainClient.Contract.contract.Transfer(opts)
}

// Transact invokes the (paid) contract method with params as input values.
func (_ParachainClient *ParachainClientTransactorRaw) Transact(opts *bind.TransactOpts, method string, params ...interface{}) (*types.Transaction, error) {
	return _ParachainClient.Contract.contract.Transact(opts, method, params...)
}

// BeefyClient is a free data retrieval call binding the contract method 0x776c81c3.
//
// Solidity: function beefyClient() view returns(address)
func (_ParachainClient *ParachainClientCaller) BeefyClient(opts *bind.CallOpts) (common.Address, error) {
	var out []interface{}
	err := _ParachainClient.contract.Call(opts, &out, "beefyClient")

	if err != nil {
		return *new(common.Address), err
	}

	out0 := *abi.ConvertType(out[0], new(common.Address)).(*common.Address)

	return out0, err

}

// BeefyClient is a free data retrieval call binding the contract method 0x776c81c3.
//
// Solidity: function beefyClient() view returns(address)
func (_ParachainClient *ParachainClientSession) BeefyClient() (common.Address, error) {
	return _ParachainClient.Contract.BeefyClient(&_ParachainClient.CallOpts)
}

// BeefyClient is a free data retrieval call binding the contract method 0x776c81c3.
//
// Solidity: function beefyClient() view returns(address)
func (_ParachainClient *ParachainClientCallerSession) BeefyClient() (common.Address, error) {
	return _ParachainClient.Contract.BeefyClient(&_ParachainClient.CallOpts)
}

// EncodedParachainID is a free data retrieval call binding the contract method 0x09571da0.
//
// Solidity: function encodedParachainID() view returns(bytes4)
func (_ParachainClient *ParachainClientCaller) EncodedParachainID(opts *bind.CallOpts) ([4]byte, error) {
	var out []interface{}
	err := _ParachainClient.contract.Call(opts, &out, "encodedParachainID")

	if err != nil {
		return *new([4]byte), err
	}

	out0 := *abi.ConvertType(out[0], new([4]byte)).(*[4]byte)

	return out0, err

}

// EncodedParachainID is a free data retrieval call binding the contract method 0x09571da0.
//
// Solidity: function encodedParachainID() view returns(bytes4)
func (_ParachainClient *ParachainClientSession) EncodedParachainID() ([4]byte, error) {
	return _ParachainClient.Contract.EncodedParachainID(&_ParachainClient.CallOpts)
}

// EncodedParachainID is a free data retrieval call binding the contract method 0x09571da0.
//
// Solidity: function encodedParachainID() view returns(bytes4)
func (_ParachainClient *ParachainClientCallerSession) EncodedParachainID() ([4]byte, error) {
	return _ParachainClient.Contract.EncodedParachainID(&_ParachainClient.CallOpts)
}

// ParachainID is a free data retrieval call binding the contract method 0x6a1143ca.
//
// Solidity: function parachainID() view returns(uint32)
func (_ParachainClient *ParachainClientCaller) ParachainID(opts *bind.CallOpts) (uint32, error) {
	var out []interface{}
	err := _ParachainClient.contract.Call(opts, &out, "parachainID")

	if err != nil {
		return *new(uint32), err
	}

	out0 := *abi.ConvertType(out[0], new(uint32)).(*uint32)

	return out0, err

}

// ParachainID is a free data retrieval call binding the contract method 0x6a1143ca.
//
// Solidity: function parachainID() view returns(uint32)
func (_ParachainClient *ParachainClientSession) ParachainID() (uint32, error) {
	return _ParachainClient.Contract.ParachainID(&_ParachainClient.CallOpts)
}

// ParachainID is a free data retrieval call binding the contract method 0x6a1143ca.
//
// Solidity: function parachainID() view returns(uint32)
func (_ParachainClient *ParachainClientCallerSession) ParachainID() (uint32, error) {
	return _ParachainClient.Contract.ParachainID(&_ParachainClient.CallOpts)
}

// VerifyCommitment is a free data retrieval call binding the contract method 0x9bc2cb0a.
//
// Solidity: function verifyCommitment(bytes32 commitment, (bytes,bytes,(uint256,uint256,bytes32[]),(uint8,uint32,bytes32,uint64,uint32,bytes32),(bytes32[],uint64)) proof) view returns(bool)
func (_ParachainClient *ParachainClientCaller) VerifyCommitment(opts *bind.CallOpts, commitment [32]byte, proof ParachainClientProof) (bool, error) {
	var out []interface{}
	err := _ParachainClient.contract.Call(opts, &out, "verifyCommitment", commitment, proof)

	if err != nil {
		return *new(bool), err
	}

	out0 := *abi.ConvertType(out[0], new(bool)).(*bool)

	return out0, err

}

// VerifyCommitment is a free data retrieval call binding the contract method 0x9bc2cb0a.
//
// Solidity: function verifyCommitment(bytes32 commitment, (bytes,bytes,(uint256,uint256,bytes32[]),(uint8,uint32,bytes32,uint64,uint32,bytes32),(bytes32[],uint64)) proof) view returns(bool)
func (_ParachainClient *ParachainClientSession) VerifyCommitment(commitment [32]byte, proof ParachainClientProof) (bool, error) {
	return _ParachainClient.Contract.VerifyCommitment(&_ParachainClient.CallOpts, commitment, proof)
}

// VerifyCommitment is a free data retrieval call binding the contract method 0x9bc2cb0a.
//
// Solidity: function verifyCommitment(bytes32 commitment, (bytes,bytes,(uint256,uint256,bytes32[]),(uint8,uint32,bytes32,uint64,uint32,bytes32),(bytes32[],uint64)) proof) view returns(bool)
func (_ParachainClient *ParachainClientCallerSession) VerifyCommitment(commitment [32]byte, proof ParachainClientProof) (bool, error) {
	return _ParachainClient.Contract.VerifyCommitment(&_ParachainClient.CallOpts, commitment, proof)
}

// VerifyCommitment0 is a free data retrieval call binding the contract method 0xa8eb1620.
//
// Solidity: function verifyCommitment(bytes32 commitment, bytes opaqueProof) view returns(bool)
func (_ParachainClient *ParachainClientCaller) VerifyCommitment0(opts *bind.CallOpts, commitment [32]byte, opaqueProof []byte) (bool, error) {
	var out []interface{}
	err := _ParachainClient.contract.Call(opts, &out, "verifyCommitment0", commitment, opaqueProof)

	if err != nil {
		return *new(bool), err
	}

	out0 := *abi.ConvertType(out[0], new(bool)).(*bool)

	return out0, err

}

// VerifyCommitment0 is a free data retrieval call binding the contract method 0xa8eb1620.
//
// Solidity: function verifyCommitment(bytes32 commitment, bytes opaqueProof) view returns(bool)
func (_ParachainClient *ParachainClientSession) VerifyCommitment0(commitment [32]byte, opaqueProof []byte) (bool, error) {
	return _ParachainClient.Contract.VerifyCommitment0(&_ParachainClient.CallOpts, commitment, opaqueProof)
}

// VerifyCommitment0 is a free data retrieval call binding the contract method 0xa8eb1620.
//
// Solidity: function verifyCommitment(bytes32 commitment, bytes opaqueProof) view returns(bool)
func (_ParachainClient *ParachainClientCallerSession) VerifyCommitment0(commitment [32]byte, opaqueProof []byte) (bool, error) {
	return _ParachainClient.Contract.VerifyCommitment0(&_ParachainClient.CallOpts, commitment, opaqueProof)
}
