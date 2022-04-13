// Code generated - DO NOT EDIT.
// This file is a generated binding and any manual changes will be lost.

package beefylightclient

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

// BeefyLightClientCommitment is an auto generated low-level Go binding around an user-defined struct.
type BeefyLightClientCommitment struct {
	BlockNumber    uint32
	ValidatorSetId uint64
	Payload        BeefyLightClientPayload
}

// BeefyLightClientMMRLeaf is an auto generated low-level Go binding around an user-defined struct.
type BeefyLightClientMMRLeaf struct {
	Version              uint8
	ParentNumber         uint32
	ParentHash           [32]byte
	NextAuthoritySetId   uint64
	NextAuthoritySetLen  uint32
	NextAuthoritySetRoot [32]byte
	ParachainHeadsRoot   [32]byte
}

// BeefyLightClientPayload is an auto generated low-level Go binding around an user-defined struct.
type BeefyLightClientPayload struct {
	MmrRootHash [32]byte
	Prefix      []byte
	Suffix      []byte
}

// BeefyLightClientValidatorProof is an auto generated low-level Go binding around an user-defined struct.
type BeefyLightClientValidatorProof struct {
	Signatures            [][]byte
	Positions             []*big.Int
	PublicKeys            []common.Address
	PublicKeyMerkleProofs [][][32]byte
}

// BeefyLightClientValidatorSet is an auto generated low-level Go binding around an user-defined struct.
type BeefyLightClientValidatorSet struct {
	Id     *big.Int
	Root   [32]byte
	Length *big.Int
}

// SimplifiedMMRProof is an auto generated low-level Go binding around an user-defined struct.
type SimplifiedMMRProof struct {
	MerkleProofItems         [][32]byte
	MerkleProofOrderBitField uint64
}

// ContractMetaData contains all meta data concerning the Contract contract.
var ContractMetaData = &bind.MetaData{
	ABI: "[{\"inputs\":[{\"internalType\":\"contractSimplifiedMMRVerification\",\"name\":\"_mmrVerification\",\"type\":\"address\"}],\"stateMutability\":\"nonpayable\",\"type\":\"constructor\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":false,\"internalType\":\"address\",\"name\":\"prover\",\"type\":\"address\"},{\"indexed\":false,\"internalType\":\"uint256\",\"name\":\"id\",\"type\":\"uint256\"}],\"name\":\"FinalVerificationSuccessful\",\"type\":\"event\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":false,\"internalType\":\"address\",\"name\":\"prover\",\"type\":\"address\"},{\"indexed\":false,\"internalType\":\"uint256\",\"name\":\"blockNumber\",\"type\":\"uint256\"},{\"indexed\":false,\"internalType\":\"uint256\",\"name\":\"id\",\"type\":\"uint256\"}],\"name\":\"InitialVerificationSuccessful\",\"type\":\"event\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":false,\"internalType\":\"bytes32\",\"name\":\"mmrRoot\",\"type\":\"bytes32\"},{\"indexed\":false,\"internalType\":\"uint64\",\"name\":\"blockNumber\",\"type\":\"uint64\"}],\"name\":\"NewMMRRoot\",\"type\":\"event\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":false,\"internalType\":\"uint256\",\"name\":\"validatorSetID\",\"type\":\"uint256\"},{\"indexed\":false,\"internalType\":\"bytes32\",\"name\":\"validatorSetRoot\",\"type\":\"bytes32\"},{\"indexed\":false,\"internalType\":\"uint256\",\"name\":\"validatorSetLength\",\"type\":\"uint256\"}],\"name\":\"NewSession\",\"type\":\"event\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":true,\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\"},{\"indexed\":true,\"internalType\":\"bytes32\",\"name\":\"previousAdminRole\",\"type\":\"bytes32\"},{\"indexed\":true,\"internalType\":\"bytes32\",\"name\":\"newAdminRole\",\"type\":\"bytes32\"}],\"name\":\"RoleAdminChanged\",\"type\":\"event\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":true,\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\"},{\"indexed\":true,\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\"},{\"indexed\":true,\"internalType\":\"address\",\"name\":\"sender\",\"type\":\"address\"}],\"name\":\"RoleGranted\",\"type\":\"event\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":true,\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\"},{\"indexed\":true,\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\"},{\"indexed\":true,\"internalType\":\"address\",\"name\":\"sender\",\"type\":\"address\"}],\"name\":\"RoleRevoked\",\"type\":\"event\"},{\"inputs\":[],\"name\":\"BLOCK_WAIT_PERIOD\",\"outputs\":[{\"internalType\":\"uint64\",\"name\":\"\",\"type\":\"uint64\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"DEFAULT_ADMIN_ROLE\",\"outputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"THRESHOLD_DENOMINATOR\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"THRESHOLD_NUMERATOR\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"id\",\"type\":\"uint256\"},{\"components\":[{\"internalType\":\"uint32\",\"name\":\"blockNumber\",\"type\":\"uint32\"},{\"internalType\":\"uint64\",\"name\":\"validatorSetId\",\"type\":\"uint64\"},{\"components\":[{\"internalType\":\"bytes32\",\"name\":\"mmrRootHash\",\"type\":\"bytes32\"},{\"internalType\":\"bytes\",\"name\":\"prefix\",\"type\":\"bytes\"},{\"internalType\":\"bytes\",\"name\":\"suffix\",\"type\":\"bytes\"}],\"internalType\":\"structBeefyLightClient.Payload\",\"name\":\"payload\",\"type\":\"tuple\"}],\"internalType\":\"structBeefyLightClient.Commitment\",\"name\":\"commitment\",\"type\":\"tuple\"},{\"components\":[{\"internalType\":\"bytes[]\",\"name\":\"signatures\",\"type\":\"bytes[]\"},{\"internalType\":\"uint256[]\",\"name\":\"positions\",\"type\":\"uint256[]\"},{\"internalType\":\"address[]\",\"name\":\"publicKeys\",\"type\":\"address[]\"},{\"internalType\":\"bytes32[][]\",\"name\":\"publicKeyMerkleProofs\",\"type\":\"bytes32[][]\"}],\"internalType\":\"structBeefyLightClient.ValidatorProof\",\"name\":\"validatorProof\",\"type\":\"tuple\"},{\"components\":[{\"internalType\":\"uint8\",\"name\":\"version\",\"type\":\"uint8\"},{\"internalType\":\"uint32\",\"name\":\"parentNumber\",\"type\":\"uint32\"},{\"internalType\":\"bytes32\",\"name\":\"parentHash\",\"type\":\"bytes32\"},{\"internalType\":\"uint64\",\"name\":\"nextAuthoritySetId\",\"type\":\"uint64\"},{\"internalType\":\"uint32\",\"name\":\"nextAuthoritySetLen\",\"type\":\"uint32\"},{\"internalType\":\"bytes32\",\"name\":\"nextAuthoritySetRoot\",\"type\":\"bytes32\"},{\"internalType\":\"bytes32\",\"name\":\"parachainHeadsRoot\",\"type\":\"bytes32\"}],\"internalType\":\"structBeefyLightClient.MMRLeaf\",\"name\":\"leaf\",\"type\":\"tuple\"},{\"components\":[{\"internalType\":\"bytes32[]\",\"name\":\"merkleProofItems\",\"type\":\"bytes32[]\"},{\"internalType\":\"uint64\",\"name\":\"merkleProofOrderBitField\",\"type\":\"uint64\"}],\"internalType\":\"structSimplifiedMMRProof\",\"name\":\"leafProof\",\"type\":\"tuple\"}],\"name\":\"completeSignatureCommitment\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"uint256[]\",\"name\":\"bitsToSet\",\"type\":\"uint256[]\"},{\"internalType\":\"uint256\",\"name\":\"length\",\"type\":\"uint256\"}],\"name\":\"createInitialBitfield\",\"outputs\":[{\"internalType\":\"uint256[]\",\"name\":\"\",\"type\":\"uint256[]\"}],\"stateMutability\":\"pure\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"id\",\"type\":\"uint256\"}],\"name\":\"createRandomBitfield\",\"outputs\":[{\"internalType\":\"uint256[]\",\"name\":\"\",\"type\":\"uint256[]\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"currentValidatorSet\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"id\",\"type\":\"uint256\"},{\"internalType\":\"bytes32\",\"name\":\"root\",\"type\":\"bytes32\"},{\"internalType\":\"uint256\",\"name\":\"length\",\"type\":\"uint256\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\"}],\"name\":\"getRoleAdmin\",\"outputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\"},{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\"}],\"name\":\"grantRole\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\"},{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\"}],\"name\":\"hasRole\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"uint64\",\"name\":\"_startingBeefyBlock\",\"type\":\"uint64\"},{\"components\":[{\"internalType\":\"uint256\",\"name\":\"id\",\"type\":\"uint256\"},{\"internalType\":\"bytes32\",\"name\":\"root\",\"type\":\"bytes32\"},{\"internalType\":\"uint256\",\"name\":\"length\",\"type\":\"uint256\"}],\"internalType\":\"structBeefyLightClient.ValidatorSet\",\"name\":\"_initialValidatorSet\",\"type\":\"tuple\"},{\"components\":[{\"internalType\":\"uint256\",\"name\":\"id\",\"type\":\"uint256\"},{\"internalType\":\"bytes32\",\"name\":\"root\",\"type\":\"bytes32\"},{\"internalType\":\"uint256\",\"name\":\"length\",\"type\":\"uint256\"}],\"internalType\":\"structBeefyLightClient.ValidatorSet\",\"name\":\"_nextValidatorSet\",\"type\":\"tuple\"}],\"name\":\"initialize\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[{\"components\":[{\"internalType\":\"uint256\",\"name\":\"id\",\"type\":\"uint256\"},{\"internalType\":\"bytes32\",\"name\":\"root\",\"type\":\"bytes32\"},{\"internalType\":\"uint256\",\"name\":\"length\",\"type\":\"uint256\"}],\"internalType\":\"structBeefyLightClient.ValidatorSet\",\"name\":\"vset\",\"type\":\"tuple\"},{\"internalType\":\"address\",\"name\":\"addr\",\"type\":\"address\"},{\"internalType\":\"uint256\",\"name\":\"pos\",\"type\":\"uint256\"},{\"internalType\":\"bytes32[]\",\"name\":\"proof\",\"type\":\"bytes32[]\"}],\"name\":\"isValidatorInSet\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"latestBeefyBlock\",\"outputs\":[{\"internalType\":\"uint64\",\"name\":\"\",\"type\":\"uint64\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"latestMMRRoot\",\"outputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"mmrVerification\",\"outputs\":[{\"internalType\":\"contractSimplifiedMMRVerification\",\"name\":\"\",\"type\":\"address\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"commitmentHash\",\"type\":\"bytes32\"},{\"internalType\":\"uint64\",\"name\":\"validatorSetID\",\"type\":\"uint64\"},{\"internalType\":\"uint256[]\",\"name\":\"validatorClaimsBitfield\",\"type\":\"uint256[]\"},{\"internalType\":\"bytes\",\"name\":\"validatorSignature\",\"type\":\"bytes\"},{\"internalType\":\"uint256\",\"name\":\"validatorPosition\",\"type\":\"uint256\"},{\"internalType\":\"address\",\"name\":\"validatorPublicKey\",\"type\":\"address\"},{\"internalType\":\"bytes32[]\",\"name\":\"validatorPublicKeyMerkleProof\",\"type\":\"bytes32[]\"}],\"name\":\"newSignatureCommitment\",\"outputs\":[],\"stateMutability\":\"payable\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"nextID\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"nextValidatorSet\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"id\",\"type\":\"uint256\"},{\"internalType\":\"bytes32\",\"name\":\"root\",\"type\":\"bytes32\"},{\"internalType\":\"uint256\",\"name\":\"length\",\"type\":\"uint256\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\"},{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\"}],\"name\":\"renounceRole\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\"},{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\"}],\"name\":\"revokeRole\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"bytes4\",\"name\":\"interfaceId\",\"type\":\"bytes4\"}],\"name\":\"supportsInterface\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\"}],\"name\":\"validationData\",\"outputs\":[{\"internalType\":\"address\",\"name\":\"senderAddress\",\"type\":\"address\"},{\"internalType\":\"bytes32\",\"name\":\"commitmentHash\",\"type\":\"bytes32\"},{\"internalType\":\"uint256\",\"name\":\"validatorSetID\",\"type\":\"uint256\"},{\"internalType\":\"uint256\",\"name\":\"blockNumber\",\"type\":\"uint256\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"beefyMMRLeaf\",\"type\":\"bytes32\"},{\"components\":[{\"internalType\":\"bytes32[]\",\"name\":\"merkleProofItems\",\"type\":\"bytes32[]\"},{\"internalType\":\"uint64\",\"name\":\"merkleProofOrderBitField\",\"type\":\"uint64\"}],\"internalType\":\"structSimplifiedMMRProof\",\"name\":\"proof\",\"type\":\"tuple\"}],\"name\":\"verifyBeefyMerkleLeaf\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\"}],\"stateMutability\":\"view\",\"type\":\"function\"}]",
}

