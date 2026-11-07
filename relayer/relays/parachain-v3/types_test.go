package parachainV3

import (
	"testing"

	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/stretchr/testify/assert"
)

func TestIsBlockedV2Origin(t *testing.T) {
	var origin types.H256
	err := types.DecodeFromHexString(blockedV2OriginHex, &origin)
	assert.NoError(t, err)
	assert.True(t, isBlockedV2Origin(origin))

	other := types.H256{1}
	assert.False(t, isBlockedV2Origin(other))
}
