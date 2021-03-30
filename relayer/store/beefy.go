package store

import (
	"encoding/hex"
	"encoding/json"
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

type Status int

const (
	CommitmentWitnessed            Status = iota // 0
	InitialVerificationTxSent      Status = iota // 1
	InitialVerificationTxConfirmed Status = iota // 2
	ReadyToComplete                Status = iota // 3
	CompleteVerificationTxSent     Status = iota // 4
)

type Beefy struct {
	ValidatorAddresses         []common.Address
	SignedCommitment           SignedCommitment
	Status                     Status
	InitialVerificationTxHash  common.Hash
	CompleteOnBlock            uint64
	RandomSeed                 common.Hash
	CompleteVerificationTxHash common.Hash
}

func NewBeefy(validatorAddresses []common.Address, signedCommitment SignedCommitment,
	status Status, initialVerificationTxHash common.Hash, completeOnBlock uint64,
	randomSeed, completeVerificationTxHash common.Hash) Beefy {
	return Beefy{
		ValidatorAddresses:         validatorAddresses,
		SignedCommitment:           signedCommitment,
		Status:                     status,
		InitialVerificationTxHash:  initialVerificationTxHash,
		CompleteOnBlock:            completeOnBlock,
		RandomSeed:                 randomSeed,
		CompleteVerificationTxHash: completeVerificationTxHash,
	}
}

func (b *Beefy) ToItem() (BeefyItem, error) {
	validatorAddressesBytes, err := json.Marshal(b.ValidatorAddresses)
	if err != nil {
		return BeefyItem{}, err
	}

	signedCommitmentBytes, err := json.Marshal(b.SignedCommitment)
	if err != nil {
		return BeefyItem{}, err
	}

	beefyItem := NewBeefyItem(validatorAddressesBytes, signedCommitmentBytes, b.Status,
		b.InitialVerificationTxHash, b.CompleteOnBlock, b.RandomSeed, b.CompleteVerificationTxHash)

	return beefyItem, nil
}

func (b *Beefy) BuildNewSignatureCommitmentMessage(valAddrIndex int) (NewSignatureCommitmentMessage, error) {
	sig0ProofContents, err := b.GenerateMmrProofOffchain(valAddrIndex)
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

func (b *Beefy) GenerateMmrProofOffchain(valAddrIndex int) ([][32]byte, error) {
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

func (b *Beefy) BuildCompleteSignatureCommitmentMessage(randIndex int64) (CompleteSignatureCommitmentMessage, error) {
	commitmentHash := blake2b.Sum256(b.SignedCommitment.Commitment.Bytes())

	validationDataID := big.NewInt(int64(b.SignedCommitment.Commitment.ValidatorSetID))

	//TODO: Generate randomSignatureBitfieldPositions properly
	randomSignatureBitfieldPositions := []*big.Int{}

	//TODO: Populate randomSignatureCommitments, randomValidatorAddresses, and based on randomSignatureBitfieldPositions
	randomSignatureCommitments := [][]byte{}
	randomValidatorAddresses := b.ValidatorAddresses
	randomPublicKeyMerkleProofs := [][][32]byte{}

	// TODO: select addresses and proofs using randIndex

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
