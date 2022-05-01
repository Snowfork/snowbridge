// Code generated - DO NOT EDIT.
// This file is a generated binding and any manual changes will be lost.

package beefyclient

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

// BeefyClientCommitment is an auto generated low-level Go binding around an user-defined struct.
type BeefyClientCommitment struct {
	BlockNumber    uint32
	ValidatorSetId uint64
	Payload        BeefyClientPayload
}

// BeefyClientMMRLeaf is an auto generated low-level Go binding around an user-defined struct.
type BeefyClientMMRLeaf struct {
	Version              uint8
	ParentNumber         uint32
	ParentHash           [32]byte
	NextAuthoritySetID   uint64
	NextAuthoritySetLen  uint32
	NextAuthoritySetRoot [32]byte
	ParachainHeadsRoot   [32]byte
}

// BeefyClientPayload is an auto generated low-level Go binding around an user-defined struct.
type BeefyClientPayload struct {
	MmrRootHash [32]byte
	Prefix      []byte
	Suffix      []byte
}

// BeefyClientValidatorProof is an auto generated low-level Go binding around an user-defined struct.
type BeefyClientValidatorProof struct {
	Signatures            [][]byte
	Positions             []*big.Int
	PublicKeys            []common.Address
	PublicKeyMerkleProofs [][][32]byte
}

// BeefyClientValidatorSet is an auto generated low-level Go binding around an user-defined struct.
type BeefyClientValidatorSet struct {
	Id     *big.Int
	Root   [32]byte
	Length *big.Int
}

// MMRProof is an auto generated low-level Go binding around an user-defined struct.
type MMRProof struct {
	Items [][32]byte
	Order uint64
}

// BeefyClientMetaData contains all meta data concerning the BeefyClient contract.
var BeefyClientMetaData = &bind.MetaData{
	ABI: "[{\"inputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"constructor\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":false,\"internalType\":\"uint256\",\"name\":\"id\",\"type\":\"uint256\"},{\"indexed\":false,\"internalType\":\"enumBeefyClient.Phase\",\"name\":\"phase\",\"type\":\"uint8\"},{\"indexed\":false,\"internalType\":\"bytes32\",\"name\":\"commitmentHash\",\"type\":\"bytes32\"},{\"indexed\":false,\"internalType\":\"address\",\"name\":\"prover\",\"type\":\"address\"}],\"name\":\"CommitmentVerified\",\"type\":\"event\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":false,\"internalType\":\"bytes32\",\"name\":\"mmrRoot\",\"type\":\"bytes32\"},{\"indexed\":false,\"internalType\":\"uint64\",\"name\":\"blockNumber\",\"type\":\"uint64\"}],\"name\":\"NewMMRRoot\",\"type\":\"event\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":true,\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\"},{\"indexed\":true,\"internalType\":\"bytes32\",\"name\":\"previousAdminRole\",\"type\":\"bytes32\"},{\"indexed\":true,\"internalType\":\"bytes32\",\"name\":\"newAdminRole\",\"type\":\"bytes32\"}],\"name\":\"RoleAdminChanged\",\"type\":\"event\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":true,\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\"},{\"indexed\":true,\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\"},{\"indexed\":true,\"internalType\":\"address\",\"name\":\"sender\",\"type\":\"address\"}],\"name\":\"RoleGranted\",\"type\":\"event\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":true,\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\"},{\"indexed\":true,\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\"},{\"indexed\":true,\"internalType\":\"address\",\"name\":\"sender\",\"type\":\"address\"}],\"name\":\"RoleRevoked\",\"type\":\"event\"},{\"inputs\":[],\"name\":\"BLOCK_WAIT_PERIOD\",\"outputs\":[{\"internalType\":\"uint64\",\"name\":\"\",\"type\":\"uint64\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"DEFAULT_ADMIN_ROLE\",\"outputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"THRESHOLD_DENOMINATOR\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"THRESHOLD_NUMERATOR\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"id\",\"type\":\"uint256\"},{\"components\":[{\"internalType\":\"uint32\",\"name\":\"blockNumber\",\"type\":\"uint32\"},{\"internalType\":\"uint64\",\"name\":\"validatorSetId\",\"type\":\"uint64\"},{\"components\":[{\"internalType\":\"bytes32\",\"name\":\"mmrRootHash\",\"type\":\"bytes32\"},{\"internalType\":\"bytes\",\"name\":\"prefix\",\"type\":\"bytes\"},{\"internalType\":\"bytes\",\"name\":\"suffix\",\"type\":\"bytes\"}],\"internalType\":\"structBeefyClient.Payload\",\"name\":\"payload\",\"type\":\"tuple\"}],\"internalType\":\"structBeefyClient.Commitment\",\"name\":\"commitment\",\"type\":\"tuple\"},{\"components\":[{\"internalType\":\"bytes[]\",\"name\":\"signatures\",\"type\":\"bytes[]\"},{\"internalType\":\"uint256[]\",\"name\":\"positions\",\"type\":\"uint256[]\"},{\"internalType\":\"address[]\",\"name\":\"publicKeys\",\"type\":\"address[]\"},{\"internalType\":\"bytes32[][]\",\"name\":\"publicKeyMerkleProofs\",\"type\":\"bytes32[][]\"}],\"internalType\":\"structBeefyClient.ValidatorProof\",\"name\":\"proof\",\"type\":\"tuple\"},{\"components\":[{\"internalType\":\"uint8\",\"name\":\"version\",\"type\":\"uint8\"},{\"internalType\":\"uint32\",\"name\":\"parentNumber\",\"type\":\"uint32\"},{\"internalType\":\"bytes32\",\"name\":\"parentHash\",\"type\":\"bytes32\"},{\"internalType\":\"uint64\",\"name\":\"nextAuthoritySetID\",\"type\":\"uint64\"},{\"internalType\":\"uint32\",\"name\":\"nextAuthoritySetLen\",\"type\":\"uint32\"},{\"internalType\":\"bytes32\",\"name\":\"nextAuthoritySetRoot\",\"type\":\"bytes32\"},{\"internalType\":\"bytes32\",\"name\":\"parachainHeadsRoot\",\"type\":\"bytes32\"}],\"internalType\":\"structBeefyClient.MMRLeaf\",\"name\":\"leaf\",\"type\":\"tuple\"},{\"components\":[{\"internalType\":\"bytes32[]\",\"name\":\"items\",\"type\":\"bytes32[]\"},{\"internalType\":\"uint64\",\"name\":\"order\",\"type\":\"uint64\"}],\"internalType\":\"structMMRProof\",\"name\":\"leafProof\",\"type\":\"tuple\"}],\"name\":\"completeCommitment\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"uint256[]\",\"name\":\"bitsToSet\",\"type\":\"uint256[]\"},{\"internalType\":\"uint256\",\"name\":\"length\",\"type\":\"uint256\"}],\"name\":\"createInitialBitfield\",\"outputs\":[{\"internalType\":\"uint256[]\",\"name\":\"\",\"type\":\"uint256[]\"}],\"stateMutability\":\"pure\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"id\",\"type\":\"uint256\"}],\"name\":\"createRandomBitfield\",\"outputs\":[{\"internalType\":\"uint256[]\",\"name\":\"\",\"type\":\"uint256[]\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"currentValidatorSet\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"id\",\"type\":\"uint256\"},{\"internalType\":\"bytes32\",\"name\":\"root\",\"type\":\"bytes32\"},{\"internalType\":\"uint256\",\"name\":\"length\",\"type\":\"uint256\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\"}],\"name\":\"getRoleAdmin\",\"outputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\"},{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\"}],\"name\":\"grantRole\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\"},{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\"}],\"name\":\"hasRole\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"uint64\",\"name\":\"_startingBeefyBlock\",\"type\":\"uint64\"},{\"components\":[{\"internalType\":\"uint256\",\"name\":\"id\",\"type\":\"uint256\"},{\"internalType\":\"bytes32\",\"name\":\"root\",\"type\":\"bytes32\"},{\"internalType\":\"uint256\",\"name\":\"length\",\"type\":\"uint256\"}],\"internalType\":\"structBeefyClient.ValidatorSet\",\"name\":\"_initialValidatorSet\",\"type\":\"tuple\"},{\"components\":[{\"internalType\":\"uint256\",\"name\":\"id\",\"type\":\"uint256\"},{\"internalType\":\"bytes32\",\"name\":\"root\",\"type\":\"bytes32\"},{\"internalType\":\"uint256\",\"name\":\"length\",\"type\":\"uint256\"}],\"internalType\":\"structBeefyClient.ValidatorSet\",\"name\":\"_nextValidatorSet\",\"type\":\"tuple\"}],\"name\":\"initialize\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"latestBeefyBlock\",\"outputs\":[{\"internalType\":\"uint64\",\"name\":\"\",\"type\":\"uint64\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"latestMMRRoot\",\"outputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"commitmentHash\",\"type\":\"bytes32\"},{\"internalType\":\"uint256[]\",\"name\":\"validatorClaimsBitfield\",\"type\":\"uint256[]\"},{\"internalType\":\"bytes\",\"name\":\"validatorSignature\",\"type\":\"bytes\"},{\"internalType\":\"uint256\",\"name\":\"validatorPosition\",\"type\":\"uint256\"},{\"internalType\":\"address\",\"name\":\"validatorPublicKey\",\"type\":\"address\"},{\"internalType\":\"bytes32[]\",\"name\":\"validatorPublicKeyMerkleProof\",\"type\":\"bytes32[]\"}],\"name\":\"newSignatureCommitment\",\"outputs\":[],\"stateMutability\":\"payable\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"nextID\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"nextValidatorSet\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"id\",\"type\":\"uint256\"},{\"internalType\":\"bytes32\",\"name\":\"root\",\"type\":\"bytes32\"},{\"internalType\":\"uint256\",\"name\":\"length\",\"type\":\"uint256\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\"},{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\"}],\"name\":\"renounceRole\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\"},{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\"}],\"name\":\"revokeRole\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"bytes4\",\"name\":\"interfaceId\",\"type\":\"bytes4\"}],\"name\":\"supportsInterface\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\"}],\"name\":\"validationData\",\"outputs\":[{\"internalType\":\"address\",\"name\":\"senderAddress\",\"type\":\"address\"},{\"internalType\":\"bytes32\",\"name\":\"commitmentHash\",\"type\":\"bytes32\"},{\"internalType\":\"uint256\",\"name\":\"blockNumber\",\"type\":\"uint256\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"leafHash\",\"type\":\"bytes32\"},{\"components\":[{\"internalType\":\"bytes32[]\",\"name\":\"items\",\"type\":\"bytes32[]\"},{\"internalType\":\"uint64\",\"name\":\"order\",\"type\":\"uint64\"}],\"internalType\":\"structMMRProof\",\"name\":\"proof\",\"type\":\"tuple\"}],\"name\":\"verifyMMRLeafProof\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\"}],\"stateMutability\":\"view\",\"type\":\"function\"}]",
}

