// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package ethereum

import (
	"bytes"

	etypes "github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/rlp"
	etrie "github.com/ethereum/go-ethereum/trie"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/chain/parachain"
)

func MakeMessageFromEvent(event *etypes.Log, receiptsTrie *etrie.Trie) (*parachain.MessageV3, error) {
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

	legacyProof := parachain.NewProofData()
	err = receiptsTrie.Prove(receiptKey, legacyProof)
	if err != nil {
		return nil, err
	}

	var convertedTopics []types.H256
	for _, topic := range event.Topics {
		convertedTopics = append(convertedTopics, types.H256(topic))
	}

	legacyMessage := parachain.Message{
		EventLog: parachain.EventLog{
			Address: types.H160(event.Address),
			Topics:  convertedTopics,
			Data:    event.Data,
		},
		Proof: parachain.Proof{
			ReceiptProof: legacyProof,
		},
	}

	return parachain.ConvertLegacyToV3Message(&legacyMessage, uint64(event.TxIndex))
}
