package parachain

import (
	"math/big"

	"github.com/snowfork/go-substrate-rpc-client/v4/scale"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/contracts"
)

type OutboundQueueMessageV2 struct {
	Origin   types.H256
	Nonce    types.U64
	ID       types.H256
	Commands []CommandWrapper
}

type CommandWrapper struct {
	Kind           types.U8
	MaxDispatchGas types.U64
	Params         types.Bytes
}

func (r CommandWrapper) IntoCommandV2() contracts.Command {
	return contracts.Command{
		Kind:    uint8(r.Kind),
		Gas:     uint64(r.MaxDispatchGas),
		Payload: r.Params,
	}
}

func (m OutboundQueueMessageV2) IntoInboundMessageV2() contracts.InboundMessageV2 {
	var commands []contracts.Command
	for _, command := range m.Commands {
		commands = append(commands, command.IntoCommandV2())
	}
	return contracts.InboundMessageV2{
		Origin:   m.Origin,
		Nonce:    uint64(m.Nonce),
		Commands: commands,
	}
}

// A Task contains the working state for message commitments in a single parachain block
type TaskV2 struct {
	// Parachain header
	Header *types.Header
	// Inputs for MMR proof generation
	ProofInput *ProofInput
	// Outputs of MMR proof generation
	ProofOutput *ProofOutput
	// Proofs for messages from outbound channel on Polkadot
	MessageProofs *[]MessageProofV2
}

type MessageProofV2 struct {
	Message OutboundQueueMessageV2
	Proof   MerkleProof
}

type PendingOrder struct {
	Nonce       uint64
	BlockNumber uint32
	Fee         big.Int
}

func (p *PendingOrder) Decode(decoder scale.Decoder) error {
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
