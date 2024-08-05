package json

import (
	"math/big"
)

type InitialRequestJsonParams struct {
	Commitment BeefyClientCommitment
	Bitfield   []*big.Int
	Proof      BeefyClientValidatorProof
}

type BeefyClientCommitment struct {
	BlockNumber    uint32
	ValidatorSetID uint64
	Payload        []BeefyClientPayloadItem
}

type BeefyClientPayloadItem struct {
	PayloadID [2]byte
	Data      string
}

type BeefyClientValidatorProof struct {
	V       uint8
	R       string
	S       string
	Index   uint64
	Account string
	Proof   []string
}
