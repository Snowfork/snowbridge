package parachainV3

import (
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
)

func ExtractCommitmentFromDigest(digest types.Digest) (*types.H256, error) {
	return extractCommitmentFromDigestWithPrefix(digest, 1)
}

func ExtractCommitmentFromDigestV1(digest types.Digest) (*types.H256, error) {
	return extractCommitmentFromDigestWithPrefix(digest, 0)
}

func extractCommitmentFromDigestWithPrefix(digest types.Digest, prefix byte) (*types.H256, error) {
	for _, digestItem := range digest {
		if digestItem.IsOther {
			digestItemRawBytes := digestItem.AsOther
			if len(digestItemRawBytes) == 0 {
				continue
			}
			if digestItemRawBytes[0] == prefix {
				var commitment types.H256
				err := types.DecodeFromBytes(digestItemRawBytes[1:], &commitment)
				if err != nil {
					return nil, err
				}
				return &commitment, nil
			}
		}
	}
	return nil, nil
}
