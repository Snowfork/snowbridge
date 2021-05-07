package parachain

import (
	"bytes"

	"github.com/snowfork/go-substrate-rpc-client/v2/scale"
	"github.com/snowfork/go-substrate-rpc-client/v2/types"
	chainTypes "github.com/snowfork/polkadot-ethereum/relayer/substrate"
)

var IndexingPrefix = []byte("commitment")

func MakeStorageKey(channelID chainTypes.ChannelID, hash types.H256) ([]byte, error) {
	var buffer = bytes.Buffer{}
	encoder := scale.NewEncoder(&buffer)

	err := encoder.Encode(IndexingPrefix)
	if err != nil {
		return nil, err
	}

	err = encoder.Encode(channelID)
	if err != nil {
		return nil, err
	}

	err = encoder.Encode(hash)
	if err != nil {
		return nil, err
	}

	return buffer.Bytes(), nil
}
