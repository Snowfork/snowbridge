package beefy

import (
	"fmt"
	"testing"

	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/stretchr/testify/assert"
)


func makeCommitment() (*types.Commitment, error) {
	data := types.MustHexDecodeString("3ac49cd24778522203e8bf40a4712ea3f07c3803bbd638cb53ebb3564ec13e8c")

	item1 := types.PayloadItem{
		ID:   [2]byte{'a', 'b'},
		Data: []byte{0, 1, 2},
	}

	item2 := types.PayloadItem{
		ID:   [2]byte{'m', 'h'},
		Data: data,
	}

	commitment := types.Commitment{
		Payload:        []types.PayloadItem{item1, item2},
		BlockNumber:    5,
		ValidatorSetID: 7,
	}

	return &commitment, nil
}

func TestCommitment_Split(t *testing.T) {
	c, err := makeCommitment()
	assert.NoError(t, err)

	payload, err := buildPayload(c.Payload)
	assert.NoError(t, err)

	fmt.Printf("mmrroothash: %s\n", types.HexEncodeToString(payload.MmrRootHash[:]))
	fmt.Printf("prefix: %s\n", types.HexEncodeToString(payload.Prefix))
	fmt.Printf("suffix: %s\n", types.HexEncodeToString(payload.Suffix))

	commitmentBytes, _ := types.EncodeToHexString(c)
	fmt.Println(commitmentBytes)
}
