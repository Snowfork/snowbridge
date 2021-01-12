// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package ethereum

import (
	"bytes"
	"encoding/hex"
	"fmt"

	"github.com/ethereum/go-ethereum/common"
	etypes "github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/rlp"
	etrie "github.com/ethereum/go-ethereum/trie"
	"github.com/sirupsen/logrus"
	"github.com/snowfork/go-substrate-rpc-client/v2/scale"
	"github.com/snowfork/go-substrate-rpc-client/v2/types"
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

func MakeMessageFromEvent(event *etypes.Log, receiptsTrie *etrie.Trie, log *logrus.Entry) (*Message, error) {
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
		"payload":    value,
		"blockHash":  message.VerificationInput.AsReceiptProof.BlockHash.Hex(),
		"eventIndex": message.VerificationInput.AsReceiptProof.TxIndex,
	}).Debug("Generated message from Ethereum log")

	return &message, nil
}

func MakeMessageChunker(messagesByAddress map[common.Address][]*Message, chunkSize int) func() ([]chain.Message, bool) {
	i, offset := 0, 0
	addresses := make([]common.Address, 0, len(messagesByAddress))
	for k := range messagesByAddress {
		addresses = append(addresses, k)
	}

	return func() ([]chain.Message, bool) {
		chunk := make([]chain.Message, 0)
		count := 0

		for i < len(addresses) {
			if count == chunkSize {
				return chunk, true
			}

			address := addresses[i]
			messagesForAddress := messagesByAddress[address]
			r := chunkSize - count
			start := offset
			end := offset + r
			if end >= len(messagesForAddress) {
				end = len(messagesForAddress)
				i++
				offset = 0
			} else {
				offset += r
			}

			part := messagesForAddress[start:end]
			chunk = append(chunk, chain.Message{AppID: address, Payload: part})

			count += len(part)
		}

		return chunk, false
	}
}
