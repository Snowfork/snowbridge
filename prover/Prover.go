package prover

import (
	"crypto/ecdsa"
	"encoding/hex"

	"github.com/ethereum/go-ethereum/crypto"
	solsha3 "github.com/miguelmota/go-solidity-sha3"
)

// Prover generates proofs
type Prover interface {
	// GenerateProof generates a new proof that can be used to verify transactions
	GenerateProof(data []byte, privateKey interface{}) Proof
}

// Proof contains information for verifying a signature
type Proof struct {
	Hash      []byte
	Signature []byte
}

// NewProof initializes a new instance of Proof
func NewProof(hash, signature []byte) Proof {
	return Proof{
		Hash:      hash,
		Signature: signature,
	}
}

// GenerateProof creates a new proof by signing a data hash with a private key
func GenerateProof(data []byte, pk *ecdsa.PrivateKey) (Proof, error) {
	// Turn the message into a 32-byte hash
	hash := solsha3.SoliditySHA3(solsha3.String(data))
	// Prefix and then hash to mimic behavior of eth_sign
	prefixed := solsha3.SoliditySHA3(solsha3.String("\x19Ethereum Signed Message:\n32" + hex.EncodeToString(hash)))

	signature, err := crypto.Sign(prefixed, pk)
	if err != nil {
		return Proof{}, err
	}

	proof := NewProof(prefixed, signature)
	return proof, nil
}