// ContractABI is the input ABI used to generate the binding from.
// Deprecated: Use ContractMetaData.ABI instead.
var ContractABI = ContractMetaData.ABI

// Contract is an auto generated Go binding around an Ethereum contract.
type Contract struct {
	ContractCaller     // Read-only binding to the contract
	ContractTransactor // Write-only binding to the contract
	ContractFilterer   // Log filterer for contract events
}

// ContractCaller is an auto generated read-only Go binding around an Ethereum contract.
type ContractCaller struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// ContractTransactor is an auto generated write-only Go binding around an Ethereum contract.
type ContractTransactor struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// ContractFilterer is an auto generated log filtering Go binding around an Ethereum contract events.
type ContractFilterer struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// ContractSession is an auto generated Go binding around an Ethereum contract,
// with pre-set call and transact options.
type ContractSession struct {
	Contract     *Contract         // Generic contract binding to set the session for
	CallOpts     bind.CallOpts     // Call options to use throughout this session
	TransactOpts bind.TransactOpts // Transaction auth options to use throughout this session
}

// ContractCallerSession is an auto generated read-only Go binding around an Ethereum contract,
// with pre-set call options.
type ContractCallerSession struct {
	Contract *ContractCaller // Generic contract caller binding to set the session for
	CallOpts bind.CallOpts   // Call options to use throughout this session
}

// ContractTransactorSession is an auto generated write-only Go binding around an Ethereum contract,
// with pre-set transact options.
type ContractTransactorSession struct {
	Contract     *ContractTransactor // Generic contract transactor binding to set the session for
	TransactOpts bind.TransactOpts   // Transaction auth options to use throughout this session
}

// ContractRaw is an auto generated low-level Go binding around an Ethereum contract.
type ContractRaw struct {
	Contract *Contract // Generic contract binding to access the raw methods on
}

// ContractCallerRaw is an auto generated low-level read-only Go binding around an Ethereum contract.
type ContractCallerRaw struct {
	Contract *ContractCaller // Generic read-only contract binding to access the raw methods on
}

// ContractTransactorRaw is an auto generated low-level write-only Go binding around an Ethereum contract.
type ContractTransactorRaw struct {
	Contract *ContractTransactor // Generic write-only contract binding to access the raw methods on
}

// NewContract creates a new instance of Contract, bound to a specific deployed contract.
func NewContract(address common.Address, backend bind.ContractBackend) (*Contract, error) {
	contract, err := bindContract(address, backend, backend, backend)
	if err != nil {
		return nil, err
	}
	return &Contract{ContractCaller: ContractCaller{contract: contract}, ContractTransactor: ContractTransactor{contract: contract}, ContractFilterer: ContractFilterer{contract: contract}}, nil
}

// NewContractCaller creates a new read-only instance of Contract, bound to a specific deployed contract.
func NewContractCaller(address common.Address, caller bind.ContractCaller) (*ContractCaller, error) {
	contract, err := bindContract(address, caller, nil, nil)
	if err != nil {
		return nil, err
	}
	return &ContractCaller{contract: contract}, nil
}

// NewContractTransactor creates a new write-only instance of Contract, bound to a specific deployed contract.
func NewContractTransactor(address common.Address, transactor bind.ContractTransactor) (*ContractTransactor, error) {
	contract, err := bindContract(address, nil, transactor, nil)
	if err != nil {
		return nil, err
	}
	return &ContractTransactor{contract: contract}, nil
}

// NewContractFilterer creates a new log filterer instance of Contract, bound to a specific deployed contract.
func NewContractFilterer(address common.Address, filterer bind.ContractFilterer) (*ContractFilterer, error) {
	contract, err := bindContract(address, nil, nil, filterer)
	if err != nil {
		return nil, err
	}
	return &ContractFilterer{contract: contract}, nil
}

// bindContract binds a generic wrapper to an already deployed contract.
func bindContract(address common.Address, caller bind.ContractCaller, transactor bind.ContractTransactor, filterer bind.ContractFilterer) (*bind.BoundContract, error) {
	parsed, err := abi.JSON(strings.NewReader(ContractABI))
	if err != nil {
		return nil, err
	}
	return bind.NewBoundContract(address, parsed, caller, transactor, filterer), nil
}

// Call invokes the (constant) contract method with params as input values and
// sets the output to result. The result type might be a single field for simple
// returns, a slice of interfaces for anonymous returns and a struct for named
// returns.
func (_Contract *ContractRaw) Call(opts *bind.CallOpts, result *[]interface{}, method string, params ...interface{}) error {
	return _Contract.Contract.ContractCaller.contract.Call(opts, result, method, params...)
}

// Transfer initiates a plain transaction to move funds to the contract, calling
// its default method if one is available.
func (_Contract *ContractRaw) Transfer(opts *bind.TransactOpts) (*types.Transaction, error) {
	return _Contract.Contract.ContractTransactor.contract.Transfer(opts)
}

// Transact invokes the (paid) contract method with params as input values.
func (_Contract *ContractRaw) Transact(opts *bind.TransactOpts, method string, params ...interface{}) (*types.Transaction, error) {
	return _Contract.Contract.ContractTransactor.contract.Transact(opts, method, params...)
}

// Call invokes the (constant) contract method with params as input values and
// sets the output to result. The result type might be a single field for simple
// returns, a slice of interfaces for anonymous returns and a struct for named
// returns.
func (_Contract *ContractCallerRaw) Call(opts *bind.CallOpts, result *[]interface{}, method string, params ...interface{}) error {
	return _Contract.Contract.contract.Call(opts, result, method, params...)
}

