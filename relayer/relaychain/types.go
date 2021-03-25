// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package relaychain

import (
	"encoding/binary"
	"encoding/hex"
	"fmt"
	"math/big"

	"github.com/ethereum/go-ethereum/common"
	merkletree "github.com/wealdtech/go-merkletree"
	"golang.org/x/crypto/blake2b"
	"golang.org/x/crypto/sha3"

	"github.com/snowfork/go-substrate-rpc-client/v2/scale"
	"github.com/snowfork/go-substrate-rpc-client/v2/types"
)

type Status int

const (
	CommitmentWitnessed            Status = iota // 0
	InitialVerificationTxSent      Status = iota // 1
	InitialVerificationTxConfirmed Status = iota // 2
	ReadyToComplete                Status = iota // 3
)

type BeefyCommitmentInfo struct {
	ValidatorAddresses        []common.Address
	SignedCommitment          *SignedCommitment
	Status                    Status
	InitialVerificationTxHash common.Hash
	CompleteOnBlock           uint64
}

func NewBeefyCommitmentInfo(valAddrs []common.Address, commitment *SignedCommitment) BeefyCommitmentInfo {
	return BeefyCommitmentInfo{
		ValidatorAddresses: valAddrs,
		SignedCommitment:   commitment,
		Status:             CommitmentWitnessed,
	}
}

type NewSignatureCommitmentMessage struct {
	Payload                       [32]byte
	ValidatorClaimsBitfield       []*big.Int
	ValidatorSignatureCommitment  []byte
	ValidatorPosition             *big.Int
	ValidatorPublicKey            common.Address
	ValidatorPublicKeyMerkleProof [][32]byte
}

type CompleteSignatureCommitmentMessage struct {
	ID                               *big.Int
	Payload                          [32]byte
	RandomSignatureCommitments       [][]byte
	RandomSignatureBitfieldPositions []*big.Int
	RandomValidatorAddresses         []common.Address
	RandomPublicKeyMerkleProofs      [][][32]byte
}

func (b BeefyCommitmentInfo) BuildNewSignatureCommitmentMessage(valAddrIndex int) (NewSignatureCommitmentMessage, error) {
	sig0ProofContents, err := b.GenerateMmrProofOffchain(valAddrIndex)
	if err != nil {
		return NewSignatureCommitmentMessage{}, err
	}

	// Update signature format (Polkadot uses recovery IDs 0 or 1, Eth uses 27 or 28, so we need to add 27)
	ok, sig0 := b.SignedCommitment.Signatures[valAddrIndex].Unwrap()
	if !ok {
		return NewSignatureCommitmentMessage{}, fmt.Errorf("failed to unwrap signature")
	}
	sig0Commitment := sig0[:] // TODO: increment recovery ID properly

	commitmentHash := blake2b.Sum256(b.SignedCommitment.Commitment.Bytes())

	validatorClaimsBitfield := []*big.Int{big.NewInt(123)} // TODO: add bitfield stuff properly

	msg := NewSignatureCommitmentMessage{
		Payload:                       commitmentHash,
		ValidatorClaimsBitfield:       validatorClaimsBitfield,
		ValidatorSignatureCommitment:  sig0Commitment,
		ValidatorPublicKey:            b.ValidatorAddresses[valAddrIndex],
		ValidatorPosition:             big.NewInt(int64(valAddrIndex)),
		ValidatorPublicKeyMerkleProof: sig0ProofContents,
	}

	return msg, nil
}

func (b BeefyCommitmentInfo) GenerateMmrProofOffchain(valAddrIndex int) ([][32]byte, error) {
	// Hash validator addresses for leaf input data
	beefyTreeData := make([][]byte, len(b.ValidatorAddresses))
	for i, valAddr := range b.ValidatorAddresses {
		hash := sha3.New256()
		if _, err := hash.Write(valAddr.Bytes()); err != nil {
			return [][32]byte{}, err
		}
		buf := hash.Sum(nil)
		beefyTreeData[i] = buf
	}

	// Create the tree
	beefyMerkleTree, err := merkletree.New(beefyTreeData)
	if err != nil {
		return [][32]byte{}, err
	}

	// Generate MMR proof for validator at index valAddrIndex
	sigProof, err := beefyMerkleTree.GenerateProof(beefyTreeData[valAddrIndex])
	if err != nil {
		return [][32]byte{}, err
	}

	fmt.Println("sigProof:", sigProof)

	// Verify the proof
	root := beefyMerkleTree.Root()
	fmt.Println("root:", root)

	verified, err := merkletree.VerifyProof(beefyTreeData[valAddrIndex], sigProof, root)
	if err != nil {
		return [][32]byte{}, err
	}
	if !verified {
		return [][32]byte{}, fmt.Errorf("failed to verify proof")
	}

	hexRoot := hex.EncodeToString(root)
	fmt.Println("hexRoot:", hexRoot)

	sigProofContents := make([][32]byte, len(sigProof.Hashes))
	for i, hash := range sigProof.Hashes {
		var hash32Byte [32]byte
		copy(hash32Byte[:], hash)
		sigProofContents[i] = hash32Byte
	}

	return sigProofContents, nil
}

