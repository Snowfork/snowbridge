package beefy

import (
	"bytes"
	"fmt"
	"math/big"

	"github.com/ethereum/go-ethereum/common"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/contracts/beefyclient"
	"github.com/snowfork/snowbridge/relayer/crypto/keccak"
	"github.com/snowfork/snowbridge/relayer/crypto/merkle"
)

type InitialRequestParams struct {
	CommitmentHash          [32]byte
	ValidatorSetID          uint64
	ValidatorClaimsBitfield []*big.Int
	Proof                   beefyclient.BeefyClientValidatorProof
}

type FinalRequestParams struct {
	ID             *big.Int
	Commitment     beefyclient.BeefyClientCommitment
	Proof          beefyclient.BeefyClientValidatorMultiProof
	Leaf           beefyclient.BeefyClientMMRLeaf
	LeafProof      [][32]byte
	LeafProofOrder *big.Int
}

func (r *Request) MakeSubmitInitialParams(valAddrIndex int64, initialBitfield []*big.Int) (*InitialRequestParams, error) {
	commitmentBytes, err := types.EncodeToBytes(r.SignedCommitment.Commitment)
	if err != nil {
		return nil, err
	}

	commitmentHash := (&keccak.Keccak256{}).Hash(commitmentBytes)

	var commitmentHash32 [32]byte
	copy(commitmentHash32[:], commitmentHash[0:32])

	proof, err := r.generateValidatorAddressProof(valAddrIndex)
	if err != nil {
		return nil, fmt.Errorf("generate validator proof: %w", err)
	}

	ok, validatorSignature := r.SignedCommitment.Signatures[valAddrIndex].Unwrap()
	if !ok {
		return nil, fmt.Errorf("signature is empty")
	}

	validatorAddress, err := r.Validators[valAddrIndex].IntoEthereumAddress()
	if err != nil {
		return nil, fmt.Errorf("convert to ethereum address: %w", err)
	}

	msg := InitialRequestParams{
		CommitmentHash:          commitmentHash32,
		ValidatorSetID:          r.SignedCommitment.Commitment.ValidatorSetID,
		ValidatorClaimsBitfield: initialBitfield,
		Proof: beefyclient.BeefyClientValidatorProof{
			Signature:   cleanSignature(validatorSignature),
			Index:       big.NewInt(valAddrIndex),
			Addr:        validatorAddress,
			MerkleProof: proof,
		},
	}

	return &msg, nil
}

func cleanSignature(input types.BeefySignature) beefyclient.BeefyClientValidatorSignature {
	// Update signature format (Polkadot uses recovery IDs 0 or 1, Eth uses 27 or 28, so we need to add 27)
	// Split signature into r, s, v and add 27 to v
	r := *(*[32]byte)(input[:32])
	s := *(*[32]byte)(input[32:64])
	v := byte(uint8(input[64]) + 27)

	return beefyclient.BeefyClientValidatorSignature{
		V: v,
		R: r,
		S: s,
	}
}

func (r *Request) generateValidatorAddressProof(validatorIndex int64) ([][32]byte, error) {
	leaves := make([][]byte, len(r.Validators))
	for i, rawAddress := range r.Validators {
		address, err := rawAddress.IntoEthereumAddress()
		if err != nil {
			return nil, fmt.Errorf("convert to ethereum address: %w", err)
		}
		leaves[i] = address.Bytes()
	}

	_, _, proof, err := merkle.GenerateMerkleProof(leaves, validatorIndex)
	if err != nil {
		return nil, err
	}

	return proof, nil
}