// BeefyClientABI is the input ABI used to generate the binding from.
// Deprecated: Use BeefyClientMetaData.ABI instead.
var BeefyClientABI = BeefyClientMetaData.ABI

// BeefyClient is an auto generated Go binding around an Ethereum contract.
type BeefyClient struct {
	BeefyClientCaller     // Read-only binding to the contract
	BeefyClientTransactor // Write-only binding to the contract
	BeefyClientFilterer   // Log filterer for contract events
}

// BeefyClientCaller is an auto generated read-only Go binding around an Ethereum contract.
type BeefyClientCaller struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// BeefyClientTransactor is an auto generated write-only Go binding around an Ethereum contract.
type BeefyClientTransactor struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// BeefyClientFilterer is an auto generated log filtering Go binding around an Ethereum contract events.
type BeefyClientFilterer struct {
	contract *bind.BoundContract // Generic contract wrapper for the low level calls
}

// BeefyClientSession is an auto generated Go binding around an Ethereum contract,
// with pre-set call and transact options.
type BeefyClientSession struct {
	Contract     *BeefyClient      // Generic contract binding to set the session for
	CallOpts     bind.CallOpts     // Call options to use throughout this session
	TransactOpts bind.TransactOpts // Transaction auth options to use throughout this session
}

// BeefyClientCallerSession is an auto generated read-only Go binding around an Ethereum contract,
// with pre-set call options.
type BeefyClientCallerSession struct {
	Contract *BeefyClientCaller // Generic contract caller binding to set the session for
	CallOpts bind.CallOpts      // Call options to use throughout this session
}

// BeefyClientTransactorSession is an auto generated write-only Go binding around an Ethereum contract,
// with pre-set transact options.
type BeefyClientTransactorSession struct {
	Contract     *BeefyClientTransactor // Generic contract transactor binding to set the session for
	TransactOpts bind.TransactOpts      // Transaction auth options to use throughout this session
}

// BeefyClientRaw is an auto generated low-level Go binding around an Ethereum contract.
type BeefyClientRaw struct {
	Contract *BeefyClient // Generic contract binding to access the raw methods on
}

// BeefyClientCallerRaw is an auto generated low-level read-only Go binding around an Ethereum contract.
type BeefyClientCallerRaw struct {
	Contract *BeefyClientCaller // Generic read-only contract binding to access the raw methods on
}

// BeefyClientTransactorRaw is an auto generated low-level write-only Go binding around an Ethereum contract.
type BeefyClientTransactorRaw struct {
	Contract *BeefyClientTransactor // Generic write-only contract binding to access the raw methods on
}

// NewBeefyClient creates a new instance of BeefyClient, bound to a specific deployed contract.
func NewBeefyClient(address common.Address, backend bind.ContractBackend) (*BeefyClient, error) {
	contract, err := bindBeefyClient(address, backend, backend, backend)
	if err != nil {
		return nil, err
	}
	return &BeefyClient{BeefyClientCaller: BeefyClientCaller{contract: contract}, BeefyClientTransactor: BeefyClientTransactor{contract: contract}, BeefyClientFilterer: BeefyClientFilterer{contract: contract}}, nil
}

// NewBeefyClientCaller creates a new read-only instance of BeefyClient, bound to a specific deployed contract.
func NewBeefyClientCaller(address common.Address, caller bind.ContractCaller) (*BeefyClientCaller, error) {
	contract, err := bindBeefyClient(address, caller, nil, nil)
	if err != nil {
		return nil, err
	}
	return &BeefyClientCaller{contract: contract}, nil
}

// NewBeefyClientTransactor creates a new write-only instance of BeefyClient, bound to a specific deployed contract.
func NewBeefyClientTransactor(address common.Address, transactor bind.ContractTransactor) (*BeefyClientTransactor, error) {
	contract, err := bindBeefyClient(address, nil, transactor, nil)
	if err != nil {
		return nil, err
	}
	return &BeefyClientTransactor{contract: contract}, nil
}

// NewBeefyClientFilterer creates a new log filterer instance of BeefyClient, bound to a specific deployed contract.
func NewBeefyClientFilterer(address common.Address, filterer bind.ContractFilterer) (*BeefyClientFilterer, error) {
	contract, err := bindBeefyClient(address, nil, nil, filterer)
	if err != nil {
		return nil, err
	}
	return &BeefyClientFilterer{contract: contract}, nil
}

// bindBeefyClient binds a generic wrapper to an already deployed contract.
func bindBeefyClient(address common.Address, caller bind.ContractCaller, transactor bind.ContractTransactor, filterer bind.ContractFilterer) (*bind.BoundContract, error) {
	parsed, err := abi.JSON(strings.NewReader(BeefyClientABI))
	if err != nil {
		return nil, err
	}
	return bind.NewBoundContract(address, parsed, caller, transactor, filterer), nil
}

// Call invokes the (constant) contract method with params as input values and
// sets the output to result. The result type might be a single field for simple
// returns, a slice of interfaces for anonymous returns and a struct for named
// returns.
func (_BeefyClient *BeefyClientRaw) Call(opts *bind.CallOpts, result *[]interface{}, method string, params ...interface{}) error {
	return _BeefyClient.Contract.BeefyClientCaller.contract.Call(opts, result, method, params...)
}

// Transfer initiates a plain transaction to move funds to the contract, calling
// its default method if one is available.
func (_BeefyClient *BeefyClientRaw) Transfer(opts *bind.TransactOpts) (*types.Transaction, error) {
	return _BeefyClient.Contract.BeefyClientTransactor.contract.Transfer(opts)
}

