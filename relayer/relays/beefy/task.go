package beefy

import (
	"github.com/ethereum/go-ethereum/common"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/crypto/merkle"
)

type Status int

const (
	CommitmentWitnessed            Status = iota // 0
	InitialVerificationTxSent      Status = iota // 1
	InitialVerificationTxConfirmed Status = iota // 2
	ReadyToComplete                Status = iota // 3
	CompleteVerificationTxSent     Status = iota // 4
)

type Task struct {
	Validators            []common.Address
	SignedCommitment      types.SignedCommitment
	Proof                 merkle.SimplifiedMMRProof
	ValidationID          int64
	CompleteOnBlock       uint64
}