func (r *Request) MakeSubmitFinalParams(validationID int64, validatorIndices []uint64) (*FinalRequestParams, error) {
	validationDataID := big.NewInt(validationID)

	signatures := []beefyclient.BeefyClientValidatorSignature{}
	validatorAddresses := []common.Address{}
	validatorAddressProofs := [][][32]byte{}
	for _, validatorIndex := range validatorIndices {

		ok, beefySig := r.SignedCommitment.Signatures[validatorIndex].Unwrap()
		if !ok {
			return nil, fmt.Errorf("signature is empty")
		}

		signatures = append(signatures, cleanSignature(beefySig))
		pubKey := r.Validators[validatorIndex]

		address, err := pubKey.IntoEthereumAddress()
		if err != nil {
			return nil, fmt.Errorf("convert to ethereum address: %w", err)
		}

		validatorAddresses = append(validatorAddresses, address)

		merkleProof, err := r.generateValidatorAddressProof(int64(validatorIndex))
		if err != nil {
			return nil, err
		}

		validatorAddressProofs = append(validatorAddressProofs, merkleProof)
	}

	payload, err := buildPayload(r.SignedCommitment.Commitment.Payload)
	if err != nil {
		return nil, err
	}

	commitment := beefyclient.BeefyClientCommitment{
		Payload:        *payload,
		BlockNumber:    r.SignedCommitment.Commitment.BlockNumber,
		ValidatorSetID: r.SignedCommitment.Commitment.ValidatorSetID,
	}

	inputLeaf := beefyclient.BeefyClientMMRLeaf{
		Version:              uint8(r.Proof.Leaf.Version),
		ParentNumber:         uint32(r.Proof.Leaf.ParentNumberAndHash.ParentNumber),
		ParentHash:           r.Proof.Leaf.ParentNumberAndHash.Hash,
		ParachainHeadsRoot:   r.Proof.Leaf.ParachainHeads,
		NextAuthoritySetID:   uint64(r.Proof.Leaf.BeefyNextAuthoritySet.ID),
		NextAuthoritySetLen:  uint32(r.Proof.Leaf.BeefyNextAuthoritySet.Len),
		NextAuthoritySetRoot: r.Proof.Leaf.BeefyNextAuthoritySet.Root,
	}

	merkleProofItems := [][32]byte{}
	for _, mmrProofItem := range r.Proof.MerkleProofItems {
		merkleProofItems = append(merkleProofItems, mmrProofItem)
	}

	validatorIndicesBigInt := []*big.Int{}
	for _, index := range validatorIndices {
		validatorIndicesBigInt = append(validatorIndicesBigInt, new(big.Int).SetUint64(index))
	}

	msg := FinalRequestParams{
		ID:         validationDataID,
		Commitment: commitment,
		Proof: beefyclient.BeefyClientValidatorMultiProof{
			Signatures:   signatures,
			Indices:      validatorIndicesBigInt,
			Addrs:        validatorAddresses,
			MerkleProofs: validatorAddressProofs,
		},
		Leaf:           inputLeaf,
		LeafProof:      merkleProofItems,
		LeafProofOrder: new(big.Int).SetUint64(r.Proof.MerkleProofOrder),
	}

	return &msg, nil
}

// Builds a payload which is partially SCALE-encoded. This is more efficient for the light client to verify
// as it does not have to implement a fully fledged SCALE-encoder.
func buildPayload(items []types.PayloadItem) (*beefyclient.BeefyClientPayload, error) {
	index := -1

	for i, payloadItem := range items {
		if payloadItem.ID == [2]byte{0x6d, 0x68} {
			index = i
		}
	}

	if index < 0 {
		return nil, fmt.Errorf("did not find mmr root hash in commitment")
	}

	mmrRootHash := [32]byte{}

	if len(items[index].Data) != 32 {
		return nil, fmt.Errorf("mmr root hash is invalid")
	}

	if copy(mmrRootHash[:], items[index].Data) != 32 {
		return nil, fmt.Errorf("mmr root hash is invalid")
	}

	payloadBytes, err := types.EncodeToBytes(items)
	if err != nil {
		return nil, err
	}

	slices := bytes.Split(payloadBytes, mmrRootHash[:])
	if len(slices) != 2 {
		// Its theoretically possible that the payload items may contain mmrRootHash more than once, causing an invalid split
		return nil, fmt.Errorf("expected 2 slices")
	}

	return &beefyclient.BeefyClientPayload{
		MmrRootHash: mmrRootHash,
		Prefix:      slices[0],
		Suffix:      slices[1],
	}, nil
}
