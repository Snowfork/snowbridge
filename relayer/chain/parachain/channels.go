package parachain

import "github.com/snowfork/go-substrate-rpc-client/v3/types"

type OutboundChannelMessage struct {
	Target  [20]byte
	Nonce   uint64
	Payload []byte
}

type BasicOutboundChannelMessage = OutboundChannelMessage

type IncentivizedOutboundChannelMessage struct {
	BasicOutboundChannelMessage
	Fee types.U256
}
