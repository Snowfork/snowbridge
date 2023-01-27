package parachain

import (
	"fmt"

	"github.com/snowfork/go-substrate-rpc-client/v4/scale"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
)

type AuxiliaryDigestItem struct {
	IsCommitment bool
	AsCommitment AuxiliaryDigestItemCommitment
}

type AuxiliaryDigestItemCommitment struct {
	Hash types.H256
}

func (a *AuxiliaryDigestItem) Decode(decoder scale.Decoder) error {
	tag, err := decoder.ReadOneByte()
	if err != nil {
		return err
	}

	switch tag {
	case 0:
		a.IsCommitment = true
		err = decoder.Decode(&a.AsCommitment)
	default:
		return fmt.Errorf("No such variant for DigestItem")
	}

	if err != nil {
		return err
	}

	return nil
}

func ExtractAuxiliaryDigestItems(digest types.Digest) ([]AuxiliaryDigestItem, error) {
	var auxDigestItems []AuxiliaryDigestItem
	for _, digestItem := range digest {
		if digestItem.IsOther {
			var auxDigestItem AuxiliaryDigestItem
			err := types.DecodeFromBytes(digestItem.AsOther, &auxDigestItem)
			if err != nil {
				return nil, err
			}
			auxDigestItems = append(auxDigestItems, auxDigestItem)
		}
	}
	return auxDigestItems, nil
}
