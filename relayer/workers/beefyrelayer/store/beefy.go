package store

import (
	"fmt"
	"math/big"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/crypto"
	"github.com/snowfork/polkadot-ethereum/relayer/contracts/lightclientbridge"
	merkletree "github.com/wealdtech/go-merkletree"
	"golang.org/x/crypto/blake2b"
)

type NewSignatureCommitmentMessage struct {
	CommitmentHash                [32]byte
	ValidatorClaimsBitfield       []*big.Int
	ValidatorSignatureCommitment  []byte
	ValidatorPosition             *big.Int
	ValidatorPublicKey            common.Address
	ValidatorPublicKeyMerkleProof [][32]byte
}

type CompleteSignatureCommitmentMessage struct {
	ID                             *big.Int
	CommitmentHash                 [32]byte
	Commitment                     lightclientbridge.LightClientBridgeCommitment
	Signatures                     [][]byte
	ValidatorPositions             []*big.Int
	ValidatorPublicKeys            []common.Address
	ValidatorPublicKeyMerkleProofs [][][32]byte
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
	// Split signature into r, s, v and add 27 to v
	sigValPolkadot := b.SignedCommitment.Signatures[valAddrIndex].Value
	sigValrs := sigValPolkadot[:64]
	sigValv := sigValPolkadot[64]
	sigValvAdded := byte(uint8(sigValv) + 27)
	sigValEthereum := append(sigValrs, sigValvAdded)

	commitmentHash := blake2b.Sum256(b.SignedCommitment.Commitment.Bytes())

	validatorClaimsBitfield := []*big.Int{big.NewInt(123)} // TODO: add bitfield stuff properly

	msg := NewSignatureCommitmentMessage{
		CommitmentHash:                commitmentHash,
		ValidatorClaimsBitfield:       validatorClaimsBitfield,
		ValidatorSignatureCommitment:  sigValEthereum,
		ValidatorPublicKey:            b.ValidatorAddresses[valAddrIndex],
		ValidatorPosition:             big.NewInt(int64(valAddrIndex)),
		ValidatorPublicKeyMerkleProof: sig0ProofContents,
	}

	return msg, nil
}

// Keccak256 is the Keccak256 hashing method
type Keccak256 struct{}

// New creates a new Keccak256 hashing method
func New() *Keccak256 {
	return &Keccak256{}
}

// Hash generates a Keccak256 hash from a byte array
func (h *Keccak256) Hash(data []byte) []byte {
	hash := crypto.Keccak256(data)
	return hash[:]
}

func (b *BeefyJustification) GenerateMerkleProofOffchain(valAddrIndex int) ([][32]byte, error) {
	// Hash validator addresses for leaf input data
	beefyTreeData := make([][]byte, len(b.ValidatorAddresses))
	for i, valAddr := range b.ValidatorAddresses {
		beefyTreeData[i] = valAddr.Bytes()
	}

	// Create the tree
	beefyMerkleTree, err := merkletree.NewUsing(beefyTreeData, &Keccak256{}, nil)
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
	verified, err := merkletree.VerifyProofUsing(beefyTreeData[valAddrIndex], sigProof, root, &Keccak256{}, nil)
	if err != nil {
		return [][32]byte{}, err
	}
	if !verified {
		return [][32]byte{}, fmt.Errorf("failed to verify proof")
	}

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

	//TODO: Use info.RandomSeed.Big() to generate validatorPositions
	validatorPositions := []*big.Int{}

	//TODO: Populate signatures, validatorPublicKeys, and based on validatorPositions
	signatures := [][]byte{}
	validatorPublicKeys := b.ValidatorAddresses
	validatorPublicKeyMerkleProofs := [][][32]byte{}

	commitment := lightclientbridge.LightClientBridgeCommitment{
		Payload:        b.SignedCommitment.Commitment.Payload,
		BlockNumber:    uint64(b.SignedCommitment.Commitment.BlockNumber),
		ValidatorSetId: uint32(b.SignedCommitment.Commitment.ValidatorSetID),
	}

	msg := CompleteSignatureCommitmentMessage{
		ID:                             validationDataID,
		CommitmentHash:                 commitmentHash,
		Commitment:                     commitment,
		Signatures:                     signatures,
		ValidatorPositions:             validatorPositions,
		ValidatorPublicKeys:            validatorPublicKeys,
		ValidatorPublicKeyMerkleProofs: validatorPublicKeyMerkleProofs,
	}
	return msg, nil
}
