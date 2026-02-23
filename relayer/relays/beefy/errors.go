package beefy

import (
	"strings"

	"github.com/ethereum/go-ethereum/common"
	"golang.org/x/crypto/sha3"
)

// Expected error selectors (first 4 bytes of keccak256 hash of error signature).
// These errors indicate normal competition between relayers or stale data,
// and should not cause relayer restarts.
var (
	ErrTicketAlreadyOwned    = computeSelector("TicketAlreadyOwned()")
	ErrNotTicketOwner        = computeSelector("NotTicketOwner()")
	ErrStaleCommitment       = computeSelector("StaleCommitment()")
	ErrInvalidCommitment     = computeSelector("InvalidCommitment()")
	ErrInsufficientProgress  = computeSelector("InsufficientProgress()")
)

func computeSelector(sig string) [4]byte {
	hasher := sha3.NewLegacyKeccak256()
	hasher.Write([]byte(sig))
	var sel [4]byte
	copy(sel[:], hasher.Sum(nil)[:4])
	return sel
}

// JsonError interface for extracting error data from Ethereum RPC errors.
type JsonError interface {
	Error() string
	ErrorCode() int
	ErrorData() interface{}
}

// isExpectedCompetitionError checks if an error is due to normal relayer competition
// or other expected conditions that should not cause a relayer restart.
func isExpectedCompetitionError(err error) bool {
	if err == nil {
		return false
	}

	// Try to extract error data from the error chain
	for currentErr := err; currentErr != nil; {
		if jsonErr, ok := currentErr.(JsonError); ok {
			if isExpectedSelector(jsonErr.ErrorData()) {
				return true
			}
		}
		if unwrapper, ok := currentErr.(interface{ Unwrap() error }); ok {
			currentErr = unwrapper.Unwrap()
		} else {
			break
		}
	}

	// Fallback: check error string for hex selectors
	return containsExpectedSelector(err.Error())
}

func isExpectedSelector(data interface{}) bool {
	var bytes []byte
	switch v := data.(type) {
	case string:
		bytes = common.FromHex(v)
	case []byte:
		bytes = v
	default:
		return false
	}

	if len(bytes) < 4 {
		return false
	}

	var sel [4]byte
	copy(sel[:], bytes[:4])

	return sel == ErrTicketAlreadyOwned ||
		sel == ErrNotTicketOwner ||
		sel == ErrStaleCommitment ||
		sel == ErrInvalidCommitment ||
		sel == ErrInsufficientProgress
}

func containsExpectedSelector(s string) bool {
	for _, sel := range [][4]byte{ErrTicketAlreadyOwned, ErrNotTicketOwner, ErrStaleCommitment, ErrInvalidCommitment, ErrInsufficientProgress} {
		hex := common.Bytes2Hex(sel[:])
		if strings.Contains(s, "0x"+hex) || strings.Contains(s, hex) {
			return true
		}
	}
	return false
}
