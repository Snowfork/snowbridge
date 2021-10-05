package parachain

import (
	"github.com/snowfork/go-substrate-rpc-client/v3/types"
	"github.com/snowfork/snowbridge/relayer/contracts/basic"
	"github.com/snowfork/snowbridge/relayer/contracts/incentivized"
)

type BasicOutboundChannelMessages []BasicOutboundChannelMessage

func (ms BasicOutboundChannelMessages) IsRelayed(nonceToFind uint64) bool {

	ms[len(ms)-1]

	for _, message := range ms {
		if message.Nonce <= nonceToFind {
			return true
		}
	}
	return false
}

func (ms BasicOutboundChannelMessages) IntoInboundMessages() []basic.BasicInboundChannelMessage {
	var output []basic.BasicInboundChannelMessage
	for _, m := range ms {
		output = append(output, m.IntoInboundMessage())
	}
	return output
}

type BasicOutboundChannelMessage struct {
	Target  [20]byte
	Nonce   uint64
	Payload []byte
}

func (m *BasicOutboundChannelMessage) IntoInboundMessage() basic.BasicInboundChannelMessage {
	return basic.BasicInboundChannelMessage{
		Target:  m.Target,
		Nonce:   m.Nonce,
		Payload: m.Payload,
	}
}

type IncentivizedOutboundChannelMessages []IncentivizedOutboundChannelMessage

type IncentivizedOutboundChannelMessage struct {
	Target  [20]byte
	Nonce   uint64
	Fee     types.U256
	Payload []byte
}

func (m *IncentivizedOutboundChannelMessage) IntoInboundMessage() *incentivized.IncentivizedInboundChannelMessage {
	return &incentivized.IncentivizedInboundChannelMessage{
		Target:  m.Target,
		Nonce:   m.Nonce,
		Fee:     m.Fee.Int,
		Payload: m.Payload,
	}
}
