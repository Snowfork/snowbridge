// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package ethereum

import (
	"bytes"

	etypes "github.com/ethereum/go-ethereum/core/types"
	"github.com/snowfork/go-substrate-rpc-client/types"
	"github.com/snowfork/polkadot-ethereum/bridgerelayer/chain"
	"github.com/snowfork/polkadot-ethereum/bridgerelayer/crypto/secp256k1"
	"github.com/snowfork/polkadot-ethereum/prover"
)

type Payload struct {
	Data      []byte
	Signature []byte
}

func MakeMessageFromEvent(event etypes.Log, kp *secp256k1.Keypair) (*chain.Message, error) {
	var appID [32]byte
	copy(appID[:], event.Address.Bytes())

	// RLP encode event log's Address, Topics, and Data
	var buf bytes.Buffer
	err := event.EncodeRLP(&buf)
	if err != nil {
		return nil, err
	}

	// Generate a proof by signing a hash of the encoded data
	proof, err := prover.GenerateProof(buf.Bytes(), kp.PrivateKey())
	if err != nil {
		return nil, err
	}

	p := Payload{Data: buf.Bytes(), Signature: proof.Signature}
	payload, err := types.EncodeToBytes(p)
	if err != nil {
		return nil, err
	}

	msg := chain.Message{AppID: appID, Payload: payload}

	return &msg, nil
}
