package substrate

import "github.com/snowfork/go-substrate-rpc-client/v2/types"

type BasicChannelCommitment struct {
	Subcommitments []BasicChannelAccountSubcommitment
}

type BasicChannelAccountSubcommitment struct {
	AccountID      types.AccountID
	Messages       []BasicChannelMessage
	FlatCommitment []byte
}

type BasicChannelMessage struct {
	Target  [20]byte
	Nonce   uint64
	Payload []byte
}
