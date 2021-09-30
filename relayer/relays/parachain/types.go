package parachain

import (
	"github.com/snowfork/go-substrate-rpc-client/v3/types"
	"github.com/snowfork/snowbridge/relayer/chain/parachain"
	"github.com/snowfork/snowbridge/relayer/chain/relaychain"
	"github.com/snowfork/snowbridge/relayer/crypto/merkle"
)

type Task struct {
	ParaID      uint32
	BlockNumber uint64
	Header      *types.Header
	Commitments map[parachain.ChannelID]Commitment
	ProofInput  *ProofInput
	ProofOutput *ProofOutput
}

type Commitment struct {
	Hash types.H256
	Data interface{}
}

type ProofInput struct {
	PolkadotBlockNumber uint64
	ParaHeads           []relaychain.ParaHead
}

type ProofOutput struct {
	MMRProof        merkle.SimplifiedMMRProof
	MMRRootHash     types.Hash
	Header          types.Header
	MerkleProofData MerkleProofData
}
