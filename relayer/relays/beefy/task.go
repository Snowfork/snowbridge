package beefy

import (
	"github.com/ethereum/go-ethereum/common"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
)

type Task struct {
	Validators       []common.Address
	SignedCommitment types.SignedCommitment
	ValidationID     int64
}
