// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package ethereum

import (
	"bytes"

	etypes "github.com/ethereum/go-ethereum/core/types"
	"github.com/snowfork/go-substrate-rpc-client/types"
	"github.com/snowfork/polkadot-ethereum/bridgerelayer/chain"
)

type Payload struct {
	Data        []byte
	TxHash      [32]byte
	BlockNumber uint64
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

	p := Payload{Data: buf.Bytes(), TxHash: event.TxHash, BlockNumber: event.BlockNumber}
	payload, err := types.EncodeToBytes(p)
	if err != nil {
		return nil, err
	}

	msg := chain.Message{AppID: appID, Payload: payload}

	return &msg, nil
}
