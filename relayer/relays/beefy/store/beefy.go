package store

import (
	"math/big"

	"github.com/ethereum/go-ethereum/common"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/contracts/beefylightclient"
	"github.com/snowfork/snowbridge/relayer/crypto/keccak"
	"github.com/snowfork/snowbridge/relayer/crypto/merkle"
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
	Commitment                     beefylightclient.BeefyLightClientCommitment
	Signatures                     [][]byte
	ValidatorPositions             []*big.Int
	ValidatorPublicKeys            []common.Address
	ValidatorPublicKeyMerkleProofs [][][32]byte
	LatestMMRLeaf                  beefylightclient.BeefyLightClientBeefyMMRLeaf
	SimplifiedProof                beefylightclient.SimplifiedMMRProof
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

func (b *BeefyJustification) BuildNewSignatureCommitmentMessage(valAddrIndex int64, initialBitfield []*big.Int) (NewSignatureCommitmentMessage, error) {
	commitmentHash := (&keccak.Keccak256{}).Hash(b.SignedCommitment.Commitment.Bytes())

	var commitmentHash32 [32]byte
	copy(commitmentHash32[:], commitmentHash[0:32])

	sig0ProofContents, err := b.GenerateValidatorAddressProof(valAddrIndex)
	if err != nil {
		return NewSignatureCommitmentMessage{}, err
	}

	sigValEthereum := BeefySigToEthSig(b.SignedCommitment.Signatures[valAddrIndex].Value)

	msg := NewSignatureCommitmentMessage{
		CommitmentHash:                commitmentHash32,
		ValidatorClaimsBitfield:       initialBitfield,
		ValidatorSignatureCommitment:  sigValEthereum,
		ValidatorPublicKey:            b.ValidatorAddresses[valAddrIndex],
		ValidatorPosition:             big.NewInt(valAddrIndex),
		ValidatorPublicKeyMerkleProof: sig0ProofContents,
	}

	return msg, nil
}

func BeefySigToEthSig(beefySig BeefySignature) []byte {
	// Update signature format (Polkadot uses recovery IDs 0 or 1, Eth uses 27 or 28, so we need to add 27)
	// Split signature into r, s, v and add 27 to v
	sigValrs := beefySig[:64]
	sigValv := beefySig[64]
	sigValvAdded := byte(uint8(sigValv) + 27)
	sigValEthereum := append(sigValrs, sigValvAdded)

	return sigValEthereum
}

func (b *BeefyJustification) GenerateValidatorAddressProof(valAddrIndex int64) ([][32]byte, error) {
	// Hash validator addresses for leaf input data
	beefyTreeData := make([][]byte, len(b.ValidatorAddresses))
	for i, valAddr := range b.ValidatorAddresses {
		beefyTreeData[i] = valAddr.Bytes()
	}

	_, _, proof, err := merkle.GenerateMerkleProof(beefyTreeData, valAddrIndex)
	if err != nil {
		return nil, err
	}

	return proof, nil
}

func (b *BeefyJustification) BuildCompleteSignatureCommitmentMessage(info BeefyRelayInfo, bitfield string) (CompleteSignatureCommitmentMessage, error) {
	validationDataID := big.NewInt(int64(info.ContractID))

	validatorPositions := []*big.Int{}

	// bitfield is right to left order, so loop backwards
	for i := len(bitfield) - 1; i >= 0; i-- {
		bit := bitfield[i : i+1]
		if bit == "1" {
			position := len(bitfield) - 1 - i // positions start from 0 and increase to len(bitfield) - 1
			validatorPositions = append(validatorPositions, big.NewInt(int64(position)))
		}
	}

	signatures := [][]byte{}
	validatorPublicKeys := []common.Address{}
	validatorPublicKeyMerkleProofs := [][][32]byte{}
	for _, validatorPosition := range validatorPositions {
		beefySig := b.SignedCommitment.Signatures[validatorPosition.Int64()].Value
		ethSig := BeefySigToEthSig(beefySig)
		signatures = append(signatures, ethSig)

		pubKey := b.ValidatorAddresses[validatorPosition.Int64()]
		validatorPublicKeys = append(validatorPublicKeys, pubKey)

		merkleProof, err := b.GenerateValidatorAddressProof(validatorPosition.Int64())
		if err != nil {
			return CompleteSignatureCommitmentMessage{}, err
		}

		validatorPublicKeyMerkleProofs = append(validatorPublicKeyMerkleProofs, merkleProof)
	}

	commitment := beefylightclient.BeefyLightClientCommitment{
		Payload:        b.SignedCommitment.Commitment.Payload,
		BlockNumber:    uint64(b.SignedCommitment.Commitment.BlockNumber),
		ValidatorSetId: uint32(b.SignedCommitment.Commitment.ValidatorSetID),
	}

	var latestMMRProof merkle.SimplifiedMMRProof
	err := types.DecodeFromBytes(info.SerializedLatestMMRProof, &latestMMRProof)
	if err != nil {
		return CompleteSignatureCommitmentMessage{}, err
	}

	latestMMRLeaf := beefylightclient.BeefyLightClientBeefyMMRLeaf{
		Version:              uint8(latestMMRProof.Leaf.Version),
		ParentNumber:         uint32(latestMMRProof.Leaf.ParentNumberAndHash.ParentNumber),
		ParentHash:           latestMMRProof.Leaf.ParentNumberAndHash.Hash,
		ParachainHeadsRoot:   latestMMRProof.Leaf.ParachainHeads,
		NextAuthoritySetId:   uint64(latestMMRProof.Leaf.BeefyNextAuthoritySet.ID),
		NextAuthoritySetLen:  uint32(latestMMRProof.Leaf.BeefyNextAuthoritySet.Len),
		NextAuthoritySetRoot: latestMMRProof.Leaf.BeefyNextAuthoritySet.Root,
	}

	merkleProofItems := [][32]byte{}
	for _, mmrProofItem := range latestMMRProof.MerkleProofItems {
		merkleProofItems = append(merkleProofItems, mmrProofItem)
	}

	msg := CompleteSignatureCommitmentMessage{
		ID:                             validationDataID,
		Commitment:                     commitment,
		Signatures:                     signatures,
		ValidatorPositions:             validatorPositions,
		ValidatorPublicKeys:            validatorPublicKeys,
		ValidatorPublicKeyMerkleProofs: validatorPublicKeyMerkleProofs,
		LatestMMRLeaf:                  latestMMRLeaf,

		SimplifiedProof: beefylightclient.SimplifiedMMRProof{
			MerkleProofItems:         merkleProofItems,
			MerkleProofOrderBitField: latestMMRProof.MerkleProofOrder,
		},
	}
	return msg, nil
}
