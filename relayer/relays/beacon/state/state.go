package state

import (
	"github.com/ethereum/go-ethereum/common"
)

type ExecutionHeader struct {
	BeaconHeaderBlockRoot common.Hash
	BeaconHeaderSlot      uint64
	BlockHash             common.Hash
	BlockNumber           uint64
}
