// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package parachain

import (
	"fmt"
	"math/big"
	"strings"

	gethCommon "github.com/ethereum/go-ethereum/common"
	sc "github.com/snowfork/go-substrate-rpc-client/v4/scale"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/json"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/scale"
	"github.com/snowfork/snowbridge/relayer/relays/util"
)

type EventLog struct {
	Address types.H160
	Topics  []types.H256
	Data    types.Bytes
}

type Message struct {
	EventLog EventLog
	Proof    Proof
}

type Proof struct {
	ReceiptProof   *ProofData
	ExecutionProof scale.HeaderUpdatePayload
}

type ProofData struct {
	Keys   []types.Bytes
	Values []types.Bytes
}

type MessageJSON struct {
	EventLog EventLogJSON `json:"event_log"`
	Proof    ProofJSON    `json:"proof"`
}

type EventLogJSON struct {
	Address string   `json:"address"`
	Topics  []string `json:"topics"`
	Data    string   `json:"data"`
}

type ProofJSON struct {
	ReceiptProof   *ProofDataJSON    `json:"receipt_proof"`
	ExecutionProof json.HeaderUpdate `json:"execution_proof"`
}

type ProofDataJSON struct {
	Keys   []string `json:"keys"`
	Values []string `json:"values"`
}

func NewProofData() *ProofData {
	return &ProofData{
		Keys:   make([]types.Bytes, 0),
		Values: make([]types.Bytes, 0),
	}
}

// For interface ethdb.KeyValueWriter
func (p *ProofData) Put(key []byte, value []byte) error {
	p.Keys = append(p.Keys, types.NewBytes(gethCommon.CopyBytes(key)))
	p.Values = append(p.Values, types.NewBytes(gethCommon.CopyBytes(value)))
	return nil
}

// For interface ethdb.KeyValueWriter
func (p *ProofData) Delete(_ []byte) error {
	return fmt.Errorf("Delete should never be called to generate a proof")
}

func (m Message) ToJSON() MessageJSON {
	return MessageJSON{
		EventLog: EventLogJSON{
			Address: m.EventLog.Address.Hex(),
			Topics:  util.ScaleBranchToString(m.EventLog.Topics),
			Data:    m.EventLog.Data.Hex(),
		},
		Proof: ProofJSON{
			ReceiptProof: &ProofDataJSON{
				Keys:   util.ScaleBytesToArrayHexArray(m.Proof.ReceiptProof.Keys),
				Values: util.ScaleBytesToArrayHexArray(m.Proof.ReceiptProof.Values),
			},
			ExecutionProof: m.Proof.ExecutionProof.ToJSON(),
		},
	}
}

func (m *MessageJSON) RemoveLeadingZeroHashes() {
	m.EventLog.RemoveLeadingZeroHashes()
	m.Proof.RemoveLeadingZeroHashes()
}

func (e *EventLogJSON) RemoveLeadingZeroHashes() {
	e.Address = removeLeadingZeroHash(e.Address)
	e.Topics = removeLeadingZeroHashForSlice(e.Topics)
	e.Data = removeLeadingZeroHash(e.Data)
}

func (p *ProofJSON) RemoveLeadingZeroHashes() {
	p.ReceiptProof.RemoveLeadingZeroHashes()
	p.ExecutionProof.RemoveLeadingZeroHashes()
}

func (p *ProofDataJSON) RemoveLeadingZeroHashes() {
	p.Keys = removeLeadingZeroHashForSlice(p.Keys)
	p.Values = removeLeadingZeroHashForSlice(p.Values)
}

func removeLeadingZeroHashForSlice(s []string) []string {
	result := make([]string, len(s))

	for i, item := range s {
		result[i] = removeLeadingZeroHash(item)
	}
	return result
}

func removeLeadingZeroHash(s string) string {
	return strings.Replace(s, "0x", "", 1)
}

// Storage ValidationData from ParachainSystem pallet
type PersistedValidationData struct {
	ParentHead             []byte
	RelayParentNumber      uint32
	RelayParentStorageRoot types.Hash
	MaxPOVSize             uint32
}

// Storage PendingOrder from EthereumOutboundQueueV2 pallet
type PendingOrder struct {
	Nonce       uint64
	BlockNumber uint32
	Fee         big.Int
}

func (p *PendingOrder) Decode(decoder sc.Decoder) error {
	var nonce types.U64
	err := decoder.Decode(&nonce)
	if err != nil {
		return err
	}
	p.Nonce = uint64(nonce)
	var blockNumber types.U32
	err = decoder.Decode(&blockNumber)
	if err != nil {
		return err
	}
	p.BlockNumber = uint32(blockNumber)
	decoded, err := decoder.DecodeUintCompact()
	if err != nil {
		return err
	}
	p.Fee = *types.U128{Int: decoded}.Int
	return nil
}

type OutboundQueueMessage struct {
	ChannelID      types.H256
	Nonce          uint64
	Command        uint8
	Params         []byte
	MaxDispatchGas uint64
	MaxFeePerGas   types.U128
	Reward         types.U128
	ID             types.Bytes32
}

func (m *OutboundQueueMessage) Decode(decoder sc.Decoder) error {
	err := decoder.Decode(&m.ChannelID)
	if err != nil {
		return err
	}
	decoded, err := decoder.DecodeUintCompact()
	if err != nil {
		return err
	}
	m.Nonce = decoded.Uint64()
	err = decoder.Decode(&m.Command)
	if err != nil {
		return err
	}
	err = decoder.Decode(&m.Params)
	if err != nil {
		return err
	}
	decoded, err = decoder.DecodeUintCompact()
	if err != nil {
		return err
	}
	m.MaxDispatchGas = decoded.Uint64()
	decoded, err = decoder.DecodeUintCompact()
	if err != nil {
		return err
	}
	m.MaxFeePerGas = types.U128{Int: decoded}
	decoded, err = decoder.DecodeUintCompact()
	if err != nil {
		return err
	}
	m.Reward = types.U128{Int: decoded}
	err = decoder.Decode(&m.ID)
	if err != nil {
		return err
	}
	return nil
}
