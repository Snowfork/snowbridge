package parachain

import (
	"math/big"

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
	// Commitments for basic channel
	BasicChannelProofs *[]MessageProof
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

type RawMerkleProof struct {
	Root           types.H256
	Proof          []types.H256
	NumberOfLeaves uint64
	LeafIndex      uint64
	Leaf           []byte
}

type MerkleProof struct {
	Root        types.H256
	InnerHashes [][32]byte
	HashSides   []bool
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
	Origin  uint32
	Nonce   types.UCompact
	Handler uint16
	Payload []byte
}

func (m OutboundQueueMessage) IntoInboundMessage() contracts.InboundQueueMessage {
	return contracts.InboundQueueMessage{
		Origin:  m.Origin,
		Nonce:   (*big.Int)(&m.Nonce).Uint64(),
		Handler: m.Handler,
		Payload: m.Payload,
	}
}

type MessageProof struct {
	Message OutboundQueueMessage
	Proof   MerkleProof
}
