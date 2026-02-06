package beefy

import (
	"math/big"
	"testing"

	"github.com/ethereum/go-ethereum/common"
	"github.com/snowfork/snowbridge/relayer/contracts"
	"github.com/stretchr/testify/assert"
)

func TestToBeefyClientCommitment(t *testing.T) {
	input := &contracts.IBeefyClientCommitment{
		BlockNumber:    12345,
		ValidatorSetID: 67890,
		Payload: []contracts.IBeefyClientPayloadItem{
			{PayloadID: [2]byte{'m', 'h'}, Data: []byte{0x01, 0x02, 0x03}},
		},
	}

	result := ToBeefyClientCommitment(input)

	assert.Equal(t, input.BlockNumber, result.BlockNumber)
	assert.Equal(t, input.ValidatorSetID, result.ValidatorSetID)
	assert.Equal(t, len(input.Payload), len(result.Payload))
	assert.Equal(t, input.Payload[0].PayloadID, result.Payload[0].PayloadID)
	assert.Equal(t, input.Payload[0].Data, result.Payload[0].Data)
}

func TestToBeefyClientCommitment_EmptyPayload(t *testing.T) {
	input := &contracts.IBeefyClientCommitment{
		BlockNumber:    100,
		ValidatorSetID: 1,
		Payload:        []contracts.IBeefyClientPayloadItem{},
	}

	result := ToBeefyClientCommitment(input)
	assert.Equal(t, 0, len(result.Payload))
}

func TestToBeefyClientValidatorProof(t *testing.T) {
	input := &contracts.IBeefyClientValidatorProof{
		V:       27,
		R:       [32]byte{0x01},
		S:       [32]byte{0x02},
		Index:   big.NewInt(5),
		Account: common.HexToAddress("0x1234567890123456789012345678901234567890"),
		Proof:   [][32]byte{{0x03}, {0x04}},
	}

	result := ToBeefyClientValidatorProof(input)

	assert.Equal(t, input.V, result.V)
	assert.Equal(t, input.R, result.R)
	assert.Equal(t, input.S, result.S)
	assert.Equal(t, input.Index, result.Index)
	assert.Equal(t, input.Account, result.Account)
	assert.Equal(t, input.Proof, result.Proof)
}

func TestToBeefyClientValidatorProofs(t *testing.T) {
	inputs := []contracts.IBeefyClientValidatorProof{
		{V: 27, R: [32]byte{0x01}, S: [32]byte{0x02}, Index: big.NewInt(0), Account: common.HexToAddress("0x1111111111111111111111111111111111111111"), Proof: [][32]byte{{0x03}}},
		{V: 28, R: [32]byte{0x11}, S: [32]byte{0x12}, Index: big.NewInt(1), Account: common.HexToAddress("0x2222222222222222222222222222222222222222"), Proof: [][32]byte{{0x13}}},
	}

	results := ToBeefyClientValidatorProofs(inputs)

	assert.Equal(t, len(inputs), len(results))
	for i, result := range results {
		assert.Equal(t, inputs[i].V, result.V)
		assert.Equal(t, inputs[i].Index, result.Index)
	}
}

func TestToBeefyClientValidatorProofs_Empty(t *testing.T) {
	results := ToBeefyClientValidatorProofs([]contracts.IBeefyClientValidatorProof{})
	assert.Equal(t, 0, len(results))
}

func TestToBeefyClientMMRLeaf(t *testing.T) {
	input := &contracts.IBeefyClientMMRLeaf{
		Version:              1,
		ParentNumber:         100,
		ParentHash:           [32]byte{0x01},
		NextAuthoritySetID:   2,
		NextAuthoritySetLen:  50,
		NextAuthoritySetRoot: [32]byte{0x02},
		ParachainHeadsRoot:   [32]byte{0x03},
	}

	result := ToBeefyClientMMRLeaf(input)

	assert.Equal(t, input.Version, result.Version)
	assert.Equal(t, input.ParentNumber, result.ParentNumber)
	assert.Equal(t, input.ParentHash, result.ParentHash)
	assert.Equal(t, input.NextAuthoritySetID, result.NextAuthoritySetID)
	assert.Equal(t, input.NextAuthoritySetLen, result.NextAuthoritySetLen)
	assert.Equal(t, input.NextAuthoritySetRoot, result.NextAuthoritySetRoot)
	assert.Equal(t, input.ParachainHeadsRoot, result.ParachainHeadsRoot)
}
