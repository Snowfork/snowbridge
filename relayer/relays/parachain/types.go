package parachain

import (
	"math/big"

	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/chain/relaychain"
	"github.com/snowfork/snowbridge/relayer/contracts"
	"github.com/snowfork/snowbridge/relayer/crypto/merkle"
	"github.com/vedhavyas/go-subkey/scale"
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
	Origin      uint32
	Nonce       uint64
	Command     uint8
	Params      []byte
	DispatchGas types.U128
}

func (m OutboundQueueMessage) IntoInboundMessage() contracts.InboundMessage {
	return contracts.InboundMessage{
		Origin:      big.NewInt(int64(m.Origin)),
		Nonce:       m.Nonce,
		Command:     m.Command,
		Params:      m.Params,
		DispatchGas: m.DispatchGas.Int,
	}
}

type MessageProof struct {
	Message OutboundQueueMessage
	Proof   MerkleProof
}
