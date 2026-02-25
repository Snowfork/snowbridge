package beefy

import (
	"errors"
	"fmt"
	"testing"

	"github.com/stretchr/testify/assert"
)

// mockJsonError implements the JsonError interface for testing
type mockJsonError struct {
	message   string
	code      int
	errorData interface{}
}

func (e *mockJsonError) Error() string          { return e.message }
func (e *mockJsonError) ErrorCode() int         { return e.code }
func (e *mockJsonError) ErrorData() interface{} { return e.errorData }

// wrappedError wraps another error for testing error unwrapping
type wrappedError struct {
	msg   string
	inner error
}

func (e *wrappedError) Error() string { return e.msg }
func (e *wrappedError) Unwrap() error { return e.inner }

func TestIsExpectedCompetitionError_NilError(t *testing.T) {
	assert.False(t, isExpectedCompetitionError(nil))
}

func TestIsExpectedCompetitionError_TicketAlreadyOwned(t *testing.T) {
	err := errors.New("execution reverted: 0x60bbe44e")
	assert.True(t, isExpectedCompetitionError(err))
}

func TestIsExpectedCompetitionError_StaleCommitment(t *testing.T) {
	err := errors.New("execution reverted: 0x3d618e50")
	assert.True(t, isExpectedCompetitionError(err))
}

func TestIsExpectedCompetitionError_NotTicketOwner(t *testing.T) {
	err := errors.New("execution reverted: 0xe18d39ad")
	assert.True(t, isExpectedCompetitionError(err))
}

func TestIsExpectedCompetitionError_InvalidCommitment(t *testing.T) {
	err := errors.New("execution reverted: 0xc06789fa")
	assert.True(t, isExpectedCompetitionError(err))
}

func TestIsExpectedCompetitionError_UnrelatedError(t *testing.T) {
	assert.False(t, isExpectedCompetitionError(errors.New("some random error")))
}

func TestIsExpectedCompetitionError_JsonError(t *testing.T) {
	tests := []struct {
		name      string
		errorData string
		expected  bool
	}{
		{"TicketAlreadyOwned", "0x60bbe44e", true},
		{"StaleCommitment", "0x3d618e50", true},
		{"InvalidCommitment", "0xc06789fa", true},
		{"NotTicketOwner", "0xe18d39ad", true},
		{"Unrelated", "0xdeadbeef", false},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			err := &mockJsonError{message: "execution reverted", code: 3, errorData: tc.errorData}
			assert.Equal(t, tc.expected, isExpectedCompetitionError(err))
		})
	}
}

func TestIsExpectedCompetitionError_WrappedJsonError(t *testing.T) {
	innerErr := &mockJsonError{message: "execution reverted", code: 3, errorData: "0x60bbe44e"}
	err := &wrappedError{msg: "outer error", inner: innerErr}
	assert.True(t, isExpectedCompetitionError(err))
}

func TestIsExpectedCompetitionError_DeeplyWrappedJsonError(t *testing.T) {
	innerErr := &mockJsonError{message: "execution reverted", code: 3, errorData: "0x3d618e50"}
	err := &wrappedError{
		msg:   "outer error",
		inner: &wrappedError{msg: "middle error", inner: innerErr},
	}
	assert.True(t, isExpectedCompetitionError(err))
}

func TestIsExpectedCompetitionError_ErrorWithPartialMatch(t *testing.T) {
	err := fmt.Errorf("call failed with error data: 0x60bbe44e and more text")
	assert.True(t, isExpectedCompetitionError(err))
}

func TestErrorSelectorConstants(t *testing.T) {
	// Verify selectors match expected keccak256 hashes
	assert.Equal(t, [4]byte{0x60, 0xbb, 0xe4, 0x4e}, ErrTicketAlreadyOwned)
	assert.Equal(t, [4]byte{0x3d, 0x61, 0x8e, 0x50}, ErrStaleCommitment)
	assert.Equal(t, [4]byte{0xe1, 0x8d, 0x39, 0xad}, ErrNotTicketOwner)
	assert.Equal(t, [4]byte{0xc0, 0x67, 0x89, 0xfa}, ErrInvalidCommitment)
}

func TestIsExpectedSelector_ByteSlice(t *testing.T) {
	assert.True(t, isExpectedSelector([]byte{0x60, 0xbb, 0xe4, 0x4e}))
	assert.False(t, isExpectedSelector([]byte{0xde, 0xad, 0xbe, 0xef}))
	assert.False(t, isExpectedSelector([]byte{0x60, 0xbb})) // too short
	assert.False(t, isExpectedSelector(nil))
}
