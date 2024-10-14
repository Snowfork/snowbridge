// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package parachain

import (
	"fmt"
	"strings"

	gethCommon "github.com/ethereum/go-ethereum/common"
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

type DestinationEnum uint8

const (
	DestinationAccountId32 DestinationEnum = iota
	DestinationForeignAccountId32
	DestinationForeignAccountId20
)

type Destination struct {
	Variant DestinationEnum
	// Use pointers to handle variant fields, only one should be populated.
	AccountId32        *types.H256
	ForeignAccountId32 *ForeignAccountId32
	ForeignAccountId20 *ForeignAccountId20
}

type AccountId32 struct {
	ID [32]byte
}

type ForeignAccountId32 struct {
	ParaID uint32
	ID     types.H256
	Fee    types.U128
}

type ForeignAccountId20 struct {
	ParaID uint32
	ID     types.H160
	Fee    types.U128
}

type RegisterToken struct {
	Token types.H256
	Fee   types.U128
}

type SendToken struct {
	Token       types.H256
	Destination Destination
	Amount      types.U128
	Fee         types.U128
}

type SendNativeToken struct {
	TokenID     types.H256
	Destination Destination
	Amount      types.U128
	Fee         types.U128
}

type InboundMessage struct {
	Version      types.U8
	ChainID      types.U64
	Command      types.U8
	CommandBytes types.Data
}

type Envelope struct {
	Gateway   types.H256
	ChannelID types.H256
	Nonce     types.U64
	MessageID types.H256
	Payload   InboundMessage
}

func GetDestination(input []byte) (string, error) {
	var envelope = &Envelope{}
	err := types.DecodeFromBytes(input, envelope)
	if err != nil {
		return "", fmt.Errorf("failed to decode message: %v", err)
	}

	address := ""
	switch envelope.Payload.Command {
	case 0:
		// Register token does not have a destination
		break
	case 1:
		// Send token has destination
		var command = &SendToken{}
		err = types.DecodeFromBytes(input, command)
		if err != nil {
			return "", fmt.Errorf("failed to decode send token command: %v", err)
		}

		switch command.Destination.Variant {
		case 0:
			address = command.Destination.AccountId32.Hex()
		case 1:
			address = command.Destination.ForeignAccountId32.ID.Hex()
		case 2:
			address = command.Destination.ForeignAccountId20.ID.Hex()
		}
	case 2:
		// Send native token has destination
		var command = &SendNativeToken{}
		err = types.DecodeFromBytes(input, command)
		if err != nil {
			return "", fmt.Errorf("failed to decode send native token command: %v", err)
		}

		switch command.Destination.Variant {
		case 0:
			address = command.Destination.AccountId32.Hex()
		case 1:
			address = command.Destination.ForeignAccountId32.ID.Hex()
		case 2:
			address = command.Destination.ForeignAccountId20.ID.Hex()
		}
	}

	return address, nil
}