// Transact invokes the (paid) contract method with params as input values.
func (_BeefyClient *BeefyClientRaw) Transact(opts *bind.TransactOpts, method string, params ...interface{}) (*types.Transaction, error) {
	return _BeefyClient.Contract.BeefyClientTransactor.contract.Transact(opts, method, params...)
}

// Call invokes the (constant) contract method with params as input values and
// sets the output to result. The result type might be a single field for simple
// returns, a slice of interfaces for anonymous returns and a struct for named
// returns.
func (_BeefyClient *BeefyClientCallerRaw) Call(opts *bind.CallOpts, result *[]interface{}, method string, params ...interface{}) error {
	return _BeefyClient.Contract.contract.Call(opts, result, method, params...)
}

// Transfer initiates a plain transaction to move funds to the contract, calling
// its default method if one is available.
func (_BeefyClient *BeefyClientTransactorRaw) Transfer(opts *bind.TransactOpts) (*types.Transaction, error) {
	return _BeefyClient.Contract.contract.Transfer(opts)
}

// Transact invokes the (paid) contract method with params as input values.
func (_BeefyClient *BeefyClientTransactorRaw) Transact(opts *bind.TransactOpts, method string, params ...interface{}) (*types.Transaction, error) {
	return _BeefyClient.Contract.contract.Transact(opts, method, params...)
}

// BLOCKWAITPERIOD is a free data retrieval call binding the contract method 0xfb752c62.
//
// Solidity: function BLOCK_WAIT_PERIOD() view returns(uint64)
func (_BeefyClient *BeefyClientCaller) BLOCKWAITPERIOD(opts *bind.CallOpts) (uint64, error) {
	var out []interface{}
	err := _BeefyClient.contract.Call(opts, &out, "BLOCK_WAIT_PERIOD")

	if err != nil {
		return *new(uint64), err
	}

	out0 := *abi.ConvertType(out[0], new(uint64)).(*uint64)

	return out0, err

}

// BLOCKWAITPERIOD is a free data retrieval call binding the contract method 0xfb752c62.
//
// Solidity: function BLOCK_WAIT_PERIOD() view returns(uint64)
func (_BeefyClient *BeefyClientSession) BLOCKWAITPERIOD() (uint64, error) {
	return _BeefyClient.Contract.BLOCKWAITPERIOD(&_BeefyClient.CallOpts)
}

// BLOCKWAITPERIOD is a free data retrieval call binding the contract method 0xfb752c62.
//
// Solidity: function BLOCK_WAIT_PERIOD() view returns(uint64)
func (_BeefyClient *BeefyClientCallerSession) BLOCKWAITPERIOD() (uint64, error) {
	return _BeefyClient.Contract.BLOCKWAITPERIOD(&_BeefyClient.CallOpts)
}

// DEFAULTADMINROLE is a free data retrieval call binding the contract method 0xa217fddf.
//
// Solidity: function DEFAULT_ADMIN_ROLE() view returns(bytes32)
func (_BeefyClient *BeefyClientCaller) DEFAULTADMINROLE(opts *bind.CallOpts) ([32]byte, error) {
	var out []interface{}
	err := _BeefyClient.contract.Call(opts, &out, "DEFAULT_ADMIN_ROLE")

	if err != nil {
		return *new([32]byte), err
	}

	out0 := *abi.ConvertType(out[0], new([32]byte)).(*[32]byte)

	return out0, err

}

// DEFAULTADMINROLE is a free data retrieval call binding the contract method 0xa217fddf.
//
// Solidity: function DEFAULT_ADMIN_ROLE() view returns(bytes32)
func (_BeefyClient *BeefyClientSession) DEFAULTADMINROLE() ([32]byte, error) {
	return _BeefyClient.Contract.DEFAULTADMINROLE(&_BeefyClient.CallOpts)
}

// DEFAULTADMINROLE is a free data retrieval call binding the contract method 0xa217fddf.
//
// Solidity: function DEFAULT_ADMIN_ROLE() view returns(bytes32)
func (_BeefyClient *BeefyClientCallerSession) DEFAULTADMINROLE() ([32]byte, error) {
	return _BeefyClient.Contract.DEFAULTADMINROLE(&_BeefyClient.CallOpts)
}

