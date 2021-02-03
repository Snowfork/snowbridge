package substrate

import (
	"fmt"

	"github.com/snowfork/go-substrate-rpc-client/v2/scale"
	"github.com/snowfork/go-substrate-rpc-client/v2/types"
)

type AuxiliaryDigestItem struct {
	IsCommitment bool
	AsCommitment Commitment
}

type Commitment struct {
	ChannelID ChannelID
	Hash      types.H256
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

type ChannelID struct {
	IsBasic        bool
	IsIncentivized bool
}

func (c *ChannelID) Decode(decoder scale.Decoder) error {
	tag, err := decoder.ReadOneByte()
	if err != nil {
		return err
	}

	switch tag {
	case 0:
		c.IsBasic = true
	case 1:
		c.IsIncentivized = true
	default:
		return fmt.Errorf("No such variant for ChannelID")
	}

	return nil
}

func (c ChannelID) Encode(encoder scale.Encoder) error {
	var err error
	switch {
	case c.IsBasic:
		err = encoder.PushByte(0)
	case c.IsIncentivized:
		err = encoder.PushByte(1)
	default:
		return fmt.Errorf("No such variant for ChannelID")
	}

	if err != nil {
		return err
	}

	return nil
}