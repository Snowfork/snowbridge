package parachainV3

import (
	"testing"

	"github.com/snowfork/snowbridge/relayer/relays/error_tracking"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestScannerIsIgnoredNonce(t *testing.T) {
	s := &Scanner{
		ignoreNonces: []uint64{7, 42, 264},
	}

	assert.True(t, s.isIgnoredNonce(264))
	assert.True(t, s.isIgnoredNonce(42))
	assert.True(t, s.isIgnoredNonce(7))
	assert.False(t, s.isIgnoredNonce(1))

	assert.False(t, (*Scanner)(nil).isIgnoredNonce(7))
	assert.False(t, (&Scanner{}).isIgnoredNonce(7))
}

func TestScannerIsDeadNonce(t *testing.T) {
	// A nil store (e.g. in-memory disabled or unset) must never report dead.
	assert.False(t, (*Scanner)(nil).isDeadNonce(7))
	assert.False(t, (&Scanner{}).isDeadNonce(7))

	store, err := error_tracking.NewDeadNonceStore("")
	require.NoError(t, err)
	s := &Scanner{deadNonces: store}

	assert.False(t, s.isDeadNonce(99))

	// markDeadNonce records the nonce so subsequent scans skip it.
	s.markDeadNonce(99, "operator override")
	assert.True(t, s.isDeadNonce(99))

	// markDeadNonce is a no-op when the store is unset.
	(&Scanner{}).markDeadNonce(1, "noop")
}

func TestScannerDispatchedCache(t *testing.T) {
	assert.False(t, (*Scanner)(nil).isCachedDispatched(1))
	s := &Scanner{}
	assert.False(t, s.isCachedDispatched(727))

	s.markDispatched(727)
	assert.True(t, s.isCachedDispatched(727))
	assert.False(t, s.isCachedDispatched(728))

	// Positive cache is sticky (v2_isDispatched bits are immutable once set).
	s.markDispatched(727)
	assert.True(t, s.isCachedDispatched(727))
}
