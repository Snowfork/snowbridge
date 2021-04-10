package store

import (
	"encoding/hex"
	"fmt"
	"math/big"

	"github.com/ethereum/go-ethereum/common"
	merkletree "github.com/wealdtech/go-merkletree"
	"golang.org/x/crypto/blake2b"
	"golang.org/x/crypto/sha3"
)

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

type BeefyJustification struct {
	ValidatorAddresses []common.Address
	SignedCommitment   SignedCommitment
}

func NewBeefyJustification(validatorAddresses []common.Address, signedCommitment SignedCommitment) BeefyJustification {
	return BeefyJustification{
		ValidatorAddresses: validatorAddresses,
		SignedCommitment:   signedCommitment,
	}
}

func (b *BeefyJustification) BuildNewSignatureCommitmentMessage(valAddrIndex int) (NewSignatureCommitmentMessage, error) {
	sig0ProofContents, err := b.GenerateMerkleProofOffchain(valAddrIndex)
	if err != nil {
		return NewSignatureCommitmentMessage{}, err
	}

	// Update signature format (Polkadot uses recovery IDs 0 or 1, Eth uses 27 or 28, so we need to add 27)
	sigVal := b.SignedCommitment.Signatures[valAddrIndex].Value
	sig0Commitment := sigVal[:] // TODO: increment recovery ID properly

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

func (b *BeefyJustification) GenerateMerkleProofOffchain(valAddrIndex int) ([][32]byte, error) {
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

	// Generate Merkle Proof for validator at index valAddrIndex
	sigProof, err := beefyMerkleTree.GenerateProof(beefyTreeData[valAddrIndex])
	if err != nil {
		return [][32]byte{}, err
	}

	// Verify the proof
	root := beefyMerkleTree.Root()
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

func (b *BeefyJustification) BuildCompleteSignatureCommitmentMessage() (CompleteSignatureCommitmentMessage, error) {
	commitmentHash := blake2b.Sum256(b.SignedCommitment.Commitment.Bytes())

	validationDataID := big.NewInt(int64(b.SignedCommitment.Commitment.ValidatorSetID))

	//TODO: Generate randomSignatureBitfieldPositions from info.RandomSeed.Big()
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
