// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package ethereum

import (
	"bytes"
	"encoding/hex"
	"fmt"

	"github.com/centrifuge/go-substrate-rpc-client/scale"
	"github.com/centrifuge/go-substrate-rpc-client/types"
	"github.com/ethereum/go-ethereum/common"
	etypes "github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/rlp"
	etrie "github.com/ethereum/go-ethereum/trie"
	"github.com/sirupsen/logrus"
	"github.com/snowfork/polkadot-ethereum/relayer/chain"
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
	p.Keys = append(p.Keys, types.NewBytes(common.CopyBytes(key)))
	p.Values = append(p.Values, types.NewBytes(common.CopyBytes(value)))
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
		return fmt.Errorf("VerificationInput must be one of the corresponding Rust enum types")
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
		return fmt.Errorf("VerificationInput must be one of the corresponding Rust enum types")
	}

	if err != nil {
		return err
	}

	return nil
}

func MakeMessageFromEvent(event *etypes.Log, receiptsTrie *etrie.Trie, log *logrus.Entry) (*chain.Message, error) {
	// RLP encode event log's Address, Topics, and Data
	var buf bytes.Buffer
	err := event.EncodeRLP(&buf)
	if err != nil {
		return nil, err
	}

	receiptKey, err := rlp.EncodeToBytes(event.TxIndex)
	if err != nil {
		return nil, err
	}

	proof := NewProof()
	err = receiptsTrie.Prove(receiptKey, 0, proof)
	if err != nil {
		return nil, err
	}

	message := Message{
		Data: buf.Bytes(),
		VerificationInput: VerificationInput{
			IsReceiptProof: true,
			AsReceiptProof: VerificationReceiptProof{
				BlockHash: types.NewH256(event.BlockHash.Bytes()),
				TxIndex:   types.NewU32(uint32(event.TxIndex)),
				Proof:     proof,
			},
		},
	}

	value := hex.EncodeToString(message.Data)
	log.WithFields(logrus.Fields{
		"payload":     value,
		"blockNumber": message.VerificationInput.AsBasic.BlockNumber,
		"eventIndex":  message.VerificationInput.AsBasic.EventIndex,
	}).Debug("Generated message from Ethereum log")

	msg := chain.Message{AppID: event.Address, Payload: message}

	return &msg, nil
}
