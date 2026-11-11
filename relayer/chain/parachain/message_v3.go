// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package parachain

import (
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/relays/util"
)

type EventLogV3 struct {
	Address types.H160
	Topics  []types.H256
	Data    types.Bytes
	TxIndex uint64
}

type MessageV3 struct {
	EventLog EventLogV3
	Proof    ProofV2
}

type MessageV3JSON struct {
	EventLog EventLogV3JSON `json:"event_log"`
	Proof    ProofV2JSON    `json:"proof"`
}

type EventLogV3JSON struct {
	Address string   `json:"address"`
	Topics  []string `json:"topics"`
	Data    string   `json:"data"`
	TxIndex uint64   `json:"tx_index"`
}

func (m MessageV3) ToJSON() MessageV3JSON {
	return MessageV3JSON{
		EventLog: EventLogV3JSON{
			Address: m.EventLog.Address.Hex(),
			Topics:  util.ScaleBranchToString(m.EventLog.Topics),
			Data:    m.EventLog.Data.Hex(),
			TxIndex: m.EventLog.TxIndex,
		},
		Proof: ProofV2JSON{
			ReceiptProof: &ProofDataV2JSON{
				Values: util.ScaleBytesToArrayHexArray(m.Proof.ReceiptProof.Values),
			},
			ExecutionProof: m.Proof.ExecutionProof.ToJSON(),
		},
	}
}

func (m *MessageV3JSON) RemoveLeadingZeroHashes() {
	m.EventLog.RemoveLeadingZeroHashes()
	m.Proof.RemoveLeadingZeroHashes()
}

func (e *EventLogV3JSON) RemoveLeadingZeroHashes() {
	e.Address = removeLeadingZeroHash(e.Address)
	e.Topics = removeLeadingZeroHashForSlice(e.Topics)
	e.Data = removeLeadingZeroHash(e.Data)
}

func ConvertToV3Message(inboundMsg *MessageV3) (*MessageV3, error) {
	if inboundMsg == nil {
		return nil, nil
	}
	return inboundMsg, nil
}

func ConvertToV2Message(inboundMsg *MessageV3) (*MessageV2, error) {
	if inboundMsg == nil {
		return nil, nil
	}
	proofV2 := ProofV2{
		ExecutionProof: inboundMsg.Proof.ExecutionProof,
	}
	if inboundMsg.Proof.ReceiptProof != nil {
		proofV2.ReceiptProof = &ProofDataV2{
			Values: inboundMsg.Proof.ReceiptProof.Values,
		}
	}
	return &MessageV2{
		EventLog: EventLog{
			Address: inboundMsg.EventLog.Address,
			Topics:  inboundMsg.EventLog.Topics,
			Data:    inboundMsg.EventLog.Data,
		},
		Proof: proofV2,
	}, nil
}

func ConvertLegacyToV3Message(inboundMsg *Message, txIndex uint64) (*MessageV3, error) {
	if inboundMsg == nil {
		return nil, nil
	}
	proofV2 := ProofV2{
		ExecutionProof: inboundMsg.Proof.ExecutionProof,
	}
	if inboundMsg.Proof.ReceiptProof != nil {
		proofV2.ReceiptProof = &ProofDataV2{
			Values: inboundMsg.Proof.ReceiptProof.Values,
		}
	}
	return &MessageV3{
		EventLog: EventLogV3{
			Address: inboundMsg.EventLog.Address,
			Topics:  inboundMsg.EventLog.Topics,
			Data:    inboundMsg.EventLog.Data,
			TxIndex: txIndex,
		},
		Proof: proofV2,
	}, nil
}
