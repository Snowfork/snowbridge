// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package ethereum

import (
	"bytes"
	"encoding/hex"

	"github.com/ethereum/go-ethereum/rlp"
	etrie "github.com/ethereum/go-ethereum/trie"
	"github.com/sirupsen/logrus"
	"github.com/snowfork/go-substrate-rpc-client/v2/types"
	"github.com/snowfork/polkadot-ethereum/relayer/chain"
	"github.com/snowfork/polkadot-ethereum/relayer/contracts/outbound"
	"github.com/snowfork/polkadot-ethereum/relayer/substrate"
)

func MakeMessageFromEvent(event *outbound.ContractMessage, receiptsTrie *etrie.Trie, log *logrus.Entry) (*chain.EthereumOutboundMessage, error) {
	// RLP encode event log's Address, Topics, and Data
	var buf bytes.Buffer
	err := event.Raw.EncodeRLP(&buf)
	if err != nil {
		return nil, err
	}

	receiptKey, err := rlp.EncodeToBytes(event.Raw.TxIndex)
	if err != nil {
		return nil, err
	}

	proof := substrate.NewMerkleProof()
	err = receiptsTrie.Prove(receiptKey, 0, proof)
	if err != nil {
		return nil, err
	}

	message := substrate.Message{
		Data: buf.Bytes(),
		Proof: substrate.Proof{
			BlockHash:   types.NewH256(event.Raw.BlockHash.Bytes()),
			TxIndex:     types.NewU32(uint32(event.Raw.TxIndex)),
			MerkleProof: proof,
		},
	}

	value := hex.EncodeToString(message.Data)
	log.WithFields(logrus.Fields{
		"payload":    value,
		"blockHash":  message.Proof.BlockHash.Hex(),
		"eventIndex": message.Proof.TxIndex,
	}).Debug("Generated message from Ethereum log")

	msg := chain.EthereumOutboundMessage(message)

	return &msg, nil
}
