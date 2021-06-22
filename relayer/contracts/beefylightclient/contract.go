// Code generated - DO NOT EDIT.
// This file is a generated binding and any manual changes will be lost.

package beefylightclient

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

// BeefyLightClientCommitment is an auto generated low-level Go binding around an user-defined struct.
type BeefyLightClientCommitment struct {
	Payload        [32]byte
	BlockNumber    uint64
	ValidatorSetId uint32
}

// ContractABI is the input ABI used to generate the binding from.
const ContractABI = "[{\"inputs\":[{\"internalType\":\"contractValidatorRegistry\",\"name\":\"_validatorRegistry\",\"type\":\"address\"},{\"internalType\":\"contractMMRVerification\",\"name\":\"_mmrVerification\",\"type\":\"address\"},{\"internalType\":\"contractBlake2b\",\"name\":\"_blake2b\",\"type\":\"address\"}],\"stateMutability\":\"nonpayable\",\"type\":\"constructor\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":false,\"internalType\":\"address\",\"name\":\"prover\",\"type\":\"address\"},{\"indexed\":false,\"internalType\":\"bytes32\",\"name\":\"commitmentHash\",\"type\":\"bytes32\"},{\"indexed\":false,\"internalType\":\"uint256\",\"name\":\"id\",\"type\":\"uint256\"}],\"name\":\"FinalVerificationSuccessful\",\"type\":\"event\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":false,\"internalType\":\"address\",\"name\":\"prover\",\"type\":\"address\"},{\"indexed\":false,\"internalType\":\"uint256\",\"name\":\"blockNumber\",\"type\":\"uint256\"},{\"indexed\":false,\"internalType\":\"uint256\",\"name\":\"id\",\"type\":\"uint256\"}],\"name\":\"InitialVerificationSuccessful\",\"type\":\"event\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":false,\"internalType\":\"bytes32\",\"name\":\"mmrRoot\",\"type\":\"bytes32\"},{\"indexed\":false,\"internalType\":\"uint64\",\"name\":\"blockNumber\",\"type\":\"uint64\"}],\"name\":\"NewMMRRoot\",\"type\":\"event\"},{\"inputs\":[],\"name\":\"BLOCK_WAIT_PERIOD\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"THRESHOLD_DENOMINATOR\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"THRESHOLD_NUMERATOR\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"blake2b\",\"outputs\":[{\"internalType\":\"contractBlake2b\",\"name\":\"\",\"type\":\"address\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"currentId\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"latestMMRRoot\",\"outputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"mmrVerification\",\"outputs\":[{\"internalType\":\"contractMMRVerification\",\"name\":\"\",\"type\":\"address\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\"}],\"name\":\"validationData\",\"outputs\":[{\"internalType\":\"address\",\"name\":\"senderAddress\",\"type\":\"address\"},{\"internalType\":\"bytes32\",\"name\":\"commitmentHash\",\"type\":\"bytes32\"},{\"internalType\":\"uint256\",\"name\":\"blockNumber\",\"type\":\"uint256\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"validatorRegistry\",\"outputs\":[{\"internalType\":\"contractValidatorRegistry\",\"name\":\"\",\"type\":\"address\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"beefyMMRLeaf\",\"type\":\"bytes32\"},{\"internalType\":\"uint256\",\"name\":\"beefyMMRLeafIndex\",\"type\":\"uint256\"},{\"internalType\":\"uint256\",\"name\":\"beefyMMRLeafCount\",\"type\":\"uint256\"},{\"internalType\":\"bytes32[]\",\"name\":\"beefyMMRLeafProof\",\"type\":\"bytes32[]\"}],\"name\":\"verifyBeefyMerkleLeaf\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\"}],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"commitmentHash\",\"type\":\"bytes32\"},{\"internalType\":\"uint256[]\",\"name\":\"validatorClaimsBitfield\",\"type\":\"uint256[]\"},{\"internalType\":\"bytes\",\"name\":\"validatorSignature\",\"type\":\"bytes\"},{\"internalType\":\"uint256\",\"name\":\"validatorPosition\",\"type\":\"uint256\"},{\"internalType\":\"address\",\"name\":\"validatorPublicKey\",\"type\":\"address\"},{\"internalType\":\"bytes32[]\",\"name\":\"validatorPublicKeyMerkleProof\",\"type\":\"bytes32[]\"}],\"name\":\"newSignatureCommitment\",\"outputs\":[],\"stateMutability\":\"payable\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"id\",\"type\":\"uint256\"}],\"name\":\"createRandomBitfield\",\"outputs\":[{\"internalType\":\"uint256[]\",\"name\":\"\",\"type\":\"uint256[]\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"uint256[]\",\"name\":\"bitsToSet\",\"type\":\"uint256[]\"},{\"internalType\":\"uint256\",\"name\":\"length\",\"type\":\"uint256\"}],\"name\":\"createInitialBitfield\",\"outputs\":[{\"internalType\":\"uint256[]\",\"name\":\"\",\"type\":\"uint256[]\"}],\"stateMutability\":\"pure\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"id\",\"type\":\"uint256\"},{\"components\":[{\"internalType\":\"bytes32\",\"name\":\"payload\",\"type\":\"bytes32\"},{\"internalType\":\"uint64\",\"name\":\"blockNumber\",\"type\":\"uint64\"},{\"internalType\":\"uint32\",\"name\":\"validatorSetId\",\"type\":\"uint32\"}],\"internalType\":\"structBeefyLightClient.Commitment\",\"name\":\"commitment\",\"type\":\"tuple\"},{\"internalType\":\"bytes[]\",\"name\":\"signatures\",\"type\":\"bytes[]\"},{\"internalType\":\"uint256[]\",\"name\":\"validatorPositions\",\"type\":\"uint256[]\"},{\"internalType\":\"address[]\",\"name\":\"validatorPublicKeys\",\"type\":\"address[]\"},{\"internalType\":\"bytes32[][]\",\"name\":\"validatorPublicKeyMerkleProofs\",\"type\":\"bytes32[][]\"}],\"name\":\"completeSignatureCommitment\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"requiredNumberOfSignatures\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\"}],\"stateMutability\":\"view\",\"type\":\"function\"}]"

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
// Solidity: function BLOCK_WAIT_PERIOD() view returns(uint256)
func (_Contract *ContractCaller) BLOCKWAITPERIOD(opts *bind.CallOpts) (*big.Int, error) {
	var out []interface{}
	err := _Contract.contract.Call(opts, &out, "BLOCK_WAIT_PERIOD")

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// BLOCKWAITPERIOD is a free data retrieval call binding the contract method 0xfb752c62.
//
// Solidity: function BLOCK_WAIT_PERIOD() view returns(uint256)
func (_Contract *ContractSession) BLOCKWAITPERIOD() (*big.Int, error) {
	return _Contract.Contract.BLOCKWAITPERIOD(&_Contract.CallOpts)
}

// BLOCKWAITPERIOD is a free data retrieval call binding the contract method 0xfb752c62.
//
// Solidity: function BLOCK_WAIT_PERIOD() view returns(uint256)
func (_Contract *ContractCallerSession) BLOCKWAITPERIOD() (*big.Int, error) {
	return _Contract.Contract.BLOCKWAITPERIOD(&_Contract.CallOpts)
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

// Blake2b is a free data retrieval call binding the contract method 0x6e2dc392.
//
// Solidity: function blake2b() view returns(address)
func (_Contract *ContractCaller) Blake2b(opts *bind.CallOpts) (common.Address, error) {
	var out []interface{}
	err := _Contract.contract.Call(opts, &out, "blake2b")

	if err != nil {
		return *new(common.Address), err
	}

	out0 := *abi.ConvertType(out[0], new(common.Address)).(*common.Address)

	return out0, err

}

// Blake2b is a free data retrieval call binding the contract method 0x6e2dc392.
//
// Solidity: function blake2b() view returns(address)
func (_Contract *ContractSession) Blake2b() (common.Address, error) {
	return _Contract.Contract.Blake2b(&_Contract.CallOpts)
}

// Blake2b is a free data retrieval call binding the contract method 0x6e2dc392.
//
// Solidity: function blake2b() view returns(address)
func (_Contract *ContractCallerSession) Blake2b() (common.Address, error) {
	return _Contract.Contract.Blake2b(&_Contract.CallOpts)
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

// CurrentId is a free data retrieval call binding the contract method 0xe00dd161.
//
// Solidity: function currentId() view returns(uint256)
func (_Contract *ContractCaller) CurrentId(opts *bind.CallOpts) (*big.Int, error) {
	var out []interface{}
	err := _Contract.contract.Call(opts, &out, "currentId")

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// CurrentId is a free data retrieval call binding the contract method 0xe00dd161.
//
// Solidity: function currentId() view returns(uint256)
func (_Contract *ContractSession) CurrentId() (*big.Int, error) {
	return _Contract.Contract.CurrentId(&_Contract.CallOpts)
}

// CurrentId is a free data retrieval call binding the contract method 0xe00dd161.
//
// Solidity: function currentId() view returns(uint256)
func (_Contract *ContractCallerSession) CurrentId() (*big.Int, error) {
	return _Contract.Contract.CurrentId(&_Contract.CallOpts)
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

// RequiredNumberOfSignatures is a free data retrieval call binding the contract method 0x72fe1a9f.
//
// Solidity: function requiredNumberOfSignatures() view returns(uint256)
func (_Contract *ContractCaller) RequiredNumberOfSignatures(opts *bind.CallOpts) (*big.Int, error) {
	var out []interface{}
	err := _Contract.contract.Call(opts, &out, "requiredNumberOfSignatures")

	if err != nil {
		return *new(*big.Int), err
	}

	out0 := *abi.ConvertType(out[0], new(*big.Int)).(**big.Int)

	return out0, err

}

// RequiredNumberOfSignatures is a free data retrieval call binding the contract method 0x72fe1a9f.
//
// Solidity: function requiredNumberOfSignatures() view returns(uint256)
func (_Contract *ContractSession) RequiredNumberOfSignatures() (*big.Int, error) {
	return _Contract.Contract.RequiredNumberOfSignatures(&_Contract.CallOpts)
}

// RequiredNumberOfSignatures is a free data retrieval call binding the contract method 0x72fe1a9f.
//
// Solidity: function requiredNumberOfSignatures() view returns(uint256)
func (_Contract *ContractCallerSession) RequiredNumberOfSignatures() (*big.Int, error) {
	return _Contract.Contract.RequiredNumberOfSignatures(&_Contract.CallOpts)
}

// ValidationData is a free data retrieval call binding the contract method 0x20bfa5cb.
//
// Solidity: function validationData(uint256 ) view returns(address senderAddress, bytes32 commitmentHash, uint256 blockNumber)
func (_Contract *ContractCaller) ValidationData(opts *bind.CallOpts, arg0 *big.Int) (struct {
	SenderAddress  common.Address
	CommitmentHash [32]byte
	BlockNumber    *big.Int
}, error) {
	var out []interface{}
	err := _Contract.contract.Call(opts, &out, "validationData", arg0)

	outstruct := new(struct {
		SenderAddress  common.Address
		CommitmentHash [32]byte
		BlockNumber    *big.Int
	})

	outstruct.SenderAddress = out[0].(common.Address)
	outstruct.CommitmentHash = out[1].([32]byte)
	outstruct.BlockNumber = out[2].(*big.Int)

	return *outstruct, err

}

// ValidationData is a free data retrieval call binding the contract method 0x20bfa5cb.
//
// Solidity: function validationData(uint256 ) view returns(address senderAddress, bytes32 commitmentHash, uint256 blockNumber)
func (_Contract *ContractSession) ValidationData(arg0 *big.Int) (struct {
	SenderAddress  common.Address
	CommitmentHash [32]byte
	BlockNumber    *big.Int
}, error) {
	return _Contract.Contract.ValidationData(&_Contract.CallOpts, arg0)
}

// ValidationData is a free data retrieval call binding the contract method 0x20bfa5cb.
//
// Solidity: function validationData(uint256 ) view returns(address senderAddress, bytes32 commitmentHash, uint256 blockNumber)
func (_Contract *ContractCallerSession) ValidationData(arg0 *big.Int) (struct {
	SenderAddress  common.Address
	CommitmentHash [32]byte
	BlockNumber    *big.Int
}, error) {
	return _Contract.Contract.ValidationData(&_Contract.CallOpts, arg0)
}

// ValidatorRegistry is a free data retrieval call binding the contract method 0xf376ebbb.
//
// Solidity: function validatorRegistry() view returns(address)
func (_Contract *ContractCaller) ValidatorRegistry(opts *bind.CallOpts) (common.Address, error) {
	var out []interface{}
	err := _Contract.contract.Call(opts, &out, "validatorRegistry")

	if err != nil {
		return *new(common.Address), err
	}

	out0 := *abi.ConvertType(out[0], new(common.Address)).(*common.Address)

	return out0, err

}

// ValidatorRegistry is a free data retrieval call binding the contract method 0xf376ebbb.
//
// Solidity: function validatorRegistry() view returns(address)
func (_Contract *ContractSession) ValidatorRegistry() (common.Address, error) {
	return _Contract.Contract.ValidatorRegistry(&_Contract.CallOpts)
}

// ValidatorRegistry is a free data retrieval call binding the contract method 0xf376ebbb.
//
// Solidity: function validatorRegistry() view returns(address)
func (_Contract *ContractCallerSession) ValidatorRegistry() (common.Address, error) {
	return _Contract.Contract.ValidatorRegistry(&_Contract.CallOpts)
}

// CompleteSignatureCommitment is a paid mutator transaction binding the contract method 0x18592da5.
//
// Solidity: function completeSignatureCommitment(uint256 id, (bytes32,uint64,uint32) commitment, bytes[] signatures, uint256[] validatorPositions, address[] validatorPublicKeys, bytes32[][] validatorPublicKeyMerkleProofs) returns()
func (_Contract *ContractTransactor) CompleteSignatureCommitment(opts *bind.TransactOpts, id *big.Int, commitment BeefyLightClientCommitment, signatures [][]byte, validatorPositions []*big.Int, validatorPublicKeys []common.Address, validatorPublicKeyMerkleProofs [][][32]byte) (*types.Transaction, error) {
	return _Contract.contract.Transact(opts, "completeSignatureCommitment", id, commitment, signatures, validatorPositions, validatorPublicKeys, validatorPublicKeyMerkleProofs)
}

// CompleteSignatureCommitment is a paid mutator transaction binding the contract method 0x18592da5.
//
// Solidity: function completeSignatureCommitment(uint256 id, (bytes32,uint64,uint32) commitment, bytes[] signatures, uint256[] validatorPositions, address[] validatorPublicKeys, bytes32[][] validatorPublicKeyMerkleProofs) returns()
func (_Contract *ContractSession) CompleteSignatureCommitment(id *big.Int, commitment BeefyLightClientCommitment, signatures [][]byte, validatorPositions []*big.Int, validatorPublicKeys []common.Address, validatorPublicKeyMerkleProofs [][][32]byte) (*types.Transaction, error) {
	return _Contract.Contract.CompleteSignatureCommitment(&_Contract.TransactOpts, id, commitment, signatures, validatorPositions, validatorPublicKeys, validatorPublicKeyMerkleProofs)
}

// CompleteSignatureCommitment is a paid mutator transaction binding the contract method 0x18592da5.
//
// Solidity: function completeSignatureCommitment(uint256 id, (bytes32,uint64,uint32) commitment, bytes[] signatures, uint256[] validatorPositions, address[] validatorPublicKeys, bytes32[][] validatorPublicKeyMerkleProofs) returns()
func (_Contract *ContractTransactorSession) CompleteSignatureCommitment(id *big.Int, commitment BeefyLightClientCommitment, signatures [][]byte, validatorPositions []*big.Int, validatorPublicKeys []common.Address, validatorPublicKeyMerkleProofs [][][32]byte) (*types.Transaction, error) {
	return _Contract.Contract.CompleteSignatureCommitment(&_Contract.TransactOpts, id, commitment, signatures, validatorPositions, validatorPublicKeys, validatorPublicKeyMerkleProofs)
}

// NewSignatureCommitment is a paid mutator transaction binding the contract method 0xe54d1543.
//
// Solidity: function newSignatureCommitment(bytes32 commitmentHash, uint256[] validatorClaimsBitfield, bytes validatorSignature, uint256 validatorPosition, address validatorPublicKey, bytes32[] validatorPublicKeyMerkleProof) payable returns()
func (_Contract *ContractTransactor) NewSignatureCommitment(opts *bind.TransactOpts, commitmentHash [32]byte, validatorClaimsBitfield []*big.Int, validatorSignature []byte, validatorPosition *big.Int, validatorPublicKey common.Address, validatorPublicKeyMerkleProof [][32]byte) (*types.Transaction, error) {
	return _Contract.contract.Transact(opts, "newSignatureCommitment", commitmentHash, validatorClaimsBitfield, validatorSignature, validatorPosition, validatorPublicKey, validatorPublicKeyMerkleProof)
}

// NewSignatureCommitment is a paid mutator transaction binding the contract method 0xe54d1543.
//
// Solidity: function newSignatureCommitment(bytes32 commitmentHash, uint256[] validatorClaimsBitfield, bytes validatorSignature, uint256 validatorPosition, address validatorPublicKey, bytes32[] validatorPublicKeyMerkleProof) payable returns()
func (_Contract *ContractSession) NewSignatureCommitment(commitmentHash [32]byte, validatorClaimsBitfield []*big.Int, validatorSignature []byte, validatorPosition *big.Int, validatorPublicKey common.Address, validatorPublicKeyMerkleProof [][32]byte) (*types.Transaction, error) {
	return _Contract.Contract.NewSignatureCommitment(&_Contract.TransactOpts, commitmentHash, validatorClaimsBitfield, validatorSignature, validatorPosition, validatorPublicKey, validatorPublicKeyMerkleProof)
}

// NewSignatureCommitment is a paid mutator transaction binding the contract method 0xe54d1543.
//
// Solidity: function newSignatureCommitment(bytes32 commitmentHash, uint256[] validatorClaimsBitfield, bytes validatorSignature, uint256 validatorPosition, address validatorPublicKey, bytes32[] validatorPublicKeyMerkleProof) payable returns()
func (_Contract *ContractTransactorSession) NewSignatureCommitment(commitmentHash [32]byte, validatorClaimsBitfield []*big.Int, validatorSignature []byte, validatorPosition *big.Int, validatorPublicKey common.Address, validatorPublicKeyMerkleProof [][32]byte) (*types.Transaction, error) {
	return _Contract.Contract.NewSignatureCommitment(&_Contract.TransactOpts, commitmentHash, validatorClaimsBitfield, validatorSignature, validatorPosition, validatorPublicKey, validatorPublicKeyMerkleProof)
}

// VerifyBeefyMerkleLeaf is a paid mutator transaction binding the contract method 0x2c307093.
//
// Solidity: function verifyBeefyMerkleLeaf(bytes32 beefyMMRLeaf, uint256 beefyMMRLeafIndex, uint256 beefyMMRLeafCount, bytes32[] beefyMMRLeafProof) returns(bool)
func (_Contract *ContractTransactor) VerifyBeefyMerkleLeaf(opts *bind.TransactOpts, beefyMMRLeaf [32]byte, beefyMMRLeafIndex *big.Int, beefyMMRLeafCount *big.Int, beefyMMRLeafProof [][32]byte) (*types.Transaction, error) {
	return _Contract.contract.Transact(opts, "verifyBeefyMerkleLeaf", beefyMMRLeaf, beefyMMRLeafIndex, beefyMMRLeafCount, beefyMMRLeafProof)
}

// VerifyBeefyMerkleLeaf is a paid mutator transaction binding the contract method 0x2c307093.
//
// Solidity: function verifyBeefyMerkleLeaf(bytes32 beefyMMRLeaf, uint256 beefyMMRLeafIndex, uint256 beefyMMRLeafCount, bytes32[] beefyMMRLeafProof) returns(bool)
func (_Contract *ContractSession) VerifyBeefyMerkleLeaf(beefyMMRLeaf [32]byte, beefyMMRLeafIndex *big.Int, beefyMMRLeafCount *big.Int, beefyMMRLeafProof [][32]byte) (*types.Transaction, error) {
	return _Contract.Contract.VerifyBeefyMerkleLeaf(&_Contract.TransactOpts, beefyMMRLeaf, beefyMMRLeafIndex, beefyMMRLeafCount, beefyMMRLeafProof)
}

// VerifyBeefyMerkleLeaf is a paid mutator transaction binding the contract method 0x2c307093.
//
// Solidity: function verifyBeefyMerkleLeaf(bytes32 beefyMMRLeaf, uint256 beefyMMRLeafIndex, uint256 beefyMMRLeafCount, bytes32[] beefyMMRLeafProof) returns(bool)
func (_Contract *ContractTransactorSession) VerifyBeefyMerkleLeaf(beefyMMRLeaf [32]byte, beefyMMRLeafIndex *big.Int, beefyMMRLeafCount *big.Int, beefyMMRLeafProof [][32]byte) (*types.Transaction, error) {
	return _Contract.Contract.VerifyBeefyMerkleLeaf(&_Contract.TransactOpts, beefyMMRLeaf, beefyMMRLeafIndex, beefyMMRLeafCount, beefyMMRLeafProof)
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
	Prover         common.Address
	CommitmentHash [32]byte
	Id             *big.Int
	Raw            types.Log // Blockchain specific contextual infos
}

// FilterFinalVerificationSuccessful is a free log retrieval operation binding the contract event 0x064680bd49185bfe9d29035c0d5caec57f27902d4f2037faa81246a9d686a106.
//
// Solidity: event FinalVerificationSuccessful(address prover, bytes32 commitmentHash, uint256 id)
func (_Contract *ContractFilterer) FilterFinalVerificationSuccessful(opts *bind.FilterOpts) (*ContractFinalVerificationSuccessfulIterator, error) {

	logs, sub, err := _Contract.contract.FilterLogs(opts, "FinalVerificationSuccessful")
	if err != nil {
		return nil, err
	}
	return &ContractFinalVerificationSuccessfulIterator{contract: _Contract.contract, event: "FinalVerificationSuccessful", logs: logs, sub: sub}, nil
}

// WatchFinalVerificationSuccessful is a free log subscription operation binding the contract event 0x064680bd49185bfe9d29035c0d5caec57f27902d4f2037faa81246a9d686a106.
//
// Solidity: event FinalVerificationSuccessful(address prover, bytes32 commitmentHash, uint256 id)
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

// ParseFinalVerificationSuccessful is a log parse operation binding the contract event 0x064680bd49185bfe9d29035c0d5caec57f27902d4f2037faa81246a9d686a106.
//
// Solidity: event FinalVerificationSuccessful(address prover, bytes32 commitmentHash, uint256 id)
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