func (b BeefyCommitmentInfo) BuildCompleteSignatureCommitmentMessage() (CompleteSignatureCommitmentMessage, error) {
	commitmentHash := blake2b.Sum256(b.SignedCommitment.Commitment.Bytes())

	validationDataID := big.NewInt(int64(b.SignedCommitment.Commitment.ValidatorSetID))

	//TODO: Generate randomSignatureBitfieldPositions properly
	randomSignatureBitfieldPositions := []*big.Int{}

	//TODO: Populate randomSignatureCommitments, randomValidatorAddresses, and based on randomSignatureBitfieldPositions
	randomSignatureCommitments := [][]byte{}
	randomValidatorAddresses := b.ValidatorAddresses
	randomPublicKeyMerkleProofs := [][][32]byte{}

	msg := CompleteSignatureCommitmentMessage{
		ID:                               validationDataID,
		Payload:                          commitmentHash,
		RandomSignatureCommitments:       randomSignatureCommitments,
		RandomSignatureBitfieldPositions: randomSignatureBitfieldPositions,
		RandomValidatorAddresses:         randomValidatorAddresses,
		RandomPublicKeyMerkleProofs:      randomPublicKeyMerkleProofs,
	}
	return msg, nil
}

// ---------------------------------------------------------------------------------------------
// 			Use following types from GSRPC's types/beefy.go once it's merged/published
// ---------------------------------------------------------------------------------------------

// Commitment is a beefy commitment
type Commitment struct {
	Payload        types.H256
	BlockNumber    types.BlockNumber
	ValidatorSetID types.U64
}

// Bytes gets the Bytes representation of a Commitment TODO: new function that needs to be added to GSRPC
func (c Commitment) Bytes() []byte {
	blockNumber := make([]byte, 4)
	binary.LittleEndian.PutUint32(blockNumber, uint32(c.BlockNumber))
	valSetID := make([]byte, 8)
	binary.LittleEndian.PutUint64(valSetID, uint64(c.ValidatorSetID))
	x := append(c.Payload[:], blockNumber...)
	return append(x, valSetID...)
}

// SignedCommitment is a beefy commitment with optional signatures from the set of validators
type SignedCommitment struct {
	Commitment Commitment
	Signatures []OptionBeefySignature
}

// BeefySignature is a beefy signature
type BeefySignature [65]byte

// OptionBeefySignature is a structure that can store a BeefySignature or a missing value
type OptionBeefySignature struct {
	option
	value BeefySignature
}

// NewOptionBeefySignature creates an OptionBeefySignature with a value
func NewOptionBeefySignature(value BeefySignature) OptionBeefySignature {
	return OptionBeefySignature{option{true}, value}
}

// NewOptionBeefySignatureEmpty creates an OptionBeefySignature without a value
func NewOptionBeefySignatureEmpty() OptionBeefySignature {
	return OptionBeefySignature{option: option{false}}
}

func (o OptionBeefySignature) Encode(encoder scale.Encoder) error {
	return encoder.EncodeOption(o.hasValue, o.value)
}

func (o *OptionBeefySignature) Decode(decoder scale.Decoder) error {
	return decoder.DecodeOption(&o.hasValue, &o.value)
}

// SetSome sets a value
func (o *OptionBeefySignature) SetSome(value BeefySignature) {
	o.hasValue = true
	o.value = value
}

// SetNone removes a value and marks it as missing
func (o *OptionBeefySignature) SetNone() {
	o.hasValue = false
	o.value = BeefySignature{}
}

// Unwrap returns a flag that indicates whether a value is present and the stored value
func (o OptionBeefySignature) Unwrap() (ok bool, value BeefySignature) {
	return o.hasValue, o.value
}

type option struct {
	hasValue bool
}

// IsNone returns true if the value is missing
func (o option) IsNone() bool {
	return !o.hasValue
}

// IsNone returns true if a value is present
func (o option) IsSome() bool {
	return o.hasValue
}
