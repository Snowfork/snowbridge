package beefy

import (
	"github.com/ethereum/go-ethereum/common"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/crypto/merkle"
)

type Task struct {
	Validators       []common.Address
	SignedCommitment types.SignedCommitment
	Proof            merkle.SimplifiedMMRProof
	ValidationID     int64
}
