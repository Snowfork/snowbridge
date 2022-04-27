package parachain

import (
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/contracts/basic"
	"github.com/snowfork/snowbridge/relayer/contracts/incentivized"
)

func (b BasicOutboundChannelMessageBundle) IntoInboundMessageBundle() basic.BasicInboundChannelMessageBundle {
	var messages []basic.BasicInboundChannelMessage
	for _, m := range b.Messages {
		messages = append(messages, basic.BasicInboundChannelMessage{
			Id:      m.ID,
			Target:  m.Target,
			Payload: m.Payload,
		})
	}
	return basic.BasicInboundChannelMessageBundle{
		Nonce:    b.Nonce,
		Messages: messages,
	}
}

type BasicOutboundChannelMessageBundle struct {
	Nonce    uint64
	Messages []BasicOutboundChannelMessage
}

type BasicOutboundChannelMessage struct {
	ID      uint64
	Target  [20]byte
	Payload []byte
}

func (b IncentivizedOutboundChannelMessageBundle) IntoInboundMessageBundle() incentivized.IncentivizedInboundChannelMessageBundle {
	var messages []incentivized.IncentivizedInboundChannelMessage
	for _, m := range b.Messages {
		messages = append(messages, incentivized.IncentivizedInboundChannelMessage{
			Id:      m.ID,
			Target:  m.Target,
			Fee:     m.Fee.Int,
			Payload: m.Payload,
		})
	}
	return incentivized.IncentivizedInboundChannelMessageBundle{
		Nonce:    b.Nonce,
		Messages: messages,
	}
}

type IncentivizedOutboundChannelMessageBundle struct {
	Nonce    uint64
	Messages []IncentivizedOutboundChannelMessage
}

type IncentivizedOutboundChannelMessage struct {
	ID      uint64
	Target  [20]byte
	Fee     types.U128
	Payload []byte
}
