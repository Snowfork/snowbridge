// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package parachain

import (
	"fmt"

	gethCommon "github.com/ethereum/go-ethereum/common"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/json"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/scale"
	"github.com/snowfork/snowbridge/relayer/relays/util"
)

type MessageV2 struct {
	EventLog EventLog
	Proof    ProofV2
}

type ProofV2 struct {
	ReceiptProof   *ProofDataV2
	ExecutionProof scale.HeaderUpdatePayload
}

type ProofDataV2 struct {
	Values []types.Bytes
}

type MessageV2JSON struct {
	EventLog EventLogJSON `json:"event_log"`
	Proof    ProofV2JSON  `json:"proof"`
}

type EventLogV2JSON struct {
	Address string   `json:"address"`
	Topics  []string `json:"topics"`
	Data    string   `json:"data"`
}

type ProofV2JSON struct {
	ReceiptProof   *ProofDataV2JSON  `json:"receipt_proof"`
	ExecutionProof json.HeaderUpdate `json:"execution_proof"`
}

type ProofDataV2JSON struct {
	Values []string `json:"values"`
}

func NewProofDataV2() *ProofDataV2 {
	return &ProofDataV2{
		Values: make([]types.Bytes, 0),
	}
}

// For interface ethdb.KeyValueWriter
func (p *ProofDataV2) Put(_ []byte, value []byte) error {
	p.Values = append(p.Values, types.NewBytes(gethCommon.CopyBytes(value)))
	return nil
}

// For interface ethdb.KeyValueWriter
func (p *ProofDataV2) Delete(_ []byte) error {
	return fmt.Errorf("Delete should never be called to generate a proof")
}

func (m MessageV2) ToJSON() MessageV2JSON {
	return MessageV2JSON{
		EventLog: EventLogJSON{
			Address: m.EventLog.Address.Hex(),
			Topics:  util.ScaleBranchToString(m.EventLog.Topics),
			Data:    m.EventLog.Data.Hex(),
		},
		Proof: ProofV2JSON{
			ReceiptProof: &ProofDataV2JSON{
				Values: util.ScaleBytesToArrayHexArray(m.Proof.ReceiptProof.Values),
			},
			ExecutionProof: m.Proof.ExecutionProof.ToJSON(),
		},
	}
}

func (m *MessageV2JSON) RemoveLeadingZeroHashes() {
	m.EventLog.RemoveLeadingZeroHashes()
	m.Proof.RemoveLeadingZeroHashes()
}

func (p *ProofV2JSON) RemoveLeadingZeroHashes() {
	p.ReceiptProof.RemoveLeadingZeroHashes()
	p.ExecutionProof.RemoveLeadingZeroHashes()
}

func (p *ProofDataV2JSON) RemoveLeadingZeroHashes() {
	p.Values = removeLeadingZeroHashForSlice(p.Values)
}

func ConvertToV2Message(inboundMsg *Message) (*MessageV2, error) {
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
		EventLog: inboundMsg.EventLog,
		Proof:    proofV2,
	}, nil
}
