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
	// Used to determine the execution headers that need to be backfilled.
	Headers []common.Hash
	// Stores the last attempted finalized header, whether the import succeeded or not.
	LastAttemptedSyncHash common.Hash
	// Stores the slot number of the above header
	LastAttemptedSyncSlot uint64
}
