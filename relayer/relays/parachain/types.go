package parachain

import (
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/chain/parachain"
	"github.com/snowfork/snowbridge/relayer/chain/relaychain"
	"github.com/snowfork/snowbridge/relayer/crypto/merkle"
)

// A Task is a bundle of items needed to submit commitments to Ethereum
type Task struct {
	ParaID      uint32
	BlockNumber uint64
	Header      *types.Header
	Commitments map[parachain.ChannelID]Commitment
	ProofInput  *ProofInput
	ProofOutput *ProofOutput
}

// A Commitment is data provably attested to by polkadot. The commitment hash
// is contained in a parachain header. Polkadot validator nodes attest that the header
// is genuine.
type Commitment struct {
	Hash types.H256
	Data interface{}
}

func NewCommitment(hash types.H256, data interface{}) Commitment {
	return Commitment{
		Hash: hash,
		Data: data,
	}
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
