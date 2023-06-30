// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package ethereum

import (
	"bytes"
	"encoding/hex"

	etypes "github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/rlp"
	etrie "github.com/ethereum/go-ethereum/trie"
	"github.com/sirupsen/logrus"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/chain/parachain"

	log "github.com/sirupsen/logrus"
)

func MakeMessageFromEvent(event *etypes.Log, receiptsTrie *etrie.Trie) (*parachain.Message, error) {
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

	proof := parachain.NewProofData()
	err = receiptsTrie.Prove(receiptKey, 0, proof)
	if err != nil {
		return nil, err
	}

	m := parachain.Message{
		Data: buf.Bytes(),
		Proof: parachain.Proof{
			BlockHash: types.NewH256(event.BlockHash.Bytes()),
			TxIndex:   types.NewU32(uint32(event.TxIndex)),
			Data:      proof,
		},
	}

	type PrintProof struct {
		Keys   []string
		Values []string
	}

	printProof := PrintProof{
		Keys:   make([]string, 0),
		Values: make([]string, 0),
	}

	for _, b := range proof.Keys {
		printProof.Keys = append(printProof.Keys, hex.EncodeToString(b))
	}
	for _, b := range proof.Values {
		printProof.Values = append(printProof.Values, hex.EncodeToString(b))
	}

	value := hex.EncodeToString(m.Data)
	log.WithFields(logrus.Fields{
		"payload":    value,
		"blockHash":  m.Proof.BlockHash.Hex(),
		"eventIndex": m.Proof.TxIndex,
		"proofData":  printProof,
	}).Debug("Generated message from Ethereum log")

	return &m, nil
}
