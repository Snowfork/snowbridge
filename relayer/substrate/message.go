// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package substrate

import (
	"fmt"

	gethCommon "github.com/ethereum/go-ethereum/common"
	"github.com/snowfork/go-substrate-rpc-client/v2/scale"
	"github.com/snowfork/go-substrate-rpc-client/v2/types"
)

type Message struct {
	Data              []byte
	VerificationInput VerificationInput
}

type VerificationInput struct {
	IsBasic        bool
	AsBasic        VerificationBasic
	IsReceiptProof bool
	AsReceiptProof VerificationReceiptProof
	IsNone         bool
}

type VerificationBasic struct {
	BlockNumber uint64
	EventIndex  uint32
}

type VerificationReceiptProof struct {
	BlockHash types.H256
	TxIndex   types.U32
	Proof     *Proof
}

type Proof struct {
	Keys   []types.Bytes
	Values []types.Bytes
}

func NewProof() *Proof {
	return &Proof{
		Keys:   make([]types.Bytes, 0),
		Values: make([]types.Bytes, 0),
	}
}


// For interface ethdb.KeyValueWriter
func (p *Proof) Put(key []byte, value []byte) error {
	p.Keys = append(p.Keys, types.NewBytes(gethCommon.CopyBytes(key)))
	p.Values = append(p.Values, types.NewBytes(gethCommon.CopyBytes(value)))
	return nil
}

// For interface ethdb.KeyValueWriter
func (p *Proof) Delete(_ []byte) error {
	return fmt.Errorf("Delete should never be called to generate a proof")
}


func (v VerificationInput) Encode(encoder scale.Encoder) error {
	var err1, err2 error
	switch {
	case v.IsBasic:
		err1 = encoder.PushByte(0)
		err2 = encoder.Encode(v.AsBasic)
	case v.IsReceiptProof:
		err1 = encoder.PushByte(1)
		err2 = encoder.Encode(v.AsReceiptProof)
	case v.IsNone:
		err1 = encoder.PushByte(2)
	default:
		return fmt.Errorf("Invalid variant for VerificationInput")
	}

	if err1 != nil {
		return err1
	}
	if err2 != nil {
		return err2
	}

	return nil
}

func (v *VerificationInput) Decode(decoder scale.Decoder) error {
	tag, err := decoder.ReadOneByte()
	if err != nil {
		return err
	}

	switch tag {
	case 0:
		v.IsBasic = true
		err = decoder.Decode(&v.AsBasic)
	case 1:
		v.IsReceiptProof = true
		err = decoder.Decode(&v.AsReceiptProof)
	case 2:
		v.IsNone = true
	default:
		return fmt.Errorf("Invalid variant for VerificationInput")
	}

	if err != nil {
		return err
	}

	return nil
}