// Transfer initiates a plain transaction to move funds to the contract, calling
// its default method if one is available.
func (_Contract *ContractTransactorRaw) Transfer(opts *bind.TransactOpts) (*types.Transaction, error) {
	return _Contract.Contract.contract.Transfer(opts)
}

// Transact invokes the (paid) contract method with params as input values.
func (_Contract *ContractTransactorRaw) Transact(opts *bind.TransactOpts, method string, params ...interface{}) (*types.Transaction, error) {
	return _Contract.Contract.contract.Transact(opts, method, params...)
}

// BLOCKWAITPERIOD is a free data retrieval call binding the contract method 0xfb752c62.
//
// Solidity: function BLOCK_WAIT_PERIOD() view returns(uint64)
func (_Contract *ContractCaller) BLOCKWAITPERIOD(opts *bind.CallOpts) (uint64, error) {
	var out []interface{}
	err := _Contract.contract.Call(opts, &out, "BLOCK_WAIT_PERIOD")

	if err != nil {
		return *new(uint64), err
	}

	out0 := *abi.ConvertType(out[0], new(uint64)).(*uint64)

	return out0, err

}

// BLOCKWAITPERIOD is a free data retrieval call binding the contract method 0xfb752c62.
//
// Solidity: function BLOCK_WAIT_PERIOD() view returns(uint64)
func (_Contract *ContractSession) BLOCKWAITPERIOD() (uint64, error) {
	return _Contract.Contract.BLOCKWAITPERIOD(&_Contract.CallOpts)
}

// BLOCKWAITPERIOD is a free data retrieval call binding the contract method 0xfb752c62.
//
// Solidity: function BLOCK_WAIT_PERIOD() view returns(uint64)
func (_Contract *ContractCallerSession) BLOCKWAITPERIOD() (uint64, error) {
	return _Contract.Contract.BLOCKWAITPERIOD(&_Contract.CallOpts)
}

// DEFAULTADMINROLE is a free data retrieval call binding the contract method 0xa217fddf.
//
// Solidity: function DEFAULT_ADMIN_ROLE() view returns(bytes32)
func (_Contract *ContractCaller) DEFAULTADMINROLE(opts *bind.CallOpts) ([32]byte, error) {
	var out []interface{}
	err := _Contract.contract.Call(opts, &out, "DEFAULT_ADMIN_ROLE")

	if err != nil {
		return *new([32]byte), err
	}

	out0 := *abi.ConvertType(out[0], new([32]byte)).(*[32]byte)

	return out0, err

}

// DEFAULTADMINROLE is a free data retrieval call binding the contract method 0xa217fddf.
//
// Solidity: function DEFAULT_ADMIN_ROLE() view returns(bytes32)
func (_Contract *ContractSession) DEFAULTADMINROLE() ([32]byte, error) {
	return _Contract.Contract.DEFAULTADMINROLE(&_Contract.CallOpts)
}

// DEFAULTADMINROLE is a free data retrieval call binding the contract method 0xa217fddf.
//
// Solidity: function DEFAULT_ADMIN_ROLE() view returns(bytes32)
func (_Contract *ContractCallerSession) DEFAULTADMINROLE() ([32]byte, error) {
	return _Contract.Contract.DEFAULTADMINROLE(&_Contract.CallOpts)
}

