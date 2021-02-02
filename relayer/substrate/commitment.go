package substrate

type CommitmentMessage struct {
	Target  [20]byte
	Nonce   uint64
	Payload []byte
}