package substrate

import (
	"errors"
	"fmt"

	client "github.com/snowfork/go-substrate-rpc-client/v2/client"
	"github.com/snowfork/go-substrate-rpc-client/v2/types"
	chainTypes "github.com/snowfork/polkadot-ethereum/relayer/substrate"
)

func GetMerkleProofs(client client.Client, key []byte) ([]chainTypes.MerkleProof, error) {
	var res string
	err := client.Call(&res, "basicChannel_getMerkleProofs", fmt.Sprintf("%#x", key))
	if err != nil {
		return nil, err
	}
	if len(res) == 0 {
		return nil, errors.New("no data received")
	}

	var proofs []chainTypes.MerkleProof
	if err := types.DecodeFromHexString(res, &proofs); err != nil {
		return nil, err
	}
	return proofs, nil
}
