package parachain

import (
	"testing"

	chainTypes "github.com/snowfork/polkadot-ethereum/relayer/substrate"
	"github.com/stretchr/testify/assert"
)

func TestMakeStorageKey(t *testing.T) {

	commitmentHash := [32]byte{
		7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
		7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
		7, 7, 7, 7, 7, 7, 7, 7,
	}

	channelID := chainTypes.ChannelID{
		IsBasic: true,
	}

	key, err := MakeStorageKey(channelID, commitmentHash)
	if err != nil {
		panic(err)
	}

	assert.Equal(t,
		[]byte{
			40, 99, 111, 109, 109, 105, 116, 109, 101, 110, 116, 0,
			7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
			7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
			7, 7, 7, 7, 7, 7, 7, 7,
		},
		key,
	)

}
