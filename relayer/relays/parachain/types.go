package parachain

import (
	"math/big"

	"github.com/snowfork/go-substrate-rpc-client/v4/scale"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/chain/relaychain"
	"github.com/snowfork/snowbridge/relayer/contracts"
	"github.com/snowfork/snowbridge/relayer/crypto/merkle"
)

// A Task contains the working state for message commitments in a single parachain block
type Task struct {
	// Parachain header
	Header *types.Header
	// Inputs for MMR proof generation
	ProofInput *ProofInput
	// Outputs of MMR proof generation
	ProofOutput *ProofOutput
	// Proofs for messages from outbound channel on Polkadot
	MessageProofs *[]MessageProof
}

// A ProofInput is data needed to generate a proof of parachain header inclusion
type ProofInput struct {
	// Parachain ID
	ParaID uint32
	// Relay chain block number in which our parachain head was included
	RelayBlockNumber uint64
	// Relay chain block hash in which our parachain head was included
	RelayBlockHash types.Hash
	// All included paraheads in RelayBlockNumber
	ParaHeads []relaychain.ParaHead
}

// A ProofOutput represents the generated header inclusion proof
type ProofOutput struct {
	MMRProof        merkle.SimplifiedMMRProof
	MMRRootHash     types.Hash
	Header          types.Header
	MerkleProofData MerkleProofData
}

type OptionRawMerkleProof struct {
	HasValue bool
	Value    RawMerkleProof
}

func (o OptionRawMerkleProof) Encode(encoder scale.Encoder) error {
	return encoder.EncodeOption(o.HasValue, o.Value)
}

func (o *OptionRawMerkleProof) Decode(decoder scale.Decoder) error {
	return decoder.DecodeOption(&o.HasValue, &o.Value)
}

type RawMerkleProof struct {
	Root           types.H256
	Proof          []types.H256
	NumberOfLeaves uint64
	LeafIndex      uint64
	Leaf           types.H256
}

type MerkleProof struct {
	Root        types.H256
	InnerHashes [][32]byte
}

func NewMerkleProof(rawProof RawMerkleProof) (MerkleProof, error) {
	var proof MerkleProof

	byteArrayProof := make([][32]byte, len(rawProof.Proof))
	for i := 0; i < len(rawProof.Proof); i++ {
		byteArrayProof[i] = ([32]byte)(rawProof.Proof[i])
	}

	proof = MerkleProof{
		Root:        rawProof.Root,
		InnerHashes: byteArrayProof,
	}

	return proof, nil
}

type OutboundQueueMessage struct {
	Origin   types.H256
	Nonce    types.U64
	Commands []CommandWrapper
}

type CommandWrapper struct {
	Kind           types.U8
	MaxDispatchGas types.U64
	Params         types.Bytes
}

func (r CommandWrapper) IntoCommand() contracts.Command {
	return contracts.Command{
		Kind:    uint8(r.Kind),
		Gas:     uint64(r.MaxDispatchGas),
		Payload: r.Params,
	}
}

func (m OutboundQueueMessage) IntoInboundMessage() contracts.InboundMessage {
	var commands []contracts.Command
	for _, command := range m.Commands {
		commands = append(commands, command.IntoCommand())
	}
	return contracts.InboundMessage{
		Origin:   m.Origin,
		Nonce:    uint64(m.Nonce),
		Commands: commands,
	}
}

type MessageProof struct {
	Message OutboundQueueMessage
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
