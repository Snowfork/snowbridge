package store

import (
	"encoding/binary"

	"github.com/snowfork/go-substrate-rpc-client/v2/scale"
	"github.com/snowfork/go-substrate-rpc-client/v2/types"
)

// ---------------------------------------------------------------------------------------------
// 			Use following types from GSRPC's types/beefy.go once it's merged/published
// ---------------------------------------------------------------------------------------------

// Commitment is a beefy commitment
type Commitment struct {
	Payload        types.H256
	BlockNumber    types.U32
	ValidatorSetID types.U64
}

// Bytes gets the Bytes representation of a Commitment TODO: new function that needs to be added to GSRPC
func (c Commitment) Bytes() []byte {
	blockNumber := make([]byte, 4)
	binary.LittleEndian.PutUint32(blockNumber, uint32(c.BlockNumber))
	valSetID := make([]byte, 8)
	binary.LittleEndian.PutUint64(valSetID, uint64(c.ValidatorSetID))
	x := append(c.Payload[:], blockNumber...)
	return append(x, valSetID...)
}

// SignedCommitment is a beefy commitment with optional signatures from the set of validators
type SignedCommitment struct {
	Commitment Commitment
	Signatures []OptionBeefySignature
}

// BeefySignature is a beefy signature
type BeefySignature [65]byte

// OptionBeefySignature is a structure that can store a BeefySignature or a missing value
type OptionBeefySignature struct {
	Option
	Value BeefySignature
}

// NewOptionBeefySignature creates an OptionBeefySignature with a value
func NewOptionBeefySignature(value BeefySignature) OptionBeefySignature {
	return OptionBeefySignature{Option{true}, value}
}

// NewOptionBeefySignatureEmpty creates an OptionBeefySignature without a value
func NewOptionBeefySignatureEmpty() OptionBeefySignature {
	return OptionBeefySignature{Option: Option{false}}
}

func (o OptionBeefySignature) Encode(encoder scale.Encoder) error {
	return encoder.EncodeOption(o.hasValue, o.Value)
}

func (o *OptionBeefySignature) Decode(decoder scale.Decoder) error {
	return decoder.DecodeOption(&o.hasValue, &o.Value)
}

// SetSome sets a value
func (o *OptionBeefySignature) SetSome(value BeefySignature) {
	o.hasValue = true
	o.Value = value
}

// SetNone removes a value and marks it as missing
func (o *OptionBeefySignature) SetNone() {
	o.hasValue = false
	o.Value = BeefySignature{}
}

// Unwrap returns a flag that indicates whether a value is present and the stored value
func (o OptionBeefySignature) Unwrap() (ok bool, value BeefySignature) {
	return o.hasValue, o.Value
}

type Option struct {
	hasValue bool
}

// IsNone returns true if the value is missing
func (o Option) IsNone() bool {
	return !o.hasValue
}

// IsNone returns true if a value is present
func (o Option) IsSome() bool {
	return o.hasValue
}
