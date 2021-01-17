package digest

import (
	"bytes"
	"fmt"

	"github.com/snowfork/go-substrate-rpc-client/v2/scale"
	"github.com/snowfork/go-substrate-rpc-client/v2/types"
)

type AuxiliaryDigestItem struct {
	IsCommitmentHash bool
	AsCommitmentHash types.H256
}

func (a AuxiliaryDigestItem) Decode(decoder scale.Decoder) error {
	tag, err := decoder.ReadOneByte()
	if err != nil {
		return err
	}

	switch tag {
	case 0:
		a.IsCommitmentHash = true
		err = decoder.Decode(&a.AsCommitmentHash)
	default:
		return fmt.Errorf("No such variant for DigestItem")
	}

	if err != nil {
		return err
	}

	return nil
}

func DecodeFromBytes(data []byte) (AuxiliaryDigestItem, error) {
	var digestItem AuxiliaryDigestItem

	decoder := scale.NewDecoder(bytes.NewReader(data))
	err := decoder.Decode(&digestItem)
	if err != nil {
		return AuxiliaryDigestItem{}, nil
	}

	return digestItem, nil
}

func GetAuxiliaryDigestItem(digest types.Digest) (*AuxiliaryDigestItem, error) {
	for _, digestItem := range digest {
		if digestItem.IsOther {
			auxDigestItem, err := DecodeFromBytes(digestItem.AsOther)
			if err != nil {
				return nil, err
			}

			return &auxDigestItem, nil
		}
	}
	return nil, nil
}
