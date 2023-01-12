package parachain

import (
	"math/big"

	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/contracts/basic"
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
		Account:  b.Account,
		Nonce:    (*big.Int)(&b.Nonce).Uint64(),
		Messages: messages,
	}
}

type BasicOutboundChannelMessageBundle struct {
	Account  types.AccountID
	Nonce    types.UCompact
	Messages []BasicOutboundChannelMessage
}

type BasicOutboundChannelMessage struct {
	ID      types.UCompact
	Target  [20]byte
	Payload []byte
}
