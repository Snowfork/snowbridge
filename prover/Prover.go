package prover

import (
	"github.com/ethereum/go-ethereum/common"

	"github.com/ethereum/go-ethereum/core/types"
)

// Prover can verify transactions via a SPV Proof or Threshold Vote
type Prover interface {
	ProofVerifier
	ThresholdVerifier

	// VerifyTransaction passes a transaction to ProofVerifier of ThresholdVerifier
	VerifyTransaction(verificationData VerificationData) (bool, error)
}

// VerificationData is the base interface for different verification strategies
type VerificationData interface{}

// ProofVerifier verifies transactions via a SPV Proof
type ProofVerifier interface {
	// BuildProof builds a SPV proof
	BuildProof(block types.Block, txHash string) SpvProof

	// VerifyProof verifies a SPV proof by proving commitment to its root (not to block hash)
	VerifyProof(merklePath string, tx string, parentNodes string, header string, blockHash string) bool
}

// LightClientProof supports cryptographic verification
type LightClientProof struct {
	verificationData VerificationData
	proof            SpvProof // An SPV proof
}

// SpvProof contains information used to verify that a transaction is included in a specific block
type SpvProof struct {
	blockHash   common.Hash  // Block hash
	blockHeader types.Header // Block header
	parentNodes []string
	txIndex     int           // Transaction's position in a block
	txReceipt   types.Receipt // Raw transaction receipt
}

// ThresholdVerifier verifies transactions via a Threshold Vote
type ThresholdVerifier interface {
	// IsValidSignature validates that a given signature is the given validator for an individual transaction
	IsValidSignature(tx types.Transaction, signature string, validator common.Address) (bool, error)

	// VerifyVote validates that enough validators have signed a transaction to meet the threshold requirements
	VerifyVote(tx types.Transaction, votes []Vote) bool

	// UpdateValset updates the current validators and their powers
	UpdateValset(valset map[common.Address]int) error

	// SetThreshold sets the percentage threshold of total signed power required for a successful vote
	SetThreshold(threshold int) error

	// GetValidators gets the current list of validators
	GetValidators() []common.Address

	// IsValidator a boolean indicating if the given address is validator
	IsValidator(validator common.Address) bool

	// GetPower gets a validator's current power
	GetPower(validator common.Address) int
}

// ThresholdVote supports proof-of-stake based validation
type ThresholdVote struct {
	verificationData VerificationData
	votes            []Vote // A collection of votes
}

// Vote struct contains a validator address and their signature on a data payload
type Vote struct {
	address   common.Address         // Validator's address used create the signature
	payload   map[string]interface{} // The data payload this vote is being made upon
	signature string                 // Signature of validator on the message
}
