package beefy

import (
	"bytes"
	"fmt"
	"math/big"

	"github.com/ethereum/go-ethereum/common"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/contracts/beefylightclient"
	"github.com/snowfork/snowbridge/relayer/crypto/keccak"
	"github.com/snowfork/snowbridge/relayer/crypto/merkle"
)

type InitialSignatureCommitment struct {
	CommitmentHash                [32]byte
	ValidatorClaimsBitfield       []*big.Int
	ValidatorSignatureCommitment  []byte
	ValidatorPosition             *big.Int
	ValidatorPublicKey            common.Address
	ValidatorPublicKeyMerkleProof [][32]byte
}

type FinalSignatureCommitment struct {
	ID                             *big.Int
	Commitment                     beefylightclient.BeefyLightClientCommitment
	Signatures                     [][]byte
	ValidatorPositions             []*big.Int
	ValidatorPublicKeys            []common.Address
	ValidatorPublicKeyMerkleProofs [][][32]byte
	LatestMMRLeaf                  beefylightclient.BeefyLightClientBeefyMMRLeaf
	SimplifiedProof                beefylightclient.SimplifiedMMRProof
}

func (t *Task) MakeInitialSignatureCommitment(valAddrIndex int64, initialBitfield []*big.Int) (*InitialSignatureCommitment, error) {
	commitmentBytes, err := types.EncodeToBytes(t.SignedCommitment.Commitment)
	if err != nil {
		return nil, err
	}

	commitmentHash := (&keccak.Keccak256{}).Hash(commitmentBytes)

	var commitmentHash32 [32]byte
	copy(commitmentHash32[:], commitmentHash[0:32])

	proof, err := t.GenerateValidatorAddressProof(valAddrIndex)
	if err != nil {
		return nil, err
	}

	ok, beefySig := t.SignedCommitment.Signatures[valAddrIndex].Unwrap()
	if !ok {
		return nil, fmt.Errorf("signature is empty")
	}

	msg := InitialSignatureCommitment{
		CommitmentHash:                commitmentHash32,
		ValidatorClaimsBitfield:       initialBitfield,
		ValidatorSignatureCommitment:  cleanSignature(beefySig),
		ValidatorPublicKey:            t.Validators[valAddrIndex],
		ValidatorPosition:             big.NewInt(valAddrIndex),
		ValidatorPublicKeyMerkleProof: proof,
	}

	return &msg, nil
}

func cleanSignature(input types.BeefySignature) []byte {
	// Update signature format (Polkadot uses recovery IDs 0 or 1, Eth uses 27 or 28, so we need to add 27)
	// Split signature into r, s, v and add 27 to v
	rs := input[:64]
	v := input[64]
	return append(rs, byte(uint8(v)+27))
}

func (t *Task) GenerateValidatorAddressProof(valAddrIndex int64) ([][32]byte, error) {
	// Hash validator addresses for leaf input data
	beefyTreeData := make([][]byte, len(t.Validators))
	for i, valAddr := range t.Validators {
		beefyTreeData[i] = valAddr.Bytes()
	}

	_, _, proof, err := merkle.GenerateMerkleProof(beefyTreeData, valAddrIndex)
	if err != nil {
		return nil, err
	}

	return proof, nil
}

func (t *Task) MakeFinalSignatureCommitment(bitfield string) (*FinalSignatureCommitment, error) {
	validationDataID := big.NewInt(t.ValidationID)

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

		ok, beefySig := t.SignedCommitment.Signatures[validatorPosition.Int64()].Unwrap()
		if !ok {
			return nil, fmt.Errorf("signature is empty")
		}

		signatures = append(signatures, cleanSignature(beefySig))
		pubKey := t.Validators[validatorPosition.Int64()]
		validatorPublicKeys = append(validatorPublicKeys, pubKey)

		merkleProof, err := t.GenerateValidatorAddressProof(validatorPosition.Int64())
		if err != nil {
			return nil, err
		}

		validatorPublicKeyMerkleProofs = append(validatorPublicKeyMerkleProofs, merkleProof)
	}

	payload, err := buildPayload(t.SignedCommitment.Commitment.Payload)
	if err != nil {
		return nil, err
	}

	commitment := beefylightclient.BeefyLightClientCommitment{
		Payload:        *payload,
		BlockNumber:    t.SignedCommitment.Commitment.BlockNumber,
		ValidatorSetId: t.SignedCommitment.Commitment.ValidatorSetID,
	}

	latestMMRLeaf := beefylightclient.BeefyLightClientBeefyMMRLeaf{
		Version:              uint8(t.Proof.Leaf.Version),
		ParentNumber:         uint32(t.Proof.Leaf.ParentNumberAndHash.ParentNumber),
		ParentHash:           t.Proof.Leaf.ParentNumberAndHash.Hash,
		ParachainHeadsRoot:   t.Proof.Leaf.ParachainHeads,
		NextAuthoritySetId:   uint64(t.Proof.Leaf.BeefyNextAuthoritySet.ID),
		NextAuthoritySetLen:  uint32(t.Proof.Leaf.BeefyNextAuthoritySet.Len),
		NextAuthoritySetRoot: t.Proof.Leaf.BeefyNextAuthoritySet.Root,
	}

	merkleProofItems := [][32]byte{}
	for _, mmrProofItem := range t.Proof.MerkleProofItems {
		merkleProofItems = append(merkleProofItems, mmrProofItem)
	}

	msg := FinalSignatureCommitment{
		ID:                             validationDataID,
		Commitment:                     commitment,
		Signatures:                     signatures,
		ValidatorPositions:             validatorPositions,
		ValidatorPublicKeys:            validatorPublicKeys,
		ValidatorPublicKeyMerkleProofs: validatorPublicKeyMerkleProofs,
		LatestMMRLeaf:                  latestMMRLeaf,

		SimplifiedProof: beefylightclient.SimplifiedMMRProof{
			MerkleProofItems:         merkleProofItems,
			MerkleProofOrderBitField: t.Proof.MerkleProofOrder,
		},
	}
	return &msg, nil
}

// Builds a payload which is partially SCALE-encoded. This is more efficient for the light client to verify
// as it does not have to implement a fully fledged SCALE-encoder.
func buildPayload(items []types.PayloadItem) (*beefylightclient.BeefyLightClientPayload, error) {
	index := -1

	for i, payloadItem := range items {
		if payloadItem.ID == [2]byte{0x6d, 0x68} {
			index = i
		}
	}

	if index < 0 {
		return nil, fmt.Errorf("Did not find mmr root hash in commitment")
	}

	mmrRootHash := [32]byte{}

	if len(items[index].Data) != 32 {
		return nil, fmt.Errorf("Mmr root hash is invalid")
	}

	if copy(mmrRootHash[:], items[index].Data) != 32 {
		return nil, fmt.Errorf("Mmr root hash is invalid")
	}

	payloadBytes, err := types.EncodeToBytes(items)
	if err != nil {
		return nil, err
	}

	slices := bytes.Split(payloadBytes, mmrRootHash[:])
	if len(slices) != 2 {
		// Its theoretically possible that the payload items may contain mmrRootHash more than once, causing an invalid split
		return nil, fmt.Errorf("Expected 2 slices")
	}

	return &beefylightclient.BeefyLightClientPayload{
		MmrRootHash: mmrRootHash,
		Prefix:      slices[0],
		Suffix:      slices[1],
	}, nil
}
