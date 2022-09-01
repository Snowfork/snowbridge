package parachain

import (
	"fmt"
	"math/bits"

	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/chain/relaychain"
	"github.com/snowfork/snowbridge/relayer/crypto/merkle"
)

// A Task is a bundle of items needed to submit commitments to Ethereum
type Task struct {
	ParaID                        uint32
	BlockNumber                   uint64
	Header                        *types.Header
	BasicChannelProofs            *[]MerkleProof
	IncentivizedChannelCommitment *IncentivizedChannelCommitment
	ProofInput                    *ProofInput
	ProofOutput                   *ProofOutput
}

// A Commitment is data provably attested to by polkadot. The commitment hash
// is contained in a parachain header. Polkadot validator nodes attest that the header
// is genuine.
type IncentivizedChannelCommitment struct {
	Hash types.H256
	Data IncentivizedOutboundChannelMessageBundle
}

func NewIncentivizedChannelCommitment(hash types.H256, data IncentivizedOutboundChannelMessageBundle) IncentivizedChannelCommitment {
	return IncentivizedChannelCommitment{
		Hash: hash,
		Data: data,
	}
}

type RawMerkleProof struct {
	Root  types.H256
	Proof []types.H256
	// TODO: test that this decodes properly
	// Leaf  []byte
	Leaf BasicOutboundChannelMessageBundle

	NumberOfLeaves uint64
	LeafIndex      uint64
}

type MerkleProof struct {
	Root  types.H256
	Proof [][32]byte
	Leaf  BasicOutboundChannelMessageBundle

	HashSides []bool
}

func NewMerkleProof(rawProof RawMerkleProof) (MerkleProof, error) {
	var proof MerkleProof

	byteArrayProof := make([][32]byte, len(rawProof.Proof))
	for i := 0; i < len(rawProof.Proof); i++ {
		byteArrayProof[i] = ([32]byte)(rawProof.Proof[i])
	}

	hashSides, err := generateHashSides(rawProof)
	if err != nil {
		return proof, err
	}

	proof = MerkleProof{
		Root:      rawProof.Root,
		Proof:     byteArrayProof,
		HashSides: hashSides,
		Leaf:      rawProof.Leaf,
	}

	return proof, nil
}

func generateHashSides(commitmentProof RawMerkleProof) ([]bool, error) {
	pos := commitmentProof.LeafIndex
	width := commitmentProof.NumberOfLeaves

	if pos >= width {
		return nil, fmt.Errorf("leaf position %v is too high in proof with %v leaves", pos, width)
	}

	if width == 0 {
		return nil, fmt.Errorf("no hash sides for an empty proof")
	}

	// The number of intermediate hashes is the height of the complete tree, which is the base 2 log of the number of leaves, rounded up.
	// This is equivalent to the number of bits after the most significant bit, which is what we use here.
	numSides := 64 - bits.LeadingZeros64(width-1)
	sides := make([]bool, numSides)
	for i := 0; i < numSides; i++ {
		sides[i] = pos%2 == 1
		pos /= 2
		width = ((width - 1) / 2) + 1
	}

	return sides, nil
}

// A ProofInput is data needed to generate a proof of parachain header inclusion
type ProofInput struct {
	PolkadotBlockNumber uint64
	ParaHeads           []relaychain.ParaHead
}

// A ProofOutput represents the generated header inclusion proof
type ProofOutput struct {
	MMRProof        merkle.SimplifiedMMRProof
	MMRRootHash     types.Hash
	Header          types.Header
	MerkleProofData MerkleProofData
}
