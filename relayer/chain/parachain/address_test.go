package parachain

import (
	assert "github.com/stretchr/testify/require"
	"testing"
)

func TestSS58Prefix(t *testing.T) {
	address := "0x8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48"

	ss58Address, err := SS58Encode(address, 1)
	assert.NoError(t, err)
	assert.Equal(t, "A1k3praCLftTgBTb6aVavh3UNKwXN599Fqov17MkEy6bwCU", ss58Address)
}
