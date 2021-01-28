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

func MakeMessageFromEvent(event *outbound.ContractMessage, receiptsTrie *etrie.Trie, log *logrus.Entry) (chain.Message, error) {
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

	proof := substrate.NewProof()
	err = receiptsTrie.Prove(receiptKey, 0, proof)
	if err != nil {
		return nil, err
	}

	message := substrate.Message{
		Data: buf.Bytes(),
		VerificationInput: substrate.VerificationInput{
			IsReceiptProof: true,
			AsReceiptProof: substrate.VerificationReceiptProof{
				BlockHash: types.NewH256(event.Raw.BlockHash.Bytes()),
				TxIndex:   types.NewU32(uint32(event.Raw.TxIndex)),
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

	msg := chain.Message(chain.EthereumOutboundMessage{AppID: event.Source, Payload: message})
	
	return msg, nil
}
