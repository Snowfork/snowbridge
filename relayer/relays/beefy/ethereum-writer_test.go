package beefy

import (
	"errors"
	"fmt"
	"math/big"
	"testing"

	"github.com/ethereum/go-ethereum/common"
	"github.com/snowfork/snowbridge/relayer/contracts"
	"github.com/stretchr/testify/assert"
)

// mockJsonError implements the JsonError interface for testing
type mockJsonError struct {
	message   string
	code      int
	errorData interface{}
}

func (e *mockJsonError) Error() string {
	return e.message
}

func (e *mockJsonError) ErrorCode() int {
	return e.code
}

func (e *mockJsonError) ErrorData() interface{} {
	return e.errorData
}

// wrappedError wraps another error for testing error unwrapping
type wrappedError struct {
	msg   string
	inner error
}

func (e *wrappedError) Error() string {
	return e.msg
}

func (e *wrappedError) Unwrap() error {
	return e.inner
}

func TestIsExpectedCompetitionError_NilError(t *testing.T) {
	result := isExpectedCompetitionError(nil)
	assert.False(t, result, "nil error should return false")
}

func TestIsExpectedCompetitionError_TicketAlreadyOwned(t *testing.T) {
	// Test error string containing the hex code
	err := errors.New("execution reverted: 0x60bbe44e")
	result := isExpectedCompetitionError(err)
	assert.True(t, result, "TicketAlreadyOwned error should be recognized")
}

func TestIsExpectedCompetitionError_StaleCommitment(t *testing.T) {
	err := errors.New("execution reverted: 0x3d618e50")
	result := isExpectedCompetitionError(err)
	assert.True(t, result, "StaleCommitment error should be recognized")
}

func TestIsExpectedCompetitionError_NotTicketOwner(t *testing.T) {
	err := errors.New("execution reverted: 0xe18d39ad")
	result := isExpectedCompetitionError(err)
	assert.True(t, result, "NotTicketOwner error should be recognized")
}

func TestIsExpectedCompetitionError_InvalidCommitment(t *testing.T) {
	err := errors.New("execution reverted: 0xc06789fa")
	result := isExpectedCompetitionError(err)
	assert.True(t, result, "InvalidCommitment error should be recognized")
}

func TestIsExpectedCompetitionError_UnrelatedError(t *testing.T) {
	err := errors.New("some random error")
	result := isExpectedCompetitionError(err)
	assert.False(t, result, "Unrelated error should return false")
}

func TestIsExpectedCompetitionError_JsonErrorWithTicketAlreadyOwned(t *testing.T) {
	err := &mockJsonError{
		message:   "execution reverted",
		code:      3,
		errorData: "0x60bbe44e",
	}
	result := isExpectedCompetitionError(err)
	assert.True(t, result, "JsonError with TicketAlreadyOwned data should be recognized")
}

func TestIsExpectedCompetitionError_JsonErrorWithStaleCommitment(t *testing.T) {
	err := &mockJsonError{
		message:   "execution reverted",
		code:      3,
		errorData: "0x3d618e50",
	}
	result := isExpectedCompetitionError(err)
	assert.True(t, result, "JsonError with StaleCommitment data should be recognized")
}

func TestIsExpectedCompetitionError_JsonErrorWithInvalidCommitment(t *testing.T) {
	err := &mockJsonError{
		message:   "execution reverted",
		code:      3,
		errorData: "0xc06789fa",
	}
	result := isExpectedCompetitionError(err)
	assert.True(t, result, "JsonError with InvalidCommitment data should be recognized")
}

func TestIsExpectedCompetitionError_JsonErrorWithUnrelatedData(t *testing.T) {
	err := &mockJsonError{
		message:   "execution reverted",
		code:      3,
		errorData: "0xdeadbeef",
	}
	result := isExpectedCompetitionError(err)
	assert.False(t, result, "JsonError with unrelated data should return false")
}

func TestIsExpectedCompetitionError_WrappedJsonError(t *testing.T) {
	innerErr := &mockJsonError{
		message:   "execution reverted",
		code:      3,
		errorData: "0x60bbe44e",
	}
	err := &wrappedError{
		msg:   "outer error",
		inner: innerErr,
	}
	result := isExpectedCompetitionError(err)
	assert.True(t, result, "Wrapped JsonError should be unwrapped and recognized")
}

func TestIsExpectedCompetitionError_DeeplyWrappedJsonError(t *testing.T) {
	innerErr := &mockJsonError{
		message:   "execution reverted",
		code:      3,
		errorData: "0x3d618e50",
	}
	err := &wrappedError{
		msg: "outer error",
		inner: &wrappedError{
			msg:   "middle error",
			inner: innerErr,
		},
	}
	result := isExpectedCompetitionError(err)
	assert.True(t, result, "Deeply wrapped JsonError should be unwrapped and recognized")
}

func TestIsExpectedCompetitionError_WrappedUnrelatedError(t *testing.T) {
	innerErr := errors.New("some inner error")
	err := &wrappedError{
		msg:   "outer error",
		inner: innerErr,
	}
	result := isExpectedCompetitionError(err)
	assert.False(t, result, "Wrapped unrelated error should return false")
}

func TestIsExpectedCompetitionError_ErrorWithPartialMatch(t *testing.T) {
	// Error message contains the hex code as part of a larger message
	err := fmt.Errorf("call failed with error data: 0x60bbe44e and more text")
	result := isExpectedCompetitionError(err)
	assert.True(t, result, "Error containing the hex code should be recognized")
}

// Test type conversion functions

func TestToBeefyClientCommitment(t *testing.T) {
	input := &contracts.IBeefyClientCommitment{
		BlockNumber:    12345,
		ValidatorSetID: 67890,
		Payload: []contracts.IBeefyClientPayloadItem{
			{
				PayloadID: [2]byte{'m', 'h'},
				Data:      []byte{0x01, 0x02, 0x03},
			},
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

	assert.Equal(t, input.BlockNumber, result.BlockNumber)
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
		{
			V:       27,
			R:       [32]byte{0x01},
			S:       [32]byte{0x02},
			Index:   big.NewInt(0),
			Account: common.HexToAddress("0x1111111111111111111111111111111111111111"),
			Proof:   [][32]byte{{0x03}},
		},
		{
			V:       28,
			R:       [32]byte{0x11},
			S:       [32]byte{0x12},
			Index:   big.NewInt(1),
			Account: common.HexToAddress("0x2222222222222222222222222222222222222222"),
			Proof:   [][32]byte{{0x13}},
		},
	}

	results := ToBeefyClientValidatorProofs(inputs)

	assert.Equal(t, len(inputs), len(results))
	for i, result := range results {
		assert.Equal(t, inputs[i].V, result.V)
		assert.Equal(t, inputs[i].R, result.R)
		assert.Equal(t, inputs[i].S, result.S)
		assert.Equal(t, inputs[i].Index, result.Index)
		assert.Equal(t, inputs[i].Account, result.Account)
	}
}

func TestToBeefyClientValidatorProofs_Empty(t *testing.T) {
	inputs := []contracts.IBeefyClientValidatorProof{}
	results := ToBeefyClientValidatorProofs(inputs)
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

// Test error selector constants
func TestErrorSelectorConstants(t *testing.T) {
	// Verify that error selectors are correctly defined
	assert.Equal(t, "0x60bbe44e", ErrTicketAlreadyOwned)
	assert.Equal(t, "0x3d618e50", ErrStaleCommitment)
	assert.Equal(t, "0xe18d39ad", ErrNotTicketOwner)
	assert.Equal(t, "0xc06789fa", ErrInvalidCommitment)
}
