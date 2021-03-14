// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package parachain

import (
	"crypto/sha256"
	"fmt"
	"math/big"

	"github.com/snowfork/go-substrate-rpc-client/v2/scale"
	"github.com/snowfork/go-substrate-rpc-client/v2/types"

	"github.com/ethereum/go-ethereum/common"
	merkletree "github.com/wealdtech/go-merkletree"
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
	ValidatorClaimsBitfield       *big.Int
	ValidatorSignatureCommitment  []byte
	ValidatorPublicKey            common.Address
	ValidatorPublicKeyMerkleProof [][32]byte
}

type CompleteSignatureCommitmentMessage struct {
	ID                               *big.Int
	Payload                          [32]byte
	RandomSignatureCommitments       [][]byte
	RandomSignatureBitfieldPositions []uint8
	RandomValidatorAddresses         []common.Address
	RandomPublicKeyMerkleProofs      [][][32]byte
}

func (b BeefyCommitmentInfo) BuildNewSignatureCommitmentMessage() (NewSignatureCommitmentMessage, error) {
	// Hash validator addresses for leaf input data
	var beefyTreeData [][]byte
	for _, valAddr := range b.ValidatorAddresses {
		h := sha256.New()
		if _, err := h.Write(valAddr.Bytes()); err != nil {
			return NewSignatureCommitmentMessage{}, err
		}
		hashedData := h.Sum(nil)
		beefyTreeData = append(beefyTreeData, hashedData)
	}

	// Create the tree
	beefyMerkleTree, err := merkletree.New(beefyTreeData)
	if err != nil {
		return NewSignatureCommitmentMessage{}, err
	}

	// Generate a proof
	sig0ProofData := beefyTreeData[0]
	sig0Proof, err := beefyMerkleTree.GenerateProof(sig0ProofData)
	if err != nil {
		return NewSignatureCommitmentMessage{}, err
	}

	// TODO: Verify the proof
	// root := beefyMerkleTree.Root()
	// verified, err := beefyMerkleTree.VerifyProof(sig0ProofData, sig0Proof, root)
	// if err != nil {
	// 	return NewSignatureCommitmentMessage{}, err
	// }
	// if !verified {
	// 	return NewSignatureCommitmentMessage{}, fmt.Errorf("failed to verify proof")
	// }

	sig0ProofContents := make([][32]byte, len(sig0Proof.Hashes))
	for i, hash := range sig0Proof.Hashes {
		var hash32Byte [32]byte
		copy(hash32Byte[:], hash)
		sig0ProofContents[i] = hash32Byte
	}

	hashedCommitment, err := b.GetHashedCommitment()
	if err != nil {
		return NewSignatureCommitmentMessage{}, err
	}

	// // Update signature format (Polkadot uses recovery IDs 0 or 1, Eth uses 27 or 28, so we need to add 27)
	// recIdIncrement := big.NewInt(27)
	ok, sig0 := b.SignedCommitment.Signatures[0].Unwrap()
	if !ok {
		return NewSignatureCommitmentMessage{}, err
	}

	// TODO: increment recovery ID properly
	valSigCommitment := sig0[:]
	// sig0HexStr := hexutil.Encode(sig0[:])             // bytes -> 0x string
	// recoveryId0, err := hexutil.DecodeBig(sig0HexStr) // 0x string -> big.Int
	// if err != nil {
	// 	li.log.Info("err:", err)
	// }
	// incrementedRecoveryId0 := recoveryId0.Add(recoveryId0, recIdIncrement)
	// newRecoveryId0 := hexutil.EncodeBig(incrementedRecoveryId0) // big.Int -> 0x string
	// newRecoveryId0Bytes, err := hexutil.Decode(newRecoveryId0)  // 0x string -> []byte
	// if err != nil {
	// 	li.log.Info("err:", err)
	// }
	// valSigCommitment := append(sig0[:], newRecoveryId0Bytes...)

	sig0ValAddr := b.ValidatorAddresses[0]

	msg := NewSignatureCommitmentMessage{
		Payload:                       hashedCommitment,
		ValidatorClaimsBitfield:       big.NewInt(123), // TODO: add bitfield stuff properly
		ValidatorSignatureCommitment:  valSigCommitment,
		ValidatorPublicKey:            sig0ValAddr,
		ValidatorPublicKeyMerkleProof: sig0ProofContents,
	}

	return msg, nil
}

func (b BeefyCommitmentInfo) BuildCompleteSignatureCommitmentMessage() (CompleteSignatureCommitmentMessage, error) {
	hashedCommitment, err := b.GetHashedCommitment()
	if err != nil {
		return CompleteSignatureCommitmentMessage{}, err
	}

	validationDataID := big.NewInt(0)

	//TODO: Generate randomSignatureBitfieldPositions properly
	randomSignatureBitfieldPositions := []uint8{}

	//TODO: Populate randomSignatureCommitments, randomValidatorAddresses, and based on randomSignatureBitfieldPositions
	randomSignatureCommitments := [][]byte{}
	randomValidatorAddresses := b.ValidatorAddresses
	randomPublicKeyMerkleProofs := [][][32]byte{}

	msg := CompleteSignatureCommitmentMessage{
		ID:                               validationDataID,
		Payload:                          hashedCommitment,
		RandomSignatureCommitments:       randomSignatureCommitments,
		RandomSignatureBitfieldPositions: randomSignatureBitfieldPositions,
		RandomValidatorAddresses:         randomValidatorAddresses,
		RandomPublicKeyMerkleProofs:      randomPublicKeyMerkleProofs,
	}
	return msg, nil
}

func (b BeefyCommitmentInfo) GetHashedCommitment() ([32]byte, error) {
	var hashedCommitment32Byte [32]byte
	h := sha256.New()
	commitmentBytes := []byte(fmt.Sprintf("%v", b.SignedCommitment.Commitment))
	if _, err := h.Write(commitmentBytes); err != nil {
		return hashedCommitment32Byte, err
	}
	hashedCommitment := h.Sum(nil)
	copy(hashedCommitment32Byte[:], hashedCommitment)
	return hashedCommitment32Byte, nil
}

// TODO: use these types from GSRPC's types/beefy.go once it's merged/published

// Commitment is a beefy commitment
type Commitment struct {
	Payload        types.H256
	BlockNumber    types.BlockNumber
	ValidatorSetID types.U64
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
