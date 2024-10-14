package parachain

import (
	"fmt"
	gethCommon "github.com/ethereum/go-ethereum/common"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	assert "github.com/stretchr/testify/require"
	"testing"
)

func TestGetDestination(t *testing.T) {
	s := "00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000002e00a736aa00000000000087d1f7fdfee7f651fabc8bfcb6e086c278b77a7d00e40b54020000000000000000000000000000000000000000000000000000000000"

	value := types.U64(1115511)

	hex, err := types.Hex(value)
	assert.NoError(t, err)

	fmt.Println(hex)
	data := gethCommon.Hex2Bytes(s)

	destination, err := GetDestination(data)
	assert.NoError(t, err)

	fmt.Println(destination)
}
