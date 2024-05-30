package beefy

import (
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/crypto/merkle"
	"github.com/snowfork/snowbridge/relayer/substrate"
)

type BeefyAuthoritySet struct {
	// ID
	ID types.U64
	// Number of validators in the set.
	Len types.U32
	// Merkle Root Hash build from BEEFY uncompressed AuthorityIds.
	Root types.H256
}

type Request struct {
	// Validators that signed this commitment
	Validators       []substrate.Authority
	ValidatorsRoot   [32]byte
	SignedCommitment types.SignedCommitment
	Proof            merkle.SimplifiedMMRProof
}