// THRESHOLDDENOMINATOR is a free data retrieval call binding the contract method 0xef024458.
//
// Solidity: function THRESHOLD_DENOMINATOR() view returns(uint256)
func (_BeefyClient *BeefyClientCaller) THRESHOLDDENOMINATOR(opts *bind.CallOpts) (*big.Int, error) {
	var out []interface{}
	err := _BeefyClient.contract.Call(opts, &out, "THRESHOLD_DENOMINATOR")

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// THRESHOLDDENOMINATOR is a free data retrieval call binding the contract method 0xef024458.
//
// Solidity: function THRESHOLD_DENOMINATOR() view returns(uint256)
func (_BeefyClient *BeefyClientSession) THRESHOLDDENOMINATOR() (*big.Int, error) {
	return _BeefyClient.Contract.THRESHOLDDENOMINATOR(&_BeefyClient.CallOpts)
}

// THRESHOLDDENOMINATOR is a free data retrieval call binding the contract method 0xef024458.
//
// Solidity: function THRESHOLD_DENOMINATOR() view returns(uint256)
func (_BeefyClient *BeefyClientCallerSession) THRESHOLDDENOMINATOR() (*big.Int, error) {
	return _BeefyClient.Contract.THRESHOLDDENOMINATOR(&_BeefyClient.CallOpts)
}

// THRESHOLDNUMERATOR is a free data retrieval call binding the contract method 0x5a8d2f0e.
//
// Solidity: function THRESHOLD_NUMERATOR() view returns(uint256)
func (_BeefyClient *BeefyClientCaller) THRESHOLDNUMERATOR(opts *bind.CallOpts) (*big.Int, error) {
	var out []interface{}
	err := _BeefyClient.contract.Call(opts, &out, "THRESHOLD_NUMERATOR")

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// THRESHOLDNUMERATOR is a free data retrieval call binding the contract method 0x5a8d2f0e.
//
// Solidity: function THRESHOLD_NUMERATOR() view returns(uint256)
func (_BeefyClient *BeefyClientSession) THRESHOLDNUMERATOR() (*big.Int, error) {
	return _BeefyClient.Contract.THRESHOLDNUMERATOR(&_BeefyClient.CallOpts)
}

// THRESHOLDNUMERATOR is a free data retrieval call binding the contract method 0x5a8d2f0e.
//
// Solidity: function THRESHOLD_NUMERATOR() view returns(uint256)
func (_BeefyClient *BeefyClientCallerSession) THRESHOLDNUMERATOR() (*big.Int, error) {
	return _BeefyClient.Contract.THRESHOLDNUMERATOR(&_BeefyClient.CallOpts)
}

// CreateInitialBitfield is a free data retrieval call binding the contract method 0x5da57fe9.
//
// Solidity: function createInitialBitfield(uint256[] bitsToSet, uint256 length) pure returns(uint256[])
func (_BeefyClient *BeefyClientCaller) CreateInitialBitfield(opts *bind.CallOpts, bitsToSet []*big.Int, length *big.Int) ([]*big.Int, error) {
	var out []interface{}
	err := _BeefyClient.contract.Call(opts, &out, "createInitialBitfield", bitsToSet, length)

	if err != nil {
		return *new([]*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new([]*big.Int)).(*[]*big.Int)

	return out0, err

}

// CreateInitialBitfield is a free data retrieval call binding the contract method 0x5da57fe9.
//
// Solidity: function createInitialBitfield(uint256[] bitsToSet, uint256 length) pure returns(uint256[])
func (_BeefyClient *BeefyClientSession) CreateInitialBitfield(bitsToSet []*big.Int, length *big.Int) ([]*big.Int, error) {
	return _BeefyClient.Contract.CreateInitialBitfield(&_BeefyClient.CallOpts, bitsToSet, length)
}

// CreateInitialBitfield is a free data retrieval call binding the contract method 0x5da57fe9.
//
// Solidity: function createInitialBitfield(uint256[] bitsToSet, uint256 length) pure returns(uint256[])
func (_BeefyClient *BeefyClientCallerSession) CreateInitialBitfield(bitsToSet []*big.Int, length *big.Int) ([]*big.Int, error) {
	return _BeefyClient.Contract.CreateInitialBitfield(&_BeefyClient.CallOpts, bitsToSet, length)
}

// CreateRandomBitfield is a free data retrieval call binding the contract method 0x92848016.
//
// Solidity: function createRandomBitfield(uint256 id) view returns(uint256[])
func (_BeefyClient *BeefyClientCaller) CreateRandomBitfield(opts *bind.CallOpts, id *big.Int) ([]*big.Int, error) {
	var out []interface{}
	err := _BeefyClient.contract.Call(opts, &out, "createRandomBitfield", id)

	if err != nil {
		return *new([]*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new([]*big.Int)).(*[]*big.Int)

	return out0, err

}

// CreateRandomBitfield is a free data retrieval call binding the contract method 0x92848016.
//
// Solidity: function createRandomBitfield(uint256 id) view returns(uint256[])
func (_BeefyClient *BeefyClientSession) CreateRandomBitfield(id *big.Int) ([]*big.Int, error) {
	return _BeefyClient.Contract.CreateRandomBitfield(&_BeefyClient.CallOpts, id)
}

// CreateRandomBitfield is a free data retrieval call binding the contract method 0x92848016.
//
// Solidity: function createRandomBitfield(uint256 id) view returns(uint256[])
func (_BeefyClient *BeefyClientCallerSession) CreateRandomBitfield(id *big.Int) ([]*big.Int, error) {
	return _BeefyClient.Contract.CreateRandomBitfield(&_BeefyClient.CallOpts, id)
}

// CurrentValidatorSet is a free data retrieval call binding the contract method 0x2cdea717.
//
// Solidity: function currentValidatorSet() view returns(uint256 id, bytes32 root, uint256 length)
func (_BeefyClient *BeefyClientCaller) CurrentValidatorSet(opts *bind.CallOpts) (struct {
	Id     *big.Int
	Root   [32]byte
	Length *big.Int
}, error) {
	var out []interface{}
	err := _BeefyClient.contract.Call(opts, &out, "currentValidatorSet")

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
func (_BeefyClient *BeefyClientSession) CurrentValidatorSet() (struct {
	Id     *big.Int
	Root   [32]byte
	Length *big.Int
}, error) {
	return _BeefyClient.Contract.CurrentValidatorSet(&_BeefyClient.CallOpts)
}

// CurrentValidatorSet is a free data retrieval call binding the contract method 0x2cdea717.
//
// Solidity: function currentValidatorSet() view returns(uint256 id, bytes32 root, uint256 length)
func (_BeefyClient *BeefyClientCallerSession) CurrentValidatorSet() (struct {
	Id     *big.Int
	Root   [32]byte
	Length *big.Int
}, error) {
	return _BeefyClient.Contract.CurrentValidatorSet(&_BeefyClient.CallOpts)
}

// GetRoleAdmin is a free data retrieval call binding the contract method 0x248a9ca3.
//
// Solidity: function getRoleAdmin(bytes32 role) view returns(bytes32)
func (_BeefyClient *BeefyClientCaller) GetRoleAdmin(opts *bind.CallOpts, role [32]byte) ([32]byte, error) {
	var out []interface{}
	err := _BeefyClient.contract.Call(opts, &out, "getRoleAdmin", role)

	if err != nil {
		return *new([32]byte), err
	}

	out0 := *abi.ConvertType(out[0], new([32]byte)).(*[32]byte)

	return out0, err

}

// GetRoleAdmin is a free data retrieval call binding the contract method 0x248a9ca3.
//
// Solidity: function getRoleAdmin(bytes32 role) view returns(bytes32)
func (_BeefyClient *BeefyClientSession) GetRoleAdmin(role [32]byte) ([32]byte, error) {
	return _BeefyClient.Contract.GetRoleAdmin(&_BeefyClient.CallOpts, role)
}

// GetRoleAdmin is a free data retrieval call binding the contract method 0x248a9ca3.
//
// Solidity: function getRoleAdmin(bytes32 role) view returns(bytes32)
func (_BeefyClient *BeefyClientCallerSession) GetRoleAdmin(role [32]byte) ([32]byte, error) {
	return _BeefyClient.Contract.GetRoleAdmin(&_BeefyClient.CallOpts, role)
}

// HasRole is a free data retrieval call binding the contract method 0x91d14854.
//
// Solidity: function hasRole(bytes32 role, address account) view returns(bool)
func (_BeefyClient *BeefyClientCaller) HasRole(opts *bind.CallOpts, role [32]byte, account common.Address) (bool, error) {
	var out []interface{}
	err := _BeefyClient.contract.Call(opts, &out, "hasRole", role, account)

	if err != nil {
		return *new(bool), err
	}

	out0 := *abi.ConvertType(out[0], new(bool)).(*bool)

	return out0, err

}

// HasRole is a free data retrieval call binding the contract method 0x91d14854.
//
// Solidity: function hasRole(bytes32 role, address account) view returns(bool)
func (_BeefyClient *BeefyClientSession) HasRole(role [32]byte, account common.Address) (bool, error) {
	return _BeefyClient.Contract.HasRole(&_BeefyClient.CallOpts, role, account)
}

// HasRole is a free data retrieval call binding the contract method 0x91d14854.
//
// Solidity: function hasRole(bytes32 role, address account) view returns(bool)
func (_BeefyClient *BeefyClientCallerSession) HasRole(role [32]byte, account common.Address) (bool, error) {
	return _BeefyClient.Contract.HasRole(&_BeefyClient.CallOpts, role, account)
}

// LatestBeefyBlock is a free data retrieval call binding the contract method 0x66ae69a0.
//
// Solidity: function latestBeefyBlock() view returns(uint64)
func (_BeefyClient *BeefyClientCaller) LatestBeefyBlock(opts *bind.CallOpts) (uint64, error) {
	var out []interface{}
	err := _BeefyClient.contract.Call(opts, &out, "latestBeefyBlock")

	if err != nil {
		return *new(uint64), err
	}

	out0 := *abi.ConvertType(out[0], new(uint64)).(*uint64)

	return out0, err

}

// LatestBeefyBlock is a free data retrieval call binding the contract method 0x66ae69a0.
//
// Solidity: function latestBeefyBlock() view returns(uint64)
func (_BeefyClient *BeefyClientSession) LatestBeefyBlock() (uint64, error) {
	return _BeefyClient.Contract.LatestBeefyBlock(&_BeefyClient.CallOpts)
}

// LatestBeefyBlock is a free data retrieval call binding the contract method 0x66ae69a0.
//
// Solidity: function latestBeefyBlock() view returns(uint64)
func (_BeefyClient *BeefyClientCallerSession) LatestBeefyBlock() (uint64, error) {
	return _BeefyClient.Contract.LatestBeefyBlock(&_BeefyClient.CallOpts)
}

// LatestMMRRoot is a free data retrieval call binding the contract method 0x41c9634e.
//
// Solidity: function latestMMRRoot() view returns(bytes32)
func (_BeefyClient *BeefyClientCaller) LatestMMRRoot(opts *bind.CallOpts) ([32]byte, error) {
	var out []interface{}
	err := _BeefyClient.contract.Call(opts, &out, "latestMMRRoot")

	if err != nil {
		return *new([32]byte), err
	}

	out0 := *abi.ConvertType(out[0], new([32]byte)).(*[32]byte)

	return out0, err

}

// LatestMMRRoot is a free data retrieval call binding the contract method 0x41c9634e.
//
// Solidity: function latestMMRRoot() view returns(bytes32)
func (_BeefyClient *BeefyClientSession) LatestMMRRoot() ([32]byte, error) {
	return _BeefyClient.Contract.LatestMMRRoot(&_BeefyClient.CallOpts)
}

// LatestMMRRoot is a free data retrieval call binding the contract method 0x41c9634e.
//
// Solidity: function latestMMRRoot() view returns(bytes32)
func (_BeefyClient *BeefyClientCallerSession) LatestMMRRoot() ([32]byte, error) {
	return _BeefyClient.Contract.LatestMMRRoot(&_BeefyClient.CallOpts)
}

// NextID is a free data retrieval call binding the contract method 0x1e96917d.
//
// Solidity: function nextID() view returns(uint256)
func (_BeefyClient *BeefyClientCaller) NextID(opts *bind.CallOpts) (*big.Int, error) {
	var out []interface{}
	err := _BeefyClient.contract.Call(opts, &out, "nextID")

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// NextID is a free data retrieval call binding the contract method 0x1e96917d.
//
// Solidity: function nextID() view returns(uint256)
func (_BeefyClient *BeefyClientSession) NextID() (*big.Int, error) {
	return _BeefyClient.Contract.NextID(&_BeefyClient.CallOpts)
}

// NextID is a free data retrieval call binding the contract method 0x1e96917d.
//
// Solidity: function nextID() view returns(uint256)
func (_BeefyClient *BeefyClientCallerSession) NextID() (*big.Int, error) {
	return _BeefyClient.Contract.NextID(&_BeefyClient.CallOpts)
}

// NextValidatorSet is a free data retrieval call binding the contract method 0x36667513.
//
// Solidity: function nextValidatorSet() view returns(uint256 id, bytes32 root, uint256 length)
func (_BeefyClient *BeefyClientCaller) NextValidatorSet(opts *bind.CallOpts) (struct {
	Id     *big.Int
	Root   [32]byte
	Length *big.Int
}, error) {
	var out []interface{}
	err := _BeefyClient.contract.Call(opts, &out, "nextValidatorSet")

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
func (_BeefyClient *BeefyClientSession) NextValidatorSet() (struct {
	Id     *big.Int
	Root   [32]byte
	Length *big.Int
}, error) {
	return _BeefyClient.Contract.NextValidatorSet(&_BeefyClient.CallOpts)
}

// NextValidatorSet is a free data retrieval call binding the contract method 0x36667513.
//
// Solidity: function nextValidatorSet() view returns(uint256 id, bytes32 root, uint256 length)
func (_BeefyClient *BeefyClientCallerSession) NextValidatorSet() (struct {
	Id     *big.Int
	Root   [32]byte
	Length *big.Int
}, error) {
	return _BeefyClient.Contract.NextValidatorSet(&_BeefyClient.CallOpts)
}

// SupportsInterface is a free data retrieval call binding the contract method 0x01ffc9a7.
//
// Solidity: function supportsInterface(bytes4 interfaceId) view returns(bool)
func (_BeefyClient *BeefyClientCaller) SupportsInterface(opts *bind.CallOpts, interfaceId [4]byte) (bool, error) {
	var out []interface{}
	err := _BeefyClient.contract.Call(opts, &out, "supportsInterface", interfaceId)

	if err != nil {
		return *new(bool), err
	}

	out0 := *abi.ConvertType(out[0], new(bool)).(*bool)

	return out0, err

}

// SupportsInterface is a free data retrieval call binding the contract method 0x01ffc9a7.
//
// Solidity: function supportsInterface(bytes4 interfaceId) view returns(bool)
func (_BeefyClient *BeefyClientSession) SupportsInterface(interfaceId [4]byte) (bool, error) {
	return _BeefyClient.Contract.SupportsInterface(&_BeefyClient.CallOpts, interfaceId)
}

// SupportsInterface is a free data retrieval call binding the contract method 0x01ffc9a7.
//
// Solidity: function supportsInterface(bytes4 interfaceId) view returns(bool)
func (_BeefyClient *BeefyClientCallerSession) SupportsInterface(interfaceId [4]byte) (bool, error) {
	return _BeefyClient.Contract.SupportsInterface(&_BeefyClient.CallOpts, interfaceId)
}

// ValidationData is a free data retrieval call binding the contract method 0x20bfa5cb.
//
// Solidity: function validationData(uint256 ) view returns(address senderAddress, bytes32 commitmentHash, uint256 blockNumber)
func (_BeefyClient *BeefyClientCaller) ValidationData(opts *bind.CallOpts, arg0 *big.Int) (struct {
	SenderAddress  common.Address
	CommitmentHash [32]byte
	BlockNumber    *big.Int
}, error) {
	var out []interface{}
	err := _BeefyClient.contract.Call(opts, &out, "validationData", arg0)

	outstruct := new(struct {
		SenderAddress  common.Address
		CommitmentHash [32]byte
		BlockNumber    *big.Int
	})
	if err != nil {
		return *outstruct, err
	}

	outstruct.SenderAddress = *abi.ConvertType(out[0], new(common.Address)).(*common.Address)
	outstruct.CommitmentHash = *abi.ConvertType(out[1], new([32]byte)).(*[32]byte)
	outstruct.BlockNumber = *abi.ConvertType(out[2], new(*big.Int)).(**big.Int)

	return *outstruct, err

}

// ValidationData is a free data retrieval call binding the contract method 0x20bfa5cb.
//
// Solidity: function validationData(uint256 ) view returns(address senderAddress, bytes32 commitmentHash, uint256 blockNumber)
func (_BeefyClient *BeefyClientSession) ValidationData(arg0 *big.Int) (struct {
	SenderAddress  common.Address
	CommitmentHash [32]byte
	BlockNumber    *big.Int
}, error) {
	return _BeefyClient.Contract.ValidationData(&_BeefyClient.CallOpts, arg0)
}

// ValidationData is a free data retrieval call binding the contract method 0x20bfa5cb.
//
// Solidity: function validationData(uint256 ) view returns(address senderAddress, bytes32 commitmentHash, uint256 blockNumber)
func (_BeefyClient *BeefyClientCallerSession) ValidationData(arg0 *big.Int) (struct {
	SenderAddress  common.Address
	CommitmentHash [32]byte
	BlockNumber    *big.Int
}, error) {
	return _BeefyClient.Contract.ValidationData(&_BeefyClient.CallOpts, arg0)
}

// VerifyMMRLeafProof is a free data retrieval call binding the contract method 0x8cdeab50.
//
// Solidity: function verifyMMRLeafProof(bytes32 leafHash, (bytes32[],uint64) proof) view returns(bool)
func (_BeefyClient *BeefyClientCaller) VerifyMMRLeafProof(opts *bind.CallOpts, leafHash [32]byte, proof MMRProof) (bool, error) {
	var out []interface{}
	err := _BeefyClient.contract.Call(opts, &out, "verifyMMRLeafProof", leafHash, proof)

	if err != nil {
		return *new(bool), err
	}

	out0 := *abi.ConvertType(out[0], new(bool)).(*bool)

	return out0, err

}

// VerifyMMRLeafProof is a free data retrieval call binding the contract method 0x8cdeab50.
//
// Solidity: function verifyMMRLeafProof(bytes32 leafHash, (bytes32[],uint64) proof) view returns(bool)
func (_BeefyClient *BeefyClientSession) VerifyMMRLeafProof(leafHash [32]byte, proof MMRProof) (bool, error) {
	return _BeefyClient.Contract.VerifyMMRLeafProof(&_BeefyClient.CallOpts, leafHash, proof)
}

// VerifyMMRLeafProof is a free data retrieval call binding the contract method 0x8cdeab50.
//
// Solidity: function verifyMMRLeafProof(bytes32 leafHash, (bytes32[],uint64) proof) view returns(bool)
func (_BeefyClient *BeefyClientCallerSession) VerifyMMRLeafProof(leafHash [32]byte, proof MMRProof) (bool, error) {
	return _BeefyClient.Contract.VerifyMMRLeafProof(&_BeefyClient.CallOpts, leafHash, proof)
}

// CompleteCommitment is a paid mutator transaction binding the contract method 0xaaadef17.
//
// Solidity: function completeCommitment(uint256 id, (uint32,uint64,(bytes32,bytes,bytes)) commitment, (bytes[],uint256[],address[],bytes32[][]) proof, (uint8,uint32,bytes32,uint64,uint32,bytes32,bytes32) leaf, (bytes32[],uint64) leafProof) returns()
func (_BeefyClient *BeefyClientTransactor) CompleteCommitment(opts *bind.TransactOpts, id *big.Int, commitment BeefyClientCommitment, proof BeefyClientValidatorProof, leaf BeefyClientMMRLeaf, leafProof MMRProof) (*types.Transaction, error) {
	return _BeefyClient.contract.Transact(opts, "completeCommitment", id, commitment, proof, leaf, leafProof)
}

// CompleteCommitment is a paid mutator transaction binding the contract method 0xaaadef17.
//
// Solidity: function completeCommitment(uint256 id, (uint32,uint64,(bytes32,bytes,bytes)) commitment, (bytes[],uint256[],address[],bytes32[][]) proof, (uint8,uint32,bytes32,uint64,uint32,bytes32,bytes32) leaf, (bytes32[],uint64) leafProof) returns()
func (_BeefyClient *BeefyClientSession) CompleteCommitment(id *big.Int, commitment BeefyClientCommitment, proof BeefyClientValidatorProof, leaf BeefyClientMMRLeaf, leafProof MMRProof) (*types.Transaction, error) {
	return _BeefyClient.Contract.CompleteCommitment(&_BeefyClient.TransactOpts, id, commitment, proof, leaf, leafProof)
}

// CompleteCommitment is a paid mutator transaction binding the contract method 0xaaadef17.
//
// Solidity: function completeCommitment(uint256 id, (uint32,uint64,(bytes32,bytes,bytes)) commitment, (bytes[],uint256[],address[],bytes32[][]) proof, (uint8,uint32,bytes32,uint64,uint32,bytes32,bytes32) leaf, (bytes32[],uint64) leafProof) returns()
func (_BeefyClient *BeefyClientTransactorSession) CompleteCommitment(id *big.Int, commitment BeefyClientCommitment, proof BeefyClientValidatorProof, leaf BeefyClientMMRLeaf, leafProof MMRProof) (*types.Transaction, error) {
	return _BeefyClient.Contract.CompleteCommitment(&_BeefyClient.TransactOpts, id, commitment, proof, leaf, leafProof)
}

// GrantRole is a paid mutator transaction binding the contract method 0x2f2ff15d.
//
// Solidity: function grantRole(bytes32 role, address account) returns()
func (_BeefyClient *BeefyClientTransactor) GrantRole(opts *bind.TransactOpts, role [32]byte, account common.Address) (*types.Transaction, error) {
	return _BeefyClient.contract.Transact(opts, "grantRole", role, account)
}

// GrantRole is a paid mutator transaction binding the contract method 0x2f2ff15d.
//
// Solidity: function grantRole(bytes32 role, address account) returns()
func (_BeefyClient *BeefyClientSession) GrantRole(role [32]byte, account common.Address) (*types.Transaction, error) {
	return _BeefyClient.Contract.GrantRole(&_BeefyClient.TransactOpts, role, account)
}

// GrantRole is a paid mutator transaction binding the contract method 0x2f2ff15d.
//
// Solidity: function grantRole(bytes32 role, address account) returns()
func (_BeefyClient *BeefyClientTransactorSession) GrantRole(role [32]byte, account common.Address) (*types.Transaction, error) {
	return _BeefyClient.Contract.GrantRole(&_BeefyClient.TransactOpts, role, account)
}

// Initialize is a paid mutator transaction binding the contract method 0x3795ea5f.
//
// Solidity: function initialize(uint64 _startingBeefyBlock, (uint256,bytes32,uint256) _initialValidatorSet, (uint256,bytes32,uint256) _nextValidatorSet) returns()
func (_BeefyClient *BeefyClientTransactor) Initialize(opts *bind.TransactOpts, _startingBeefyBlock uint64, _initialValidatorSet BeefyClientValidatorSet, _nextValidatorSet BeefyClientValidatorSet) (*types.Transaction, error) {
	return _BeefyClient.contract.Transact(opts, "initialize", _startingBeefyBlock, _initialValidatorSet, _nextValidatorSet)
}

// Initialize is a paid mutator transaction binding the contract method 0x3795ea5f.
//
// Solidity: function initialize(uint64 _startingBeefyBlock, (uint256,bytes32,uint256) _initialValidatorSet, (uint256,bytes32,uint256) _nextValidatorSet) returns()
func (_BeefyClient *BeefyClientSession) Initialize(_startingBeefyBlock uint64, _initialValidatorSet BeefyClientValidatorSet, _nextValidatorSet BeefyClientValidatorSet) (*types.Transaction, error) {
	return _BeefyClient.Contract.Initialize(&_BeefyClient.TransactOpts, _startingBeefyBlock, _initialValidatorSet, _nextValidatorSet)
}

// Initialize is a paid mutator transaction binding the contract method 0x3795ea5f.
//
// Solidity: function initialize(uint64 _startingBeefyBlock, (uint256,bytes32,uint256) _initialValidatorSet, (uint256,bytes32,uint256) _nextValidatorSet) returns()
func (_BeefyClient *BeefyClientTransactorSession) Initialize(_startingBeefyBlock uint64, _initialValidatorSet BeefyClientValidatorSet, _nextValidatorSet BeefyClientValidatorSet) (*types.Transaction, error) {
	return _BeefyClient.Contract.Initialize(&_BeefyClient.TransactOpts, _startingBeefyBlock, _initialValidatorSet, _nextValidatorSet)
}

// NewSignatureCommitment is a paid mutator transaction binding the contract method 0xe54d1543.
//
// Solidity: function newSignatureCommitment(bytes32 commitmentHash, uint256[] validatorClaimsBitfield, bytes validatorSignature, uint256 validatorPosition, address validatorPublicKey, bytes32[] validatorPublicKeyMerkleProof) payable returns()
func (_BeefyClient *BeefyClientTransactor) NewSignatureCommitment(opts *bind.TransactOpts, commitmentHash [32]byte, validatorClaimsBitfield []*big.Int, validatorSignature []byte, validatorPosition *big.Int, validatorPublicKey common.Address, validatorPublicKeyMerkleProof [][32]byte) (*types.Transaction, error) {
	return _BeefyClient.contract.Transact(opts, "newSignatureCommitment", commitmentHash, validatorClaimsBitfield, validatorSignature, validatorPosition, validatorPublicKey, validatorPublicKeyMerkleProof)
}

// NewSignatureCommitment is a paid mutator transaction binding the contract method 0xe54d1543.
//
// Solidity: function newSignatureCommitment(bytes32 commitmentHash, uint256[] validatorClaimsBitfield, bytes validatorSignature, uint256 validatorPosition, address validatorPublicKey, bytes32[] validatorPublicKeyMerkleProof) payable returns()
func (_BeefyClient *BeefyClientSession) NewSignatureCommitment(commitmentHash [32]byte, validatorClaimsBitfield []*big.Int, validatorSignature []byte, validatorPosition *big.Int, validatorPublicKey common.Address, validatorPublicKeyMerkleProof [][32]byte) (*types.Transaction, error) {
	return _BeefyClient.Contract.NewSignatureCommitment(&_BeefyClient.TransactOpts, commitmentHash, validatorClaimsBitfield, validatorSignature, validatorPosition, validatorPublicKey, validatorPublicKeyMerkleProof)
}

// NewSignatureCommitment is a paid mutator transaction binding the contract method 0xe54d1543.
//
// Solidity: function newSignatureCommitment(bytes32 commitmentHash, uint256[] validatorClaimsBitfield, bytes validatorSignature, uint256 validatorPosition, address validatorPublicKey, bytes32[] validatorPublicKeyMerkleProof) payable returns()
func (_BeefyClient *BeefyClientTransactorSession) NewSignatureCommitment(commitmentHash [32]byte, validatorClaimsBitfield []*big.Int, validatorSignature []byte, validatorPosition *big.Int, validatorPublicKey common.Address, validatorPublicKeyMerkleProof [][32]byte) (*types.Transaction, error) {
	return _BeefyClient.Contract.NewSignatureCommitment(&_BeefyClient.TransactOpts, commitmentHash, validatorClaimsBitfield, validatorSignature, validatorPosition, validatorPublicKey, validatorPublicKeyMerkleProof)
}

// RenounceRole is a paid mutator transaction binding the contract method 0x36568abe.
//
// Solidity: function renounceRole(bytes32 role, address account) returns()
func (_BeefyClient *BeefyClientTransactor) RenounceRole(opts *bind.TransactOpts, role [32]byte, account common.Address) (*types.Transaction, error) {
	return _BeefyClient.contract.Transact(opts, "renounceRole", role, account)
}

// RenounceRole is a paid mutator transaction binding the contract method 0x36568abe.
//
// Solidity: function renounceRole(bytes32 role, address account) returns()
func (_BeefyClient *BeefyClientSession) RenounceRole(role [32]byte, account common.Address) (*types.Transaction, error) {
	return _BeefyClient.Contract.RenounceRole(&_BeefyClient.TransactOpts, role, account)
}

// RenounceRole is a paid mutator transaction binding the contract method 0x36568abe.
//
// Solidity: function renounceRole(bytes32 role, address account) returns()
func (_BeefyClient *BeefyClientTransactorSession) RenounceRole(role [32]byte, account common.Address) (*types.Transaction, error) {
	return _BeefyClient.Contract.RenounceRole(&_BeefyClient.TransactOpts, role, account)
}

// RevokeRole is a paid mutator transaction binding the contract method 0xd547741f.
//
// Solidity: function revokeRole(bytes32 role, address account) returns()
func (_BeefyClient *BeefyClientTransactor) RevokeRole(opts *bind.TransactOpts, role [32]byte, account common.Address) (*types.Transaction, error) {
	return _BeefyClient.contract.Transact(opts, "revokeRole", role, account)
}

// RevokeRole is a paid mutator transaction binding the contract method 0xd547741f.
//
// Solidity: function revokeRole(bytes32 role, address account) returns()
func (_BeefyClient *BeefyClientSession) RevokeRole(role [32]byte, account common.Address) (*types.Transaction, error) {
	return _BeefyClient.Contract.RevokeRole(&_BeefyClient.TransactOpts, role, account)
}

// RevokeRole is a paid mutator transaction binding the contract method 0xd547741f.
//
// Solidity: function revokeRole(bytes32 role, address account) returns()
func (_BeefyClient *BeefyClientTransactorSession) RevokeRole(role [32]byte, account common.Address) (*types.Transaction, error) {
	return _BeefyClient.Contract.RevokeRole(&_BeefyClient.TransactOpts, role, account)
}

// BeefyClientCommitmentVerifiedIterator is returned from FilterCommitmentVerified and is used to iterate over the raw logs and unpacked data for CommitmentVerified events raised by the BeefyClient contract.
type BeefyClientCommitmentVerifiedIterator struct {
	Event *BeefyClientCommitmentVerified // Event containing the contract specifics and raw log

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
func (it *BeefyClientCommitmentVerifiedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(BeefyClientCommitmentVerified)
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
		it.Event = new(BeefyClientCommitmentVerified)
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
func (it *BeefyClientCommitmentVerifiedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *BeefyClientCommitmentVerifiedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// BeefyClientCommitmentVerified represents a CommitmentVerified event raised by the BeefyClient contract.
type BeefyClientCommitmentVerified struct {
	Id             *big.Int
	Phase          uint8
	CommitmentHash [32]byte
	Prover         common.Address
	Raw            types.Log // Blockchain specific contextual infos
}

// FilterCommitmentVerified is a free log retrieval operation binding the contract event 0x370ee36e1f982f6dd70d3bb1298a6c6d38b78990000b7f39caab64a0e9c74ade.
//
// Solidity: event CommitmentVerified(uint256 id, uint8 phase, bytes32 commitmentHash, address prover)
func (_BeefyClient *BeefyClientFilterer) FilterCommitmentVerified(opts *bind.FilterOpts) (*BeefyClientCommitmentVerifiedIterator, error) {

	logs, sub, err := _BeefyClient.contract.FilterLogs(opts, "CommitmentVerified")
	if err != nil {
		return nil, err
	}
	return &BeefyClientCommitmentVerifiedIterator{contract: _BeefyClient.contract, event: "CommitmentVerified", logs: logs, sub: sub}, nil
}

// WatchCommitmentVerified is a free log subscription operation binding the contract event 0x370ee36e1f982f6dd70d3bb1298a6c6d38b78990000b7f39caab64a0e9c74ade.
//
// Solidity: event CommitmentVerified(uint256 id, uint8 phase, bytes32 commitmentHash, address prover)
func (_BeefyClient *BeefyClientFilterer) WatchCommitmentVerified(opts *bind.WatchOpts, sink chan<- *BeefyClientCommitmentVerified) (event.Subscription, error) {

	logs, sub, err := _BeefyClient.contract.WatchLogs(opts, "CommitmentVerified")
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(BeefyClientCommitmentVerified)
				if err := _BeefyClient.contract.UnpackLog(event, "CommitmentVerified", log); err != nil {
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

// ParseCommitmentVerified is a log parse operation binding the contract event 0x370ee36e1f982f6dd70d3bb1298a6c6d38b78990000b7f39caab64a0e9c74ade.
//
// Solidity: event CommitmentVerified(uint256 id, uint8 phase, bytes32 commitmentHash, address prover)
func (_BeefyClient *BeefyClientFilterer) ParseCommitmentVerified(log types.Log) (*BeefyClientCommitmentVerified, error) {
	event := new(BeefyClientCommitmentVerified)
	if err := _BeefyClient.contract.UnpackLog(event, "CommitmentVerified", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// BeefyClientNewMMRRootIterator is returned from FilterNewMMRRoot and is used to iterate over the raw logs and unpacked data for NewMMRRoot events raised by the BeefyClient contract.
type BeefyClientNewMMRRootIterator struct {
	Event *BeefyClientNewMMRRoot // Event containing the contract specifics and raw log

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
func (it *BeefyClientNewMMRRootIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(BeefyClientNewMMRRoot)
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
		it.Event = new(BeefyClientNewMMRRoot)
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
func (it *BeefyClientNewMMRRootIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *BeefyClientNewMMRRootIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// BeefyClientNewMMRRoot represents a NewMMRRoot event raised by the BeefyClient contract.
type BeefyClientNewMMRRoot struct {
	MmrRoot     [32]byte
	BlockNumber uint64
	Raw         types.Log // Blockchain specific contextual infos
}

// FilterNewMMRRoot is a free log retrieval operation binding the contract event 0xd95fe1258d152dc91c81b09380498adc76ed36a6079bcb2ed31eff622ae2d0f1.
//
// Solidity: event NewMMRRoot(bytes32 mmrRoot, uint64 blockNumber)
func (_BeefyClient *BeefyClientFilterer) FilterNewMMRRoot(opts *bind.FilterOpts) (*BeefyClientNewMMRRootIterator, error) {

	logs, sub, err := _BeefyClient.contract.FilterLogs(opts, "NewMMRRoot")
	if err != nil {
		return nil, err
	}
	return &BeefyClientNewMMRRootIterator{contract: _BeefyClient.contract, event: "NewMMRRoot", logs: logs, sub: sub}, nil
}

// WatchNewMMRRoot is a free log subscription operation binding the contract event 0xd95fe1258d152dc91c81b09380498adc76ed36a6079bcb2ed31eff622ae2d0f1.
//
// Solidity: event NewMMRRoot(bytes32 mmrRoot, uint64 blockNumber)
func (_BeefyClient *BeefyClientFilterer) WatchNewMMRRoot(opts *bind.WatchOpts, sink chan<- *BeefyClientNewMMRRoot) (event.Subscription, error) {

	logs, sub, err := _BeefyClient.contract.WatchLogs(opts, "NewMMRRoot")
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(BeefyClientNewMMRRoot)
				if err := _BeefyClient.contract.UnpackLog(event, "NewMMRRoot", log); err != nil {
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
func (_BeefyClient *BeefyClientFilterer) ParseNewMMRRoot(log types.Log) (*BeefyClientNewMMRRoot, error) {
	event := new(BeefyClientNewMMRRoot)
	if err := _BeefyClient.contract.UnpackLog(event, "NewMMRRoot", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// BeefyClientRoleAdminChangedIterator is returned from FilterRoleAdminChanged and is used to iterate over the raw logs and unpacked data for RoleAdminChanged events raised by the BeefyClient contract.
type BeefyClientRoleAdminChangedIterator struct {
	Event *BeefyClientRoleAdminChanged // Event containing the contract specifics and raw log

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
func (it *BeefyClientRoleAdminChangedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(BeefyClientRoleAdminChanged)
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
		it.Event = new(BeefyClientRoleAdminChanged)
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
func (it *BeefyClientRoleAdminChangedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *BeefyClientRoleAdminChangedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// BeefyClientRoleAdminChanged represents a RoleAdminChanged event raised by the BeefyClient contract.
type BeefyClientRoleAdminChanged struct {
	Role              [32]byte
	PreviousAdminRole [32]byte
	NewAdminRole      [32]byte
	Raw               types.Log // Blockchain specific contextual infos
}

// FilterRoleAdminChanged is a free log retrieval operation binding the contract event 0xbd79b86ffe0ab8e8776151514217cd7cacd52c909f66475c3af44e129f0b00ff.
//
// Solidity: event RoleAdminChanged(bytes32 indexed role, bytes32 indexed previousAdminRole, bytes32 indexed newAdminRole)
func (_BeefyClient *BeefyClientFilterer) FilterRoleAdminChanged(opts *bind.FilterOpts, role [][32]byte, previousAdminRole [][32]byte, newAdminRole [][32]byte) (*BeefyClientRoleAdminChangedIterator, error) {

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

	logs, sub, err := _BeefyClient.contract.FilterLogs(opts, "RoleAdminChanged", roleRule, previousAdminRoleRule, newAdminRoleRule)
	if err != nil {
		return nil, err
	}
	return &BeefyClientRoleAdminChangedIterator{contract: _BeefyClient.contract, event: "RoleAdminChanged", logs: logs, sub: sub}, nil
}

// WatchRoleAdminChanged is a free log subscription operation binding the contract event 0xbd79b86ffe0ab8e8776151514217cd7cacd52c909f66475c3af44e129f0b00ff.
//
// Solidity: event RoleAdminChanged(bytes32 indexed role, bytes32 indexed previousAdminRole, bytes32 indexed newAdminRole)
func (_BeefyClient *BeefyClientFilterer) WatchRoleAdminChanged(opts *bind.WatchOpts, sink chan<- *BeefyClientRoleAdminChanged, role [][32]byte, previousAdminRole [][32]byte, newAdminRole [][32]byte) (event.Subscription, error) {

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

	logs, sub, err := _BeefyClient.contract.WatchLogs(opts, "RoleAdminChanged", roleRule, previousAdminRoleRule, newAdminRoleRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(BeefyClientRoleAdminChanged)
				if err := _BeefyClient.contract.UnpackLog(event, "RoleAdminChanged", log); err != nil {
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
func (_BeefyClient *BeefyClientFilterer) ParseRoleAdminChanged(log types.Log) (*BeefyClientRoleAdminChanged, error) {
	event := new(BeefyClientRoleAdminChanged)
	if err := _BeefyClient.contract.UnpackLog(event, "RoleAdminChanged", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// BeefyClientRoleGrantedIterator is returned from FilterRoleGranted and is used to iterate over the raw logs and unpacked data for RoleGranted events raised by the BeefyClient contract.
type BeefyClientRoleGrantedIterator struct {
	Event *BeefyClientRoleGranted // Event containing the contract specifics and raw log

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
func (it *BeefyClientRoleGrantedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(BeefyClientRoleGranted)
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
		it.Event = new(BeefyClientRoleGranted)
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
func (it *BeefyClientRoleGrantedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *BeefyClientRoleGrantedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// BeefyClientRoleGranted represents a RoleGranted event raised by the BeefyClient contract.
type BeefyClientRoleGranted struct {
	Role    [32]byte
	Account common.Address
	Sender  common.Address
	Raw     types.Log // Blockchain specific contextual infos
}

// FilterRoleGranted is a free log retrieval operation binding the contract event 0x2f8788117e7eff1d82e926ec794901d17c78024a50270940304540a733656f0d.
//
// Solidity: event RoleGranted(bytes32 indexed role, address indexed account, address indexed sender)
func (_BeefyClient *BeefyClientFilterer) FilterRoleGranted(opts *bind.FilterOpts, role [][32]byte, account []common.Address, sender []common.Address) (*BeefyClientRoleGrantedIterator, error) {

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

	logs, sub, err := _BeefyClient.contract.FilterLogs(opts, "RoleGranted", roleRule, accountRule, senderRule)
	if err != nil {
		return nil, err
	}
	return &BeefyClientRoleGrantedIterator{contract: _BeefyClient.contract, event: "RoleGranted", logs: logs, sub: sub}, nil
}

// WatchRoleGranted is a free log subscription operation binding the contract event 0x2f8788117e7eff1d82e926ec794901d17c78024a50270940304540a733656f0d.
//
// Solidity: event RoleGranted(bytes32 indexed role, address indexed account, address indexed sender)
func (_BeefyClient *BeefyClientFilterer) WatchRoleGranted(opts *bind.WatchOpts, sink chan<- *BeefyClientRoleGranted, role [][32]byte, account []common.Address, sender []common.Address) (event.Subscription, error) {

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

	logs, sub, err := _BeefyClient.contract.WatchLogs(opts, "RoleGranted", roleRule, accountRule, senderRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(BeefyClientRoleGranted)
				if err := _BeefyClient.contract.UnpackLog(event, "RoleGranted", log); err != nil {
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
func (_BeefyClient *BeefyClientFilterer) ParseRoleGranted(log types.Log) (*BeefyClientRoleGranted, error) {
	event := new(BeefyClientRoleGranted)
	if err := _BeefyClient.contract.UnpackLog(event, "RoleGranted", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}

// BeefyClientRoleRevokedIterator is returned from FilterRoleRevoked and is used to iterate over the raw logs and unpacked data for RoleRevoked events raised by the BeefyClient contract.
type BeefyClientRoleRevokedIterator struct {
	Event *BeefyClientRoleRevoked // Event containing the contract specifics and raw log

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
func (it *BeefyClientRoleRevokedIterator) Next() bool {
	// If the iterator failed, stop iterating
	if it.fail != nil {
		return false
	}
	// If the iterator completed, deliver directly whatever's available
	if it.done {
		select {
		case log := <-it.logs:
			it.Event = new(BeefyClientRoleRevoked)
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
		it.Event = new(BeefyClientRoleRevoked)
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
func (it *BeefyClientRoleRevokedIterator) Error() error {
	return it.fail
}

// Close terminates the iteration process, releasing any pending underlying
// resources.
func (it *BeefyClientRoleRevokedIterator) Close() error {
	it.sub.Unsubscribe()
	return nil
}

// BeefyClientRoleRevoked represents a RoleRevoked event raised by the BeefyClient contract.
type BeefyClientRoleRevoked struct {
	Role    [32]byte
	Account common.Address
	Sender  common.Address
	Raw     types.Log // Blockchain specific contextual infos
}

// FilterRoleRevoked is a free log retrieval operation binding the contract event 0xf6391f5c32d9c69d2a47ea670b442974b53935d1edc7fd64eb21e047a839171b.
//
// Solidity: event RoleRevoked(bytes32 indexed role, address indexed account, address indexed sender)
func (_BeefyClient *BeefyClientFilterer) FilterRoleRevoked(opts *bind.FilterOpts, role [][32]byte, account []common.Address, sender []common.Address) (*BeefyClientRoleRevokedIterator, error) {

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

	logs, sub, err := _BeefyClient.contract.FilterLogs(opts, "RoleRevoked", roleRule, accountRule, senderRule)
	if err != nil {
		return nil, err
	}
	return &BeefyClientRoleRevokedIterator{contract: _BeefyClient.contract, event: "RoleRevoked", logs: logs, sub: sub}, nil
}

// WatchRoleRevoked is a free log subscription operation binding the contract event 0xf6391f5c32d9c69d2a47ea670b442974b53935d1edc7fd64eb21e047a839171b.
//
// Solidity: event RoleRevoked(bytes32 indexed role, address indexed account, address indexed sender)
func (_BeefyClient *BeefyClientFilterer) WatchRoleRevoked(opts *bind.WatchOpts, sink chan<- *BeefyClientRoleRevoked, role [][32]byte, account []common.Address, sender []common.Address) (event.Subscription, error) {

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

	logs, sub, err := _BeefyClient.contract.WatchLogs(opts, "RoleRevoked", roleRule, accountRule, senderRule)
	if err != nil {
		return nil, err
	}
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case log := <-logs:
				// New log arrived, parse the event and forward to the user
				event := new(BeefyClientRoleRevoked)
				if err := _BeefyClient.contract.UnpackLog(event, "RoleRevoked", log); err != nil {
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
func (_BeefyClient *BeefyClientFilterer) ParseRoleRevoked(log types.Log) (*BeefyClientRoleRevoked, error) {
	event := new(BeefyClientRoleRevoked)
	if err := _BeefyClient.contract.UnpackLog(event, "RoleRevoked", log); err != nil {
		return nil, err
	}
	event.Raw = log
	return event, nil
}
