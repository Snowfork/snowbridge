package substrate

type AccountIDType = [32]byte

type BasicChannelCommitment struct {
	Subcommitments []BasicChannelAccountSubcommitment
}

type BasicChannelAccountSubcommitment struct {
	AccountID      AccountIDType
	Messages       []BasicChannelMessage
	FlatCommitment []byte
}

type BasicChannelMessage struct {
	Target  [20]byte
	Nonce   uint64
	Payload []byte
}
