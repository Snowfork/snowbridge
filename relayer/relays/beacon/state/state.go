package state

import (
	"github.com/ethereum/go-ethereum/common"
)

type ExecutionHeader struct {
	BeaconBlockRoot common.Hash
	BeaconSlot      uint64
	BlockHash       common.Hash
	BlockNumber     uint64
}

type FinalizedHeader struct {
	BeaconBlockRoot common.Hash
	BeaconSlot      uint64
}
