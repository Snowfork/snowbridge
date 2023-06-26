package beefy

import (
	"fmt"
	"math/big"

	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/contracts"
	"github.com/snowfork/snowbridge/relayer/crypto/keccak"
	"github.com/snowfork/snowbridge/relayer/crypto/merkle"
)

type InitialRequestParams struct {
	CommitmentHash [32]byte
	Bitfield       []*big.Int
	Proof          contracts.BeefyClientValidatorProof
}

type FinalRequestParams struct {
	Commitment     contracts.BeefyClientCommitment
	Bitfield       []*big.Int
	Proofs         []contracts.BeefyClientValidatorProof
	Leaf           contracts.BeefyClientMMRLeaf
	LeafProof      [][32]byte
	LeafProofOrder *big.Int
}

func (r *Request) CommitmentHash() (*[32]byte, error) {
	commitmentBytes, err := types.EncodeToBytes(r.SignedCommitment.Commitment)
	if err != nil {
		return nil, err
	}

	commitmentHash := (&keccak.Keccak256{}).Hash(commitmentBytes)

	var commitmentHash32 [32]byte
	copy(commitmentHash32[:], commitmentHash[0:32])

	return &commitmentHash32, nil
}

// Generate RequestParams which contains merkle proof by validator's index
// together with the signature which will be verified in BeefyClient contract later
func (r *Request) MakeSubmitInitialParams(valAddrIndex int64, initialBitfield []*big.Int) (*InitialRequestParams, error) {
	commitmentHash, err := r.CommitmentHash()
	if err != nil {
		return nil, fmt.Errorf("generate commitment hash: %w", err)
	}

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

	v, _r, s := cleanSignature(validatorSignature)

	msg := InitialRequestParams{
		CommitmentHash: *commitmentHash,
		Bitfield:       initialBitfield,
		Proof: contracts.BeefyClientValidatorProof{
			V:       v,
			R:       _r,
			S:       s,
			Index:   big.NewInt(valAddrIndex),
			Account: validatorAddress,
			Proof:   proof,
		},
	}

	return &msg, nil
}

func cleanSignature(input types.BeefySignature) (uint8, [32]byte, [32]byte) {
	// Update signature format (Polkadot uses recovery IDs 0 or 1, Eth uses 27 or 28, so we need to add 27)
	// Split signature into r, s, v and add 27 to v
	r := *(*[32]byte)(input[:32])
	s := *(*[32]byte)(input[32:64])
	v := byte(uint8(input[64]) + 27)
	return v, r, s
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

func (r *Request) MakeSubmitFinalParams(validatorIndices []uint64, initialBitfield []*big.Int) (*FinalRequestParams, error) {
	validatorProofs := []contracts.BeefyClientValidatorProof{}

	for _, validatorIndex := range validatorIndices {
		ok, beefySig := r.SignedCommitment.Signatures[validatorIndex].Unwrap()
		if !ok {
			return nil, fmt.Errorf("signature is empty")
		}

		v, _r, s := cleanSignature(beefySig)
		account, err := r.Validators[validatorIndex].IntoEthereumAddress()
		if err != nil {
			return nil, fmt.Errorf("convert to ethereum address: %w", err)
		}

		merkleProof, err := r.generateValidatorAddressProof(int64(validatorIndex))
		if err != nil {
			return nil, err
		}

		validatorProofs = append(validatorProofs, contracts.BeefyClientValidatorProof{
			V:       v,
			R:       _r,
			S:       s,
			Index:   new(big.Int).SetUint64(validatorIndex),
			Account: account,
			Proof:   merkleProof,
		})
	}

	payload, err := buildPayload(r.SignedCommitment.Commitment.Payload)
	if err != nil {
		return nil, err
	}

	commitment := contracts.BeefyClientCommitment{
		Payload:        *payload,
		BlockNumber:    r.SignedCommitment.Commitment.BlockNumber,
		ValidatorSetID: r.SignedCommitment.Commitment.ValidatorSetID,
	}

	inputLeaf := contracts.BeefyClientMMRLeaf{
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

	msg := FinalRequestParams{
		Commitment:     commitment,
		Bitfield:       initialBitfield,
		Proofs:         validatorProofs,
		Leaf:           inputLeaf,
		LeafProof:      merkleProofItems,
		LeafProofOrder: new(big.Int).SetUint64(r.Proof.MerkleProofOrder),
	}

	return &msg, nil
}

// Builds a payload which is partially SCALE-encoded. This is more efficient for the light client to verify
// as it does not have to implement a fully fledged SCALE-encoder.
func buildPayload(items []types.PayloadItem) (*contracts.BeefyClientPayload, error) {
	index := -1

	for i, payloadItem := range items {
		// MMR Root ID as "mh"
		// https://github.com/paritytech/substrate/blob/cbd8f1b56fd8ab9af0d9317432cc735264c89d70/primitives/beefy/src/payload.rs#L33
		if payloadItem.ID == [2]byte{0x6d, 0x68} {
			index = i
		}
	}

	// Contains one entry so index should be 0
	// https://github.com/paritytech/substrate/blob/cbd8f1b56fd8ab9af0d9317432cc735264c89d70/primitives/beefy/src/payload.rs#L48
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

	return &contracts.BeefyClientPayload{
		MmrRootHash: mmrRootHash,
	}, nil
}
