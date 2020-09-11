// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package ethereum

import (
	"bytes"

	etypes "github.com/ethereum/go-ethereum/core/types"
	"github.com/snowfork/go-substrate-rpc-client/scale"
	"github.com/snowfork/go-substrate-rpc-client/types"
	"github.com/snowfork/polkadot-ethereum/bridgerelayer/chain"
)

type Message struct {
	Data              []byte
	VerificationInput VerificationInput
}

type VerificationInput struct {
	IsBasic bool
	AsBasic VerificationBasic
	IsNone  bool
}

type VerificationBasic struct {
	BlockNumber uint64
	EventIndex  uint32
}

func (v VerificationInput) Encode(encoder scale.Encoder) error {
	var err error
	if v.IsBasic {
		err = encoder.PushByte(0)
		if err != nil {
			return err
		}
		err = encoder.Encode(v.AsBasic)
		if err != nil {
			return err
		}
	} else if v.IsNone {
		err = encoder.PushByte(1)
		if err != nil {
			return err
		}
	}
	return nil
}

func (v *VerificationInput) Decode(decoder scale.Decoder) error {
	tag, err := decoder.ReadOneByte()
	if err != nil {
		return err
	}

	if tag == 0 {
		v.IsBasic = true
		err = decoder.Decode(&v.AsBasic)
		if err != nil {
			return err
		}
	} else if tag == 1 {
		v.IsNone = true
	}

	return nil
}

func MakeMessageFromEvent(event etypes.Log) (*chain.Message, error) {
	var appID [32]byte
	copy(appID[:], event.Address.Bytes())

	// RLP encode event log's Address, Topics, and Data
	var buf bytes.Buffer
	err := event.EncodeRLP(&buf)
	if err != nil {
		return nil, err
	}

	message := Message{
		Data: buf.Bytes(),
		VerificationInput: VerificationInput{
			IsBasic: true,
			AsBasic: VerificationBasic{
				BlockNumber: event.BlockNumber,
				EventIndex:  uint32(event.Index),
			},
		},
	}
	payload, err := types.EncodeToBytes(message)
	if err != nil {
		return nil, err
	}

	msg := chain.Message{AppID: appID, Payload: payload}

	return &msg, nil
}
