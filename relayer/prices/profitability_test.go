package prices

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestMaxDelaySecsDefaultAndOverride(t *testing.T) {
	// Unset (empty) falls back to the default.
	t.Setenv(maxDelaySecsEnvKey, "")
	assert.Equal(t, uint64(defaultMaxDelaySecs), MaxDelaySecs())

	// Valid override is parsed.
	t.Setenv(maxDelaySecsEnvKey, "7200")
	assert.Equal(t, uint64(7200), MaxDelaySecs())

	// Invalid value falls back to the default.
	t.Setenv(maxDelaySecsEnvKey, "not-a-number")
	assert.Equal(t, uint64(defaultMaxDelaySecs), MaxDelaySecs())
}

func TestMaxDelayBlocksConversion(t *testing.T) {
	t.Setenv(maxDelaySecsEnvKey, "14400") // 4 hours

	assert.Equal(t, uint64(1200), MaxDelayBlocks(12)) // ethereum: 4h at 12s
	assert.Equal(t, uint64(2400), MaxDelayBlocks(6))  // parachain: 4h at 6s
	assert.Equal(t, uint64(0), MaxDelayBlocks(0))     // zero block time guarded, no divide-by-zero

	// A budget shorter than one block rounds down to 0 (valve effectively disabled).
	t.Setenv(maxDelaySecsEnvKey, "5")
	assert.Equal(t, uint64(0), MaxDelayBlocks(12))
}