// THRESHOLDDENOMINATOR is a free data retrieval call binding the contract method 0xef024458.
//
// Solidity: function THRESHOLD_DENOMINATOR() view returns(uint256)
func (_Contract *ContractCaller) THRESHOLDDENOMINATOR(opts *bind.CallOpts) (*big.Int, error) {
	var out []interface{}
	err := _Contract.contract.Call(opts, &out, "THRESHOLD_DENOMINATOR")

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// THRESHOLDDENOMINATOR is a free data retrieval call binding the contract method 0xef024458.
//
// Solidity: function THRESHOLD_DENOMINATOR() view returns(uint256)
func (_Contract *ContractSession) THRESHOLDDENOMINATOR() (*big.Int, error) {
	return _Contract.Contract.THRESHOLDDENOMINATOR(&_Contract.CallOpts)
}

// THRESHOLDDENOMINATOR is a free data retrieval call binding the contract method 0xef024458.
//
// Solidity: function THRESHOLD_DENOMINATOR() view returns(uint256)
func (_Contract *ContractCallerSession) THRESHOLDDENOMINATOR() (*big.Int, error) {
	return _Contract.Contract.THRESHOLDDENOMINATOR(&_Contract.CallOpts)
}

// THRESHOLDNUMERATOR is a free data retrieval call binding the contract method 0x5a8d2f0e.
//
// Solidity: function THRESHOLD_NUMERATOR() view returns(uint256)
func (_Contract *ContractCaller) THRESHOLDNUMERATOR(opts *bind.CallOpts) (*big.Int, error) {
	var out []interface{}
	err := _Contract.contract.Call(opts, &out, "THRESHOLD_NUMERATOR")

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// THRESHOLDNUMERATOR is a free data retrieval call binding the contract method 0x5a8d2f0e.
//
// Solidity: function THRESHOLD_NUMERATOR() view returns(uint256)
func (_Contract *ContractSession) THRESHOLDNUMERATOR() (*big.Int, error) {
	return _Contract.Contract.THRESHOLDNUMERATOR(&_Contract.CallOpts)
}

// THRESHOLDNUMERATOR is a free data retrieval call binding the contract method 0x5a8d2f0e.
//
// Solidity: function THRESHOLD_NUMERATOR() view returns(uint256)
func (_Contract *ContractCallerSession) THRESHOLDNUMERATOR() (*big.Int, error) {
	return _Contract.Contract.THRESHOLDNUMERATOR(&_Contract.CallOpts)
}

// CreateInitialBitfield is a free data retrieval call binding the contract method 0x5da57fe9.
//
// Solidity: function createInitialBitfield(uint256[] bitsToSet, uint256 length) pure returns(uint256[])
func (_Contract *ContractCaller) CreateInitialBitfield(opts *bind.CallOpts, bitsToSet []*big.Int, length *big.Int) ([]*big.Int, error) {
	var out []interface{}
	err := _Contract.contract.Call(opts, &out, "createInitialBitfield", bitsToSet, length)

	if err != nil {
		return *new([]*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new([]*big.Int)).(*[]*big.Int)

	return out0, err

}

// CreateInitialBitfield is a free data retrieval call binding the contract method 0x5da57fe9.
//
// Solidity: function createInitialBitfield(uint256[] bitsToSet, uint256 length) pure returns(uint256[])
func (_Contract *ContractSession) CreateInitialBitfield(bitsToSet []*big.Int, length *big.Int) ([]*big.Int, error) {
	return _Contract.Contract.CreateInitialBitfield(&_Contract.CallOpts, bitsToSet, length)
}

// CreateInitialBitfield is a free data retrieval call binding the contract method 0x5da57fe9.
//
// Solidity: function createInitialBitfield(uint256[] bitsToSet, uint256 length) pure returns(uint256[])
func (_Contract *ContractCallerSession) CreateInitialBitfield(bitsToSet []*big.Int, length *big.Int) ([]*big.Int, error) {
	return _Contract.Contract.CreateInitialBitfield(&_Contract.CallOpts, bitsToSet, length)
}

// CreateRandomBitfield is a free data retrieval call binding the contract method 0x92848016.
//
// Solidity: function createRandomBitfield(uint256 id) view returns(uint256[])
func (_Contract *ContractCaller) CreateRandomBitfield(opts *bind.CallOpts, id *big.Int) ([]*big.Int, error) {
	var out []interface{}
	err := _Contract.contract.Call(opts, &out, "createRandomBitfield", id)

	if err != nil {
		return *new([]*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new([]*big.Int)).(*[]*big.Int)

	return out0, err

}

// CreateRandomBitfield is a free data retrieval call binding the contract method 0x92848016.
//
// Solidity: function createRandomBitfield(uint256 id) view returns(uint256[])
func (_Contract *ContractSession) CreateRandomBitfield(id *big.Int) ([]*big.Int, error) {
	return _Contract.Contract.CreateRandomBitfield(&_Contract.CallOpts, id)
}

// CreateRandomBitfield is a free data retrieval call binding the contract method 0x92848016.
//
// Solidity: function createRandomBitfield(uint256 id) view returns(uint256[])
func (_Contract *ContractCallerSession) CreateRandomBitfield(id *big.Int) ([]*big.Int, error) {
	return _Contract.Contract.CreateRandomBitfield(&_Contract.CallOpts, id)
}

// CurrentValidatorSet is a free data retrieval call binding the contract method 0x2cdea717.
//
// Solidity: function currentValidatorSet() view returns(uint256 id, bytes32 root, uint256 length)
func (_Contract *ContractCaller) CurrentValidatorSet(opts *bind.CallOpts) (struct {
	Id     *big.Int
	Root   [32]byte
	Length *big.Int
}, error) {
	var out []interface{}
	err := _Contract.contract.Call(opts, &out, "currentValidatorSet")

	outstruct := new(struct {
		Id     *big.Int
		Root   [32]byte
		Length *big.Int
	})
	if err != nil {
		return *outstruct, err
	}

	outstruct.Id = *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)
	outstruct.Root = *abi.ConvertType(out[1], new([32]byte)).(*[32]byte)
	outstruct.Length = *abi.ConvertType(out[2], new(*big.Int)).(**big.Int)

	return *outstruct, err

}

// CurrentValidatorSet is a free data retrieval call binding the contract method 0x2cdea717.
//
// Solidity: function currentValidatorSet() view returns(uint256 id, bytes32 root, uint256 length)
func (_Contract *ContractSession) CurrentValidatorSet() (struct {
	Id     *big.Int
	Root   [32]byte
	Length *big.Int
}, error) {
	return _Contract.Contract.CurrentValidatorSet(&_Contract.CallOpts)
}

// CurrentValidatorSet is a free data retrieval call binding the contract method 0x2cdea717.
//
// Solidity: function currentValidatorSet() view returns(uint256 id, bytes32 root, uint256 length)
func (_Contract *ContractCallerSession) CurrentValidatorSet() (struct {
	Id     *big.Int
	Root   [32]byte
	Length *big.Int
}, error) {
	return _Contract.Contract.CurrentValidatorSet(&_Contract.CallOpts)
}

// GetRoleAdmin is a free data retrieval call binding the contract method 0x248a9ca3.
//
// Solidity: function getRoleAdmin(bytes32 role) view returns(bytes32)
func (_Contract *ContractCaller) GetRoleAdmin(opts *bind.CallOpts, role [32]byte) ([32]byte, error) {
	var out []interface{}
	err := _Contract.contract.Call(opts, &out, "getRoleAdmin", role)

	if err != nil {
		return *new([32]byte), err
	}

	out0 := *abi.ConvertType(out[0], new([32]byte)).(*[32]byte)

	return out0, err

}

// GetRoleAdmin is a free data retrieval call binding the contract method 0x248a9ca3.
//
// Solidity: function getRoleAdmin(bytes32 role) view returns(bytes32)
func (_Contract *ContractSession) GetRoleAdmin(role [32]byte) ([32]byte, error) {
	return _Contract.Contract.GetRoleAdmin(&_Contract.CallOpts, role)
}

// GetRoleAdmin is a free data retrieval call binding the contract method 0x248a9ca3.
//
// Solidity: function getRoleAdmin(bytes32 role) view returns(bytes32)
func (_Contract *ContractCallerSession) GetRoleAdmin(role [32]byte) ([32]byte, error) {
	return _Contract.Contract.GetRoleAdmin(&_Contract.CallOpts, role)
}

// HasRole is a free data retrieval call binding the contract method 0x91d14854.
//
// Solidity: function hasRole(bytes32 role, address account) view returns(bool)
func (_Contract *ContractCaller) HasRole(opts *bind.CallOpts, role [32]byte, account common.Address) (bool, error) {
	var out []interface{}
	err := _Contract.contract.Call(opts, &out, "hasRole", role, account)

	if err != nil {
		return *new(bool), err
	}

	out0 := *abi.ConvertType(out[0], new(bool)).(*bool)

	return out0, err

}

// HasRole is a free data retrieval call binding the contract method 0x91d14854.
//
// Solidity: function hasRole(bytes32 role, address account) view returns(bool)
func (_Contract *ContractSession) HasRole(role [32]byte, account common.Address) (bool, error) {
	return _Contract.Contract.HasRole(&_Contract.CallOpts, role, account)
}

// HasRole is a free data retrieval call binding the contract method 0x91d14854.
//
// Solidity: function hasRole(bytes32 role, address account) view returns(bool)
func (_Contract *ContractCallerSession) HasRole(role [32]byte, account common.Address) (bool, error) {
	return _Contract.Contract.HasRole(&_Contract.CallOpts, role, account)
}

// IsValidatorInSet is a free data retrieval call binding the contract method 0x983c76bf.
//
// Solidity: function isValidatorInSet((uint256,bytes32,uint256) vset, address addr, uint256 pos, bytes32[] proof) view returns(bool)
func (_Contract *ContractCaller) IsValidatorInSet(opts *bind.CallOpts, vset BeefyLightClientValidatorSet, addr common.Address, pos *big.Int, proof [][32]byte) (bool, error) {
	var out []interface{}
	err := _Contract.contract.Call(opts, &out, "isValidatorInSet", vset, addr, pos, proof)

	if err != nil {
		return *new(bool), err
	}

	out0 := *abi.ConvertType(out[0], new(bool)).(*bool)

	return out0, err

}

// IsValidatorInSet is a free data retrieval call binding the contract method 0x983c76bf.
//
// Solidity: function isValidatorInSet((uint256,bytes32,uint256) vset, address addr, uint256 pos, bytes32[] proof) view returns(bool)
func (_Contract *ContractSession) IsValidatorInSet(vset BeefyLightClientValidatorSet, addr common.Address, pos *big.Int, proof [][32]byte) (bool, error) {
	return _Contract.Contract.IsValidatorInSet(&_Contract.CallOpts, vset, addr, pos, proof)
}

// IsValidatorInSet is a free data retrieval call binding the contract method 0x983c76bf.
//
// Solidity: function isValidatorInSet((uint256,bytes32,uint256) vset, address addr, uint256 pos, bytes32[] proof) view returns(bool)
func (_Contract *ContractCallerSession) IsValidatorInSet(vset BeefyLightClientValidatorSet, addr common.Address, pos *big.Int, proof [][32]byte) (bool, error) {
	return _Contract.Contract.IsValidatorInSet(&_Contract.CallOpts, vset, addr, pos, proof)
}

// LatestBeefyBlock is a free data retrieval call binding the contract method 0x66ae69a0.
//
// Solidity: function latestBeefyBlock() view returns(uint64)
func (_Contract *ContractCaller) LatestBeefyBlock(opts *bind.CallOpts) (uint64, error) {
	var out []interface{}
	err := _Contract.contract.Call(opts, &out, "latestBeefyBlock")

	if err != nil {
		return *new(uint64), err
	}

	out0 := *abi.ConvertType(out[0], new(uint64)).(*uint64)

	return out0, err

}

// LatestBeefyBlock is a free data retrieval call binding the contract method 0x66ae69a0.
//
// Solidity: function latestBeefyBlock() view returns(uint64)
func (_Contract *ContractSession) LatestBeefyBlock() (uint64, error) {
	return _Contract.Contract.LatestBeefyBlock(&_Contract.CallOpts)
}

// LatestBeefyBlock is a free data retrieval call binding the contract method 0x66ae69a0.
//
// Solidity: function latestBeefyBlock() view returns(uint64)
func (_Contract *ContractCallerSession) LatestBeefyBlock() (uint64, error) {
	return _Contract.Contract.LatestBeefyBlock(&_Contract.CallOpts)
}

// LatestMMRRoot is a free data retrieval call binding the contract method 0x41c9634e.
//
// Solidity: function latestMMRRoot() view returns(bytes32)
func (_Contract *ContractCaller) LatestMMRRoot(opts *bind.CallOpts) ([32]byte, error) {
	var out []interface{}
	err := _Contract.contract.Call(opts, &out, "latestMMRRoot")

	if err != nil {
		return *new([32]byte), err
	}

	out0 := *abi.ConvertType(out[0], new([32]byte)).(*[32]byte)

	return out0, err

}

// LatestMMRRoot is a free data retrieval call binding the contract method 0x41c9634e.
//
// Solidity: function latestMMRRoot() view returns(bytes32)
func (_Contract *ContractSession) LatestMMRRoot() ([32]byte, error) {
	return _Contract.Contract.LatestMMRRoot(&_Contract.CallOpts)
}

// LatestMMRRoot is a free data retrieval call binding the contract method 0x41c9634e.
//
// Solidity: function latestMMRRoot() view returns(bytes32)
func (_Contract *ContractCallerSession) LatestMMRRoot() ([32]byte, error) {
	return _Contract.Contract.LatestMMRRoot(&_Contract.CallOpts)
}

// MmrVerification is a free data retrieval call binding the contract method 0x801ed1e3.
//
// Solidity: function mmrVerification() view returns(address)
func (_Contract *ContractCaller) MmrVerification(opts *bind.CallOpts) (common.Address, error) {
	var out []interface{}
	err := _Contract.contract.Call(opts, &out, "mmrVerification")

	if err != nil {
		return *new(common.Address), err
	}

	out0 := *abi.ConvertType(out[0], new(common.Address)).(*common.Address)

	return out0, err

}

// MmrVerification is a free data retrieval call binding the contract method 0x801ed1e3.
//
// Solidity: function mmrVerification() view returns(address)
func (_Contract *ContractSession) MmrVerification() (common.Address, error) {
	return _Contract.Contract.MmrVerification(&_Contract.CallOpts)
}

// MmrVerification is a free data retrieval call binding the contract method 0x801ed1e3.
//
// Solidity: function mmrVerification() view returns(address)
func (_Contract *ContractCallerSession) MmrVerification() (common.Address, error) {
	return _Contract.Contract.MmrVerification(&_Contract.CallOpts)
}

// NextID is a free data retrieval call binding the contract method 0x1e96917d.
//
// Solidity: function nextID() view returns(uint256)
func (_Contract *ContractCaller) NextID(opts *bind.CallOpts) (*big.Int, error) {
	var out []interface{}
	err := _Contract.contract.Call(opts, &out, "nextID")

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// NextID is a free data retrieval call binding the contract method 0x1e96917d.
//
// Solidity: function nextID() view returns(uint256)
func (_Contract *ContractSession) NextID() (*big.Int, error) {
	return _Contract.Contract.NextID(&_Contract.CallOpts)
}

// NextID is a free data retrieval call binding the contract method 0x1e96917d.
//
// Solidity: function nextID() view returns(uint256)
func (_Contract *ContractCallerSession) NextID() (*big.Int, error) {
	return _Contract.Contract.NextID(&_Contract.CallOpts)
}

// NextValidatorSet is a free data retrieval call binding the contract method 0x36667513.
//
// Solidity: function nextValidatorSet() view returns(uint256 id, bytes32 root, uint256 length)
func (_Contract *ContractCaller) NextValidatorSet(opts *bind.CallOpts) (struct {
	Id     *big.Int
	Root   [32]byte
	Length *big.Int
}, error) {
	var out []interface{}
	err := _Contract.contract.Call(opts, &out, "nextValidatorSet")

	outstruct := new(struct {
		Id     *big.Int
		Root   [32]byte
		Length *big.Int
	})
	if err != nil {
		return *outstruct, err
	}

	outstruct.Id = *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)
	outstruct.Root = *abi.ConvertType(out[1], new([32]byte)).(*[32]byte)
	outstruct.Length = *abi.ConvertType(out[2], new(*big.Int)).(**big.Int)

	return *outstruct, err

}

// NextValidatorSet is a free data retrieval call binding the contract method 0x36667513.
//
// Solidity: function nextValidatorSet() view returns(uint256 id, bytes32 root, uint256 length)
func (_Contract *ContractSession) NextValidatorSet() (struct {
	Id     *big.Int
	Root   [32]byte
	Length *big.Int
}, error) {
	return _Contract.Contract.NextValidatorSet(&_Contract.CallOpts)
}

// NextValidatorSet is a free data retrieval call binding the contract method 0x36667513.
//
// Solidity: function nextValidatorSet() view returns(uint256 id, bytes32 root, uint256 length)
func (_Contract *ContractCallerSession) NextValidatorSet() (struct {
	Id     *big.Int
	Root   [32]byte
	Length *big.Int
}, error) {
	return _Contract.Contract.NextValidatorSet(&_Contract.CallOpts)
}

// SupportsInterface is a free data retrieval call binding the contract method 0x01ffc9a7.
//
// Solidity: function supportsInterface(bytes4 interfaceId) view returns(bool)
func (_Contract *ContractCaller) SupportsInterface(opts *bind.CallOpts, interfaceId [4]byte) (bool, error) {
	var out []interface{}
	err := _Contract.contract.Call(opts, &out, "supportsInterface", interfaceId)

	if err != nil {
		return *new(bool), err
	}

	out0 := *abi.ConvertType(out[0], new(bool)).(*bool)

	return out0, err

}

// SupportsInterface is a free data retrieval call binding the contract method 0x01ffc9a7.
//
// Solidity: function supportsInterface(bytes4 interfaceId) view returns(bool)
func (_Contract *ContractSession) SupportsInterface(interfaceId [4]byte) (bool, error) {
	return _Contract.Contract.SupportsInterface(&_Contract.CallOpts, interfaceId)
}

// SupportsInterface is a free data retrieval call binding the contract method 0x01ffc9a7.
//
// Solidity: function supportsInterface(bytes4 interfaceId) view returns(bool)
func (_Contract *ContractCallerSession) SupportsInterface(interfaceId [4]byte) (bool, error) {
	return _Contract.Contract.SupportsInterface(&_Contract.CallOpts, interfaceId)
}

// ValidationData is a free data retrieval call binding the contract method 0x20bfa5cb.
//
// Solidity: function validationData(uint256 ) view returns(address senderAddress, bytes32 commitmentHash, uint256 validatorSetID, uint256 blockNumber)
func (_Contract *ContractCaller) ValidationData(opts *bind.CallOpts, arg0 *big.Int) (struct {
	SenderAddress  common.Address
	CommitmentHash [32]byte
	ValidatorSetID *big.Int
	BlockNumber    *big.Int
}, error) {
	var out []interface{}
	err := _Contract.contract.Call(opts, &out, "validationData", arg0)

	outstruct := new(struct {
		SenderAddress  common.Address
		CommitmentHash [32]byte
		ValidatorSetID *big.Int
		BlockNumber    *big.Int
	})
	if err != nil {
		return *outstruct, err
	}

	outstruct.SenderAddress = *abi.ConvertType(out[0], new(common.Address)).(*common.Address)
	outstruct.CommitmentHash = *abi.ConvertType(out[1], new([32]byte)).(*[32]byte)
	outstruct.ValidatorSetID = *abi.ConvertType(out[2], new(*big.Int)).(**big.Int)
	outstruct.BlockNumber = *abi.ConvertType(out[3], new(*big.Int)).(**big.Int)

	return *outstruct, err

}

// ValidationData is a free data retrieval call binding the contract method 0x20bfa5cb.
//
// Solidity: function validationData(uint256 ) view returns(address senderAddress, bytes32 commitmentHash, uint256 validatorSetID, uint256 blockNumber)
func (_Contract *ContractSession) ValidationData(arg0 *big.Int) (struct {
	SenderAddress  common.Address
	CommitmentHash [32]byte
	ValidatorSetID *big.Int
	BlockNumber    *big.Int
}, error) {
	return _Contract.Contract.ValidationData(&_Contract.CallOpts, arg0)
}

// ValidationData is a free data retrieval call binding the contract method 0x20bfa5cb.
//
// Solidity: function validationData(uint256 ) view returns(address senderAddress, bytes32 commitmentHash, uint256 validatorSetID, uint256 blockNumber)
func (_Contract *ContractCallerSession) ValidationData(arg0 *big.Int) (struct {
	SenderAddress  common.Address
	CommitmentHash [32]byte
	ValidatorSetID *big.Int
	BlockNumber    *big.Int
}, error) {
	return _Contract.Contract.ValidationData(&_Contract.CallOpts, arg0)
}

// VerifyBeefyMerkleLeaf is a free data retrieval call binding the contract method 0x2d268c9e.
//
// Solidity: function verifyBeefyMerkleLeaf(bytes32 beefyMMRLeaf, (bytes32[],uint64) proof) view returns(bool)
func (_Contract *ContractCaller) VerifyBeefyMerkleLeaf(opts *bind.CallOpts, beefyMMRLeaf [32]byte, proof SimplifiedMMRProof) (bool, error) {
	var out []interface{}
	err := _Contract.contract.Call(opts, &out, "verifyBeefyMerkleLeaf", beefyMMRLeaf, proof)

	if err != nil {
		return *new(bool), err
	}

	out0 := *abi.ConvertType(out[0], new(bool)).(*bool)

	return out0, err

}

// VerifyBeefyMerkleLeaf is a free data retrieval call binding the contract method 0x2d268c9e.
//
// Solidity: function verifyBeefyMerkleLeaf(bytes32 beefyMMRLeaf, (bytes32[],uint64) proof) view returns(bool)
func (_Contract *ContractSession) VerifyBeefyMerkleLeaf(beefyMMRLeaf [32]byte, proof SimplifiedMMRProof) (bool, error) {
	return _Contract.Contract.VerifyBeefyMerkleLeaf(&_Contract.CallOpts, beefyMMRLeaf, proof)
}

// VerifyBeefyMerkleLeaf is a free data retrieval call binding the contract method 0x2d268c9e.
//
// Solidity: function verifyBeefyMerkleLeaf(bytes32 beefyMMRLeaf, (bytes32[],uint64) proof) view returns(bool)
func (_Contract *ContractCallerSession) VerifyBeefyMerkleLeaf(beefyMMRLeaf [32]byte, proof SimplifiedMMRProof) (bool, error) {
	return _Contract.Contract.VerifyBeefyMerkleLeaf(&_Contract.CallOpts, beefyMMRLeaf, proof)
}

// CompleteSignatureCommitment is a paid mutator transaction binding the contract method 0x702d5d09.
//
// Solidity: function completeSignatureCommitment(uint256 id, (uint32,uint64,(bytes32,bytes,bytes)) commitment, (bytes[],uint256[],address[],bytes32[][]) validatorProof, (uint8,uint32,bytes32,uint64,uint32,bytes32,bytes32) leaf, (bytes32[],uint64) leafProof) returns()
func (_Contract *ContractTransactor) CompleteSignatureCommitment(opts *bind.TransactOpts, id *big.Int, commitment BeefyLightClientCommitment, validatorProof BeefyLightClientValidatorProof, leaf BeefyLightClientMMRLeaf, leafProof SimplifiedMMRProof) (*types.Transaction, error) {
	return _Contract.contract.Transact(opts, "completeSignatureCommitment", id, commitment, validatorProof, leaf, leafProof)
}

// CompleteSignatureCommitment is a paid mutator transaction binding the contract method 0x702d5d09.
//
// Solidity: function completeSignatureCommitment(uint256 id, (uint32,uint64,(bytes32,bytes,bytes)) commitment, (bytes[],uint256[],address[],bytes32[][]) validatorProof, (uint8,uint32,bytes32,uint64,uint32,bytes32,bytes32) leaf, (bytes32[],uint64) leafProof) returns()
func (_Contract *ContractSession) CompleteSignatureCommitment(id *big.Int, commitment BeefyLightClientCommitment, validatorProof BeefyLightClientValidatorProof, leaf BeefyLightClientMMRLeaf, leafProof SimplifiedMMRProof) (*types.Transaction, error) {
	return _Contract.Contract.CompleteSignatureCommitment(&_Contract.TransactOpts, id, commitment, validatorProof, leaf, leafProof)
}

// CompleteSignatureCommitment is a paid mutator transaction binding the contract method 0x702d5d09.
//
// Solidity: function completeSignatureCommitment(uint256 id, (uint32,uint64,(bytes32,bytes,bytes)) commitment, (bytes[],uint256[],address[],bytes32[][]) validatorProof, (uint8,uint32,bytes32,uint64,uint32,bytes32,bytes32) leaf, (bytes32[],uint64) leafProof) returns()
func (_Contract *ContractTransactorSession) CompleteSignatureCommitment(id *big.Int, commitment BeefyLightClientCommitment, validatorProof BeefyLightClientValidatorProof, leaf BeefyLightClientMMRLeaf, leafProof SimplifiedMMRProof) (*types.Transaction, error) {
	return _Contract.Contract.CompleteSignatureCommitment(&_Contract.TransactOpts, id, commitment, validatorProof, leaf, leafProof)
}

// GrantRole is a paid mutator transaction binding the contract method 0x2f2ff15d.
//
// Solidity: function grantRole(bytes32 role, address account) returns()
func (_Contract *ContractTransactor) GrantRole(opts *bind.TransactOpts, role [32]byte, account common.Address) (*types.Transaction, error) {
	return _Contract.contract.Transact(opts, "grantRole", role, account)
}

// GrantRole is a paid mutator transaction binding the contract method 0x2f2ff15d.
//
// Solidity: function grantRole(bytes32 role, address account) returns()
func (_Contract *ContractSession) GrantRole(role [32]byte, account common.Address) (*types.Transaction, error) {
	return _Contract.Contract.GrantRole(&_Contract.TransactOpts, role, account)
}

// GrantRole is a paid mutator transaction binding the contract method 0x2f2ff15d.
//
// Solidity: function grantRole(bytes32 role, address account) returns()
func (_Contract *ContractTransactorSession) GrantRole(role [32]byte, account common.Address) (*types.Transaction, error) {
	return _Contract.Contract.GrantRole(&_Contract.TransactOpts, role, account)
}

// Initialize is a paid mutator transaction binding the contract method 0x3795ea5f.
//
// Solidity: function initialize(uint64 _startingBeefyBlock, (uint256,bytes32,uint256) _initialValidatorSet, (uint256,bytes32,uint256) _nextValidatorSet) returns()
func (_Contract *ContractTransactor) Initialize(opts *bind.TransactOpts, _startingBeefyBlock uint64, _initialValidatorSet BeefyLightClientValidatorSet, _nextValidatorSet BeefyLightClientValidatorSet) (*types.Transaction, error) {
	return _Contract.contract.Transact(opts, "initialize", _startingBeefyBlock, _initialValidatorSet, _nextValidatorSet)
}

// Initialize is a paid mutator transaction binding the contract method 0x3795ea5f.
//
// Solidity: function initialize(uint64 _startingBeefyBlock, (uint256,bytes32,uint256) _initialValidatorSet, (uint256,bytes32,uint256) _nextValidatorSet) returns()
func (_Contract *ContractSession) Initialize(_startingBeefyBlock uint64, _initialValidatorSet BeefyLightClientValidatorSet, _nextValidatorSet BeefyLightClientValidatorSet) (*types.Transaction, error) {
	return _Contract.Contract.Initialize(&_Contract.TransactOpts, _startingBeefyBlock, _initialValidatorSet, _nextValidatorSet)
}

// Initialize is a paid mutator transaction binding the contract method 0x3795ea5f.
//
// Solidity: function initialize(uint64 _startingBeefyBlock, (uint256,bytes32,uint256) _initialValidatorSet, (uint256,bytes32,uint256) _nextValidatorSet) returns()
func (_Contract *ContractTransactorSession) Initialize(_startingBeefyBlock uint64, _initialValidatorSet BeefyLightClientValidatorSet, _nextValidatorSet BeefyLightClientValidatorSet) (*types.Transaction, error) {
	return _Contract.Contract.Initialize(&_Contract.TransactOpts, _startingBeefyBlock, _initialValidatorSet, _nextValidatorSet)
}

// NewSignatureCommitment is a paid mutator transaction binding the contract method 0xe8b1d414.
//
// Solidity: function newSignatureCommitment(bytes32 commitmentHash, uint64 validatorSetID, uint256[] validatorClaimsBitfield, bytes validatorSignature, uint256 validatorPosition, address validatorPublicKey, bytes32[] validatorPublicKeyMerkleProof) payable returns()
func (_Contract *ContractTransactor) NewSignatureCommitment(opts *bind.TransactOpts, commitmentHash [32]byte, validatorSetID uint64, validatorClaimsBitfield []*big.Int, validatorSignature []byte, validatorPosition *big.Int, validatorPublicKey common.Address, validatorPublicKeyMerkleProof [][32]byte) (*types.Transaction, error) {
	return _Contract.contract.Transact(opts, "newSignatureCommitment", commitmentHash, validatorSetID, validatorClaimsBitfield, validatorSignature, validatorPosition, validatorPublicKey, validatorPublicKeyMerkleProof)
}

// NewSignatureCommitment is a paid mutator transaction binding the contract method 0xe8b1d414.
//
// Solidity: function newSignatureCommitment(bytes32 commitmentHash, uint64 validatorSetID, uint256[] validatorClaimsBitfield, bytes validatorSignature, uint256 validatorPosition, address validatorPublicKey, bytes32[] validatorPublicKeyMerkleProof) payable returns()
func (_Contract *ContractSession) NewSignatureCommitment(commitmentHash [32]byte, validatorSetID uint64, validatorClaimsBitfield []*big.Int, validatorSignature []byte, validatorPosition *big.Int, validatorPublicKey common.Address, validatorPublicKeyMerkleProof [][32]byte) (*types.Transaction, error) {
	return _Contract.Contract.NewSignatureCommitment(&_Contract.TransactOpts, commitmentHash, validatorSetID, validatorClaimsBitfield, validatorSignature, validatorPosition, validatorPublicKey, validatorPublicKeyMerkleProof)
}

// NewSignatureCommitment is a paid mutator transaction binding the contract method 0xe8b1d414.
//
// Solidity: function newSignatureCommitment(bytes32 commitmentHash, uint64 validatorSetID, uint256[] validatorClaimsBitfield, bytes validatorSignature, uint256 validatorPosition, address validatorPublicKey, bytes32[] validatorPublicKeyMerkleProof) payable returns()
func (_Contract *ContractTransactorSession) NewSignatureCommitment(commitmentHash [32]byte, validatorSetID uint64, validatorClaimsBitfield []*big.Int, validatorSignature []byte, validatorPosition *big.Int, validatorPublicKey common.Address, validatorPublicKeyMerkleProof [][32]byte) (*types.Transaction, error) {
	return _Contract.Contract.NewSignatureCommitment(&_Contract.TransactOpts, commitmentHash, validatorSetID, validatorClaimsBitfield, validatorSignature, validatorPosition, validatorPublicKey, validatorPublicKeyMerkleProof)
}

// RenounceRole is a paid mutator transaction binding the contract method 0x36568abe.
//
// Solidity: function renounceRole(bytes32 role, address account) returns()
func (_Contract *ContractTransactor) RenounceRole(opts *bind.TransactOpts, role [32]byte, account common.Address) (*types.Transaction, error) {
	return _Contract.contract.Transact(opts, "renounceRole", role, account)
}

// RenounceRole is a paid mutator transaction binding the contract method 0x36568abe.
//
// Solidity: function renounceRole(bytes32 role, address account) returns()
func (_Contract *ContractSession) RenounceRole(role [32]byte, account common.Address) (*types.Transaction, error) {
	return _Contract.Contract.RenounceRole(&_Contract.TransactOpts, role, account)
}

// RenounceRole is a paid mutator transaction binding the contract method 0x36568abe.
//
// Solidity: function renounceRole(bytes32 role, address account) returns()
func (_Contract *ContractTransactorSession) RenounceRole(role [32]byte, account common.Address) (*types.Transaction, error) {
	return _Contract.Contract.RenounceRole(&_Contract.TransactOpts, role, account)
}

// RevokeRole is a paid mutator transaction binding the contract method 0xd547741f.
//
// Solidity: function revokeRole(bytes32 role, address account) returns()
func (_Contract *ContractTransactor) RevokeRole(opts *bind.TransactOpts, role [32]byte, account common.Address) (*types.Transaction, error) {
	return _Contract.contract.Transact(opts, "revokeRole", role, account)
}

// RevokeRole is a paid mutator transaction binding the contract method 0xd547741f.
//
// Solidity: function revokeRole(bytes32 role, address account) returns()
func (_Contract *ContractSession) RevokeRole(role [32]byte, account common.Address) (*types.Transaction, error) {
	return _Contract.Contract.RevokeRole(&_Contract.TransactOpts, role, account)
}

// RevokeRole is a paid mutator transaction binding the contract method 0xd547741f.
//
// Solidity: function revokeRole(bytes32 role, address account) returns()
func (_Contract *ContractTransactorSession) RevokeRole(role [32]byte, account common.Address) (*types.Transaction, error) {
	return _Contract.Contract.RevokeRole(&_Contract.TransactOpts, role, account)
}

// ContractFinalVerificationSuccessfulIterator is returned from FilterFinalVerificationSuccessful and is used to iterate over the raw logs and unpacked data for FinalVerificationSuccessful events raised by the Contract contract.
type ContractFinalVerificationSuccessfulIterator struct {
	Event *ContractFinalVerificationSuccessful // Event containing the contract specifics and raw log

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
func (it *ContractFinalVerificationSuccessfulIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ContractFinalVerificationSuccessful)
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
		it.Event = new(ContractFinalVerificationSuccessful)
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
func (it *ContractFinalVerificationSuccessfulIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ContractFinalVerificationSuccessfulIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ContractFinalVerificationSuccessful represents a FinalVerificationSuccessful event raised by the Contract contract.
type ContractFinalVerificationSuccessful struct {
	Prover common.Address
	Id     *big.Int
	Raw    types.Log // Blockchain specific contextual infos
}

// FilterFinalVerificationSuccessful is a free log retrieval operation binding the contract event 0xc128224dd1747f24cc4ecd95248a78fe3b1960b100c9d08ba578888bac274c4e.
//
// Solidity: event FinalVerificationSuccessful(address prover, uint256 id)
func (_Contract *ContractFilterer) FilterFinalVerificationSuccessful(opts *bind.FilterOpts) (*ContractFinalVerificationSuccessfulIterator, error) {

	logs, sub, err := _Contract.contract.FilterLogs(opts, "FinalVerificationSuccessful")
	if err != nil {
		return nil, err
	}
	return &ContractFinalVerificationSuccessfulIterator{contract: _Contract.contract, event: "FinalVerificationSuccessful", logs: logs, sub: sub}, nil
}

// WatchFinalVerificationSuccessful is a free log subscription operation binding the contract event 0xc128224dd1747f24cc4ecd95248a78fe3b1960b100c9d08ba578888bac274c4e.
//
// Solidity: event FinalVerificationSuccessful(address prover, uint256 id)
func (_Contract *ContractFilterer) WatchFinalVerificationSuccessful(opts *bind.WatchOpts, sink chan<- *ContractFinalVerificationSuccessful) (event.Subscription, error) {

	logs, sub, err := _Contract.contract.WatchLogs(opts, "FinalVerificationSuccessful")
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ContractFinalVerificationSuccessful)
				if err := _Contract.contract.UnpackLog(event, "FinalVerificationSuccessful", log); err != nil {
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

// ParseFinalVerificationSuccessful is a log parse operation binding the contract event 0xc128224dd1747f24cc4ecd95248a78fe3b1960b100c9d08ba578888bac274c4e.
//
// Solidity: event FinalVerificationSuccessful(address prover, uint256 id)
func (_Contract *ContractFilterer) ParseFinalVerificationSuccessful(log types.Log) (*ContractFinalVerificationSuccessful, error) {
	event := new(ContractFinalVerificationSuccessful)
	if err := _Contract.contract.UnpackLog(event, "FinalVerificationSuccessful", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// ContractInitialVerificationSuccessfulIterator is returned from FilterInitialVerificationSuccessful and is used to iterate over the raw logs and unpacked data for InitialVerificationSuccessful events raised by the Contract contract.
type ContractInitialVerificationSuccessfulIterator struct {
	Event *ContractInitialVerificationSuccessful // Event containing the contract specifics and raw log

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
func (it *ContractInitialVerificationSuccessfulIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ContractInitialVerificationSuccessful)
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
		it.Event = new(ContractInitialVerificationSuccessful)
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
func (it *ContractInitialVerificationSuccessfulIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ContractInitialVerificationSuccessfulIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ContractInitialVerificationSuccessful represents a InitialVerificationSuccessful event raised by the Contract contract.
type ContractInitialVerificationSuccessful struct {
	Prover      common.Address
	BlockNumber *big.Int
	Id          *big.Int
	Raw         types.Log // Blockchain specific contextual infos
}

// FilterInitialVerificationSuccessful is a free log retrieval operation binding the contract event 0xf93e67b44fe47465ca1478dbc20efe59702e2fff4b8beecf053817d7ee29fd55.
//
// Solidity: event InitialVerificationSuccessful(address prover, uint256 blockNumber, uint256 id)
func (_Contract *ContractFilterer) FilterInitialVerificationSuccessful(opts *bind.FilterOpts) (*ContractInitialVerificationSuccessfulIterator, error) {

	logs, sub, err := _Contract.contract.FilterLogs(opts, "InitialVerificationSuccessful")
	if err != nil {
		return nil, err
	}
	return &ContractInitialVerificationSuccessfulIterator{contract: _Contract.contract, event: "InitialVerificationSuccessful", logs: logs, sub: sub}, nil
}

// WatchInitialVerificationSuccessful is a free log subscription operation binding the contract event 0xf93e67b44fe47465ca1478dbc20efe59702e2fff4b8beecf053817d7ee29fd55.
//
// Solidity: event InitialVerificationSuccessful(address prover, uint256 blockNumber, uint256 id)
func (_Contract *ContractFilterer) WatchInitialVerificationSuccessful(opts *bind.WatchOpts, sink chan<- *ContractInitialVerificationSuccessful) (event.Subscription, error) {

	logs, sub, err := _Contract.contract.WatchLogs(opts, "InitialVerificationSuccessful")
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ContractInitialVerificationSuccessful)
				if err := _Contract.contract.UnpackLog(event, "InitialVerificationSuccessful", log); err != nil {
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

// ParseInitialVerificationSuccessful is a log parse operation binding the contract event 0xf93e67b44fe47465ca1478dbc20efe59702e2fff4b8beecf053817d7ee29fd55.
//
// Solidity: event InitialVerificationSuccessful(address prover, uint256 blockNumber, uint256 id)
func (_Contract *ContractFilterer) ParseInitialVerificationSuccessful(log types.Log) (*ContractInitialVerificationSuccessful, error) {
	event := new(ContractInitialVerificationSuccessful)
	if err := _Contract.contract.UnpackLog(event, "InitialVerificationSuccessful", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// ContractNewMMRRootIterator is returned from FilterNewMMRRoot and is used to iterate over the raw logs and unpacked data for NewMMRRoot events raised by the Contract contract.
type ContractNewMMRRootIterator struct {
	Event *ContractNewMMRRoot // Event containing the contract specifics and raw log

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
func (it *ContractNewMMRRootIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ContractNewMMRRoot)
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
		it.Event = new(ContractNewMMRRoot)
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
func (it *ContractNewMMRRootIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ContractNewMMRRootIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ContractNewMMRRoot represents a NewMMRRoot event raised by the Contract contract.
type ContractNewMMRRoot struct {
	MmrRoot     [32]byte
	BlockNumber uint64
	Raw         types.Log // Blockchain specific contextual infos
}

// FilterNewMMRRoot is a free log retrieval operation binding the contract event 0xd95fe1258d152dc91c81b09380498adc76ed36a6079bcb2ed31eff622ae2d0f1.
//
// Solidity: event NewMMRRoot(bytes32 mmrRoot, uint64 blockNumber)
func (_Contract *ContractFilterer) FilterNewMMRRoot(opts *bind.FilterOpts) (*ContractNewMMRRootIterator, error) {

	logs, sub, err := _Contract.contract.FilterLogs(opts, "NewMMRRoot")
	if err != nil {
		return nil, err
	}
	return &ContractNewMMRRootIterator{contract: _Contract.contract, event: "NewMMRRoot", logs: logs, sub: sub}, nil
}

// WatchNewMMRRoot is a free log subscription operation binding the contract event 0xd95fe1258d152dc91c81b09380498adc76ed36a6079bcb2ed31eff622ae2d0f1.
//
// Solidity: event NewMMRRoot(bytes32 mmrRoot, uint64 blockNumber)
func (_Contract *ContractFilterer) WatchNewMMRRoot(opts *bind.WatchOpts, sink chan<- *ContractNewMMRRoot) (event.Subscription, error) {

	logs, sub, err := _Contract.contract.WatchLogs(opts, "NewMMRRoot")
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ContractNewMMRRoot)
				if err := _Contract.contract.UnpackLog(event, "NewMMRRoot", log); err != nil {
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

// ParseNewMMRRoot is a log parse operation binding the contract event 0xd95fe1258d152dc91c81b09380498adc76ed36a6079bcb2ed31eff622ae2d0f1.
//
// Solidity: event NewMMRRoot(bytes32 mmrRoot, uint64 blockNumber)
func (_Contract *ContractFilterer) ParseNewMMRRoot(log types.Log) (*ContractNewMMRRoot, error) {
	event := new(ContractNewMMRRoot)
	if err := _Contract.contract.UnpackLog(event, "NewMMRRoot", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// ContractNewSessionIterator is returned from FilterNewSession and is used to iterate over the raw logs and unpacked data for NewSession events raised by the Contract contract.
type ContractNewSessionIterator struct {
	Event *ContractNewSession // Event containing the contract specifics and raw log

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
func (it *ContractNewSessionIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ContractNewSession)
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
		it.Event = new(ContractNewSession)
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
func (it *ContractNewSessionIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ContractNewSessionIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ContractNewSession represents a NewSession event raised by the Contract contract.
type ContractNewSession struct {
	ValidatorSetID     *big.Int
	ValidatorSetRoot   [32]byte
	ValidatorSetLength *big.Int
	Raw                types.Log // Blockchain specific contextual infos
}

// FilterNewSession is a free log retrieval operation binding the contract event 0xbeb4fe60dd44342f351e69b05a20bc6557242b29c7847c23c6aa2040c1502570.
//
// Solidity: event NewSession(uint256 validatorSetID, bytes32 validatorSetRoot, uint256 validatorSetLength)
func (_Contract *ContractFilterer) FilterNewSession(opts *bind.FilterOpts) (*ContractNewSessionIterator, error) {

	logs, sub, err := _Contract.contract.FilterLogs(opts, "NewSession")
	if err != nil {
		return nil, err
	}
	return &ContractNewSessionIterator{contract: _Contract.contract, event: "NewSession", logs: logs, sub: sub}, nil
}

// WatchNewSession is a free log subscription operation binding the contract event 0xbeb4fe60dd44342f351e69b05a20bc6557242b29c7847c23c6aa2040c1502570.
//
// Solidity: event NewSession(uint256 validatorSetID, bytes32 validatorSetRoot, uint256 validatorSetLength)
func (_Contract *ContractFilterer) WatchNewSession(opts *bind.WatchOpts, sink chan<- *ContractNewSession) (event.Subscription, error) {

	logs, sub, err := _Contract.contract.WatchLogs(opts, "NewSession")
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ContractNewSession)
				if err := _Contract.contract.UnpackLog(event, "NewSession", log); err != nil {
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

// ParseNewSession is a log parse operation binding the contract event 0xbeb4fe60dd44342f351e69b05a20bc6557242b29c7847c23c6aa2040c1502570.
//
// Solidity: event NewSession(uint256 validatorSetID, bytes32 validatorSetRoot, uint256 validatorSetLength)
func (_Contract *ContractFilterer) ParseNewSession(log types.Log) (*ContractNewSession, error) {
	event := new(ContractNewSession)
	if err := _Contract.contract.UnpackLog(event, "NewSession", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// ContractRoleAdminChangedIterator is returned from FilterRoleAdminChanged and is used to iterate over the raw logs and unpacked data for RoleAdminChanged events raised by the Contract contract.
type ContractRoleAdminChangedIterator struct {
	Event *ContractRoleAdminChanged // Event containing the contract specifics and raw log

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
func (it *ContractRoleAdminChangedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ContractRoleAdminChanged)
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
		it.Event = new(ContractRoleAdminChanged)
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
func (it *ContractRoleAdminChangedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ContractRoleAdminChangedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ContractRoleAdminChanged represents a RoleAdminChanged event raised by the Contract contract.
type ContractRoleAdminChanged struct {
	Role              [32]byte
	PreviousAdminRole [32]byte
	NewAdminRole      [32]byte
	Raw               types.Log // Blockchain specific contextual infos
}

// FilterRoleAdminChanged is a free log retrieval operation binding the contract event 0xbd79b86ffe0ab8e8776151514217cd7cacd52c909f66475c3af44e129f0b00ff.
//
// Solidity: event RoleAdminChanged(bytes32 indexed role, bytes32 indexed previousAdminRole, bytes32 indexed newAdminRole)
func (_Contract *ContractFilterer) FilterRoleAdminChanged(opts *bind.FilterOpts, role [][32]byte, previousAdminRole [][32]byte, newAdminRole [][32]byte) (*ContractRoleAdminChangedIterator, error) {

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

	logs, sub, err := _Contract.contract.FilterLogs(opts, "RoleAdminChanged", roleRule, previousAdminRoleRule, newAdminRoleRule)
	if err != nil {
		return nil, err
	}
	return &ContractRoleAdminChangedIterator{contract: _Contract.contract, event: "RoleAdminChanged", logs: logs, sub: sub}, nil
}

// WatchRoleAdminChanged is a free log subscription operation binding the contract event 0xbd79b86ffe0ab8e8776151514217cd7cacd52c909f66475c3af44e129f0b00ff.
//
// Solidity: event RoleAdminChanged(bytes32 indexed role, bytes32 indexed previousAdminRole, bytes32 indexed newAdminRole)
func (_Contract *ContractFilterer) WatchRoleAdminChanged(opts *bind.WatchOpts, sink chan<- *ContractRoleAdminChanged, role [][32]byte, previousAdminRole [][32]byte, newAdminRole [][32]byte) (event.Subscription, error) {

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

	logs, sub, err := _Contract.contract.WatchLogs(opts, "RoleAdminChanged", roleRule, previousAdminRoleRule, newAdminRoleRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ContractRoleAdminChanged)
				if err := _Contract.contract.UnpackLog(event, "RoleAdminChanged", log); err != nil {
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
func (_Contract *ContractFilterer) ParseRoleAdminChanged(log types.Log) (*ContractRoleAdminChanged, error) {
	event := new(ContractRoleAdminChanged)
	if err := _Contract.contract.UnpackLog(event, "RoleAdminChanged", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// ContractRoleGrantedIterator is returned from FilterRoleGranted and is used to iterate over the raw logs and unpacked data for RoleGranted events raised by the Contract contract.
type ContractRoleGrantedIterator struct {
	Event *ContractRoleGranted // Event containing the contract specifics and raw log

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
func (it *ContractRoleGrantedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ContractRoleGranted)
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
		it.Event = new(ContractRoleGranted)
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
func (it *ContractRoleGrantedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ContractRoleGrantedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ContractRoleGranted represents a RoleGranted event raised by the Contract contract.
type ContractRoleGranted struct {
	Role    [32]byte
	Account common.Address
	Sender  common.Address
	Raw     types.Log // Blockchain specific contextual infos
}

// FilterRoleGranted is a free log retrieval operation binding the contract event 0x2f8788117e7eff1d82e926ec794901d17c78024a50270940304540a733656f0d.
//
// Solidity: event RoleGranted(bytes32 indexed role, address indexed account, address indexed sender)
func (_Contract *ContractFilterer) FilterRoleGranted(opts *bind.FilterOpts, role [][32]byte, account []common.Address, sender []common.Address) (*ContractRoleGrantedIterator, error) {

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

	logs, sub, err := _Contract.contract.FilterLogs(opts, "RoleGranted", roleRule, accountRule, senderRule)
	if err != nil {
		return nil, err
	}
	return &ContractRoleGrantedIterator{contract: _Contract.contract, event: "RoleGranted", logs: logs, sub: sub}, nil
}

// WatchRoleGranted is a free log subscription operation binding the contract event 0x2f8788117e7eff1d82e926ec794901d17c78024a50270940304540a733656f0d.
//
// Solidity: event RoleGranted(bytes32 indexed role, address indexed account, address indexed sender)
func (_Contract *ContractFilterer) WatchRoleGranted(opts *bind.WatchOpts, sink chan<- *ContractRoleGranted, role [][32]byte, account []common.Address, sender []common.Address) (event.Subscription, error) {

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

	logs, sub, err := _Contract.contract.WatchLogs(opts, "RoleGranted", roleRule, accountRule, senderRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ContractRoleGranted)
				if err := _Contract.contract.UnpackLog(event, "RoleGranted", log); err != nil {
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
func (_Contract *ContractFilterer) ParseRoleGranted(log types.Log) (*ContractRoleGranted, error) {
	event := new(ContractRoleGranted)
	if err := _Contract.contract.UnpackLog(event, "RoleGranted", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// ContractRoleRevokedIterator is returned from FilterRoleRevoked and is used to iterate over the raw logs and unpacked data for RoleRevoked events raised by the Contract contract.
type ContractRoleRevokedIterator struct {
	Event *ContractRoleRevoked // Event containing the contract specifics and raw log

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
func (it *ContractRoleRevokedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(ContractRoleRevoked)
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
		it.Event = new(ContractRoleRevoked)
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
func (it *ContractRoleRevokedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *ContractRoleRevokedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// ContractRoleRevoked represents a RoleRevoked event raised by the Contract contract.
type ContractRoleRevoked struct {
	Role    [32]byte
	Account common.Address
	Sender  common.Address
	Raw     types.Log // Blockchain specific contextual infos
}

// FilterRoleRevoked is a free log retrieval operation binding the contract event 0xf6391f5c32d9c69d2a47ea670b442974b53935d1edc7fd64eb21e047a839171b.
//
// Solidity: event RoleRevoked(bytes32 indexed role, address indexed account, address indexed sender)
func (_Contract *ContractFilterer) FilterRoleRevoked(opts *bind.FilterOpts, role [][32]byte, account []common.Address, sender []common.Address) (*ContractRoleRevokedIterator, error) {

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

	logs, sub, err := _Contract.contract.FilterLogs(opts, "RoleRevoked", roleRule, accountRule, senderRule)
	if err != nil {
		return nil, err
	}
	return &ContractRoleRevokedIterator{contract: _Contract.contract, event: "RoleRevoked", logs: logs, sub: sub}, nil
}

// WatchRoleRevoked is a free log subscription operation binding the contract event 0xf6391f5c32d9c69d2a47ea670b442974b53935d1edc7fd64eb21e047a839171b.
//
// Solidity: event RoleRevoked(bytes32 indexed role, address indexed account, address indexed sender)
func (_Contract *ContractFilterer) WatchRoleRevoked(opts *bind.WatchOpts, sink chan<- *ContractRoleRevoked, role [][32]byte, account []common.Address, sender []common.Address) (event.Subscription, error) {

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

	logs, sub, err := _Contract.contract.WatchLogs(opts, "RoleRevoked", roleRule, accountRule, senderRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(ContractRoleRevoked)
				if err := _Contract.contract.UnpackLog(event, "RoleRevoked", log); err != nil {
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
func (_Contract *ContractFilterer) ParseRoleRevoked(log types.Log) (*ContractRoleRevoked, error) {
	event := new(ContractRoleRevoked)
	if err := _Contract.contract.UnpackLog(event, "RoleRevoked", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}
