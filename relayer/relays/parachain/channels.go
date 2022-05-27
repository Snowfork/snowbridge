package parachain

import (
	"math/big"

	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/contracts/basic"
	"github.com/snowfork/snowbridge/relayer/contracts/incentivized"
)

func (b BasicOutboundChannelMessageBundle) IntoInboundMessageBundle() basic.BasicInboundChannelMessageBundle {
	var messages []basic.BasicInboundChannelMessage
	for _, m := range b.Messages {
		messages = append(messages, basic.BasicInboundChannelMessage{
			Id:      (*big.Int)(&m.ID).Uint64(),
			Target:  m.Target,
			Payload: m.Payload,
		})
	}
	return basic.BasicInboundChannelMessageBundle{
		SourceChannelID: b.SourceChannelID,
		Nonce:    (*big.Int)(&b.Nonce).Uint64(),
		Messages: messages,
	}
}

type BasicOutboundChannelMessageBundle struct {
	SourceChannelID uint8
	Nonce    types.UCompact
	Messages []BasicOutboundChannelMessage
}

type BasicOutboundChannelMessage struct {
	ID      types.UCompact
	Target  [20]byte
	Payload []byte
}

func (b IncentivizedOutboundChannelMessageBundle) IntoInboundMessageBundle() incentivized.IncentivizedInboundChannelMessageBundle {
	var messages []incentivized.IncentivizedInboundChannelMessage
	for _, m := range b.Messages {
		messages = append(messages, incentivized.IncentivizedInboundChannelMessage{
			Id:      (*big.Int)(&m.ID).Uint64(),
			Target:  m.Target,
			Payload: m.Payload,
		})
	}
	return incentivized.IncentivizedInboundChannelMessageBundle{
		SourceChannelID: b.SourceChannelID,
		Nonce:    (*big.Int)(&b.Nonce).Uint64(),
		Fee:      (*big.Int)(&b.Fee),
		Messages: messages,
	}
}

type IncentivizedOutboundChannelMessageBundle struct {
	SourceChannelID uint8
	Nonce    types.UCompact
	Fee      types.UCompact
	Messages []IncentivizedOutboundChannelMessage
}

type IncentivizedOutboundChannelMessage struct {
	ID      types.UCompact
	Target  [20]byte
	Payload []byte
}
